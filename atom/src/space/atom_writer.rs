use crate::header::AtomHeader;
use crate::space::{
    error::AtomWriteError, AtomSpace, SpaceAllocator, SpaceWriter, SpaceWriterSplitAllocation,
};
use urid::URID;

/// A [`SpaceWriter`] that tracks the amount of allocated space in an atom header.
///
/// This allows for writing dynamic, variable-size atoms without having to track their size manually.
///
/// # Example
///
/// ```
/// use lv2_atom::atom_prelude::*;
/// use urid::URID;
///
/// let mut buf = vec![0; 64];
/// let mut cursor = SpaceCursor::new(&mut buf);
///
/// let mut writer = AtomWriter::write_new(&mut cursor, URID::new(42).unwrap()).unwrap();
///
/// let message = b"Hello, world!";
/// writer.write_bytes(message).unwrap();
/// assert_eq!(writer.atom_header().size_of_body(), message.len());
/// ```
pub struct AtomWriter<'a> {
    atom_header_index: usize,
    parent: &'a mut (dyn SpaceAllocator),
}

impl<'a> AtomWriter<'a> {
    /// Retrieves a copy of the header this writer is currently tracking.
    #[inline]
    pub fn atom_header(&self) -> AtomHeader {
        let previous = self
            .parent
            .allocated_bytes()
            .get(self.atom_header_index..)
            .expect("Unable to locate atom header");

        let space = AtomSpace::from_bytes(previous).expect("Atom header location is unaligned");

        unsafe { space.assume_init_slice()[0] }
    }

    #[inline]
    fn atom_header_mut(&mut self) -> &mut AtomHeader {
        let previous = unsafe { self.parent.allocated_bytes_mut() }
            .get_mut(self.atom_header_index..)
            .unwrap();
        let space = AtomSpace::from_bytes_mut(previous).unwrap();

        unsafe { &mut space.assume_init_slice_mut()[0] }
    }

    /// Writes an atom header into the given [`SpaceWriter`], and returns a new writer that starts
    /// tracking its size.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    pub fn write_new<A: ?Sized>(
        parent: &'a mut impl SpaceWriter,
        urid: URID<A>,
    ) -> Result<Self, AtomWriteError> {
        let atom = AtomHeader::new(urid);

        parent.write_value(atom)?;
        let atom_header_index = parent.allocated_bytes().len() - std::mem::size_of::<AtomHeader>();

        Ok(Self {
            atom_header_index,
            parent,
        })
    }
}

impl<'a> SpaceAllocator for AtomWriter<'a> {
    #[inline]
    fn allocate_and_split(
        &mut self,
        size: usize,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError> {
        let alloc = self.parent.allocate_and_split(size)?;

        let space = AtomSpace::from_bytes_mut(
            // PANIC: We rely on the parent allocator not shifting bytes around
            &mut alloc.previous[self.atom_header_index..],
        )?;

        // SAFETY: We already initialized that part of the buffer
        let header = unsafe { space.assume_init_slice_mut() }
            .get_mut(0)
            .expect("Unable to locate Atom Header. This is a bug due to an incorrect Allocator implementation");

        // SAFETY: We just allocated `size` additional bytes for the body, we know they are properly allocated
        unsafe { header.set_size_of_body(header.size_of_body() + size) };

        Ok(alloc)
    }

    #[inline]
    unsafe fn rewind(&mut self, byte_count: usize) -> Result<(), AtomWriteError> {
        self.parent.rewind(byte_count)?;
        let header = self.atom_header_mut();

        // SAFETY: Reducing the size of the atom is fine if rewind was successful
        header.set_size_of_body(header.size_of_body() - byte_count);

        Ok(())
    }

    #[inline]
    fn allocated_bytes(&self) -> &[u8] {
        self.parent.allocated_bytes()
    }

    #[inline]
    unsafe fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        self.parent.allocated_bytes_mut()
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.parent.remaining_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::atom_prelude::*;
    use core::mem::size_of;
    use urid::URID;

    #[test]
    fn test_padding_inside_frame() {
        let mut space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = space.as_bytes_mut();

        // writing
        {
            let mut root = SpaceCursor::new(raw_space);
            let mut frame = AtomWriter::write_new(&mut root, URID::new(1).unwrap()).unwrap();
            frame.write_value(42u32).unwrap();
            frame.write_value(17u32).unwrap();
        }

        // checking
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, 1);
            assert_eq!(atom.size as usize, 8);

            let (value, space) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 42);

            let (value, _) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 17);
        }
    }
}
