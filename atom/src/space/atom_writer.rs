use crate::header::AtomHeader;
use crate::space::{
    error::AtomWriteError, AtomSpace, SpaceWriter, SpaceWriterImpl, SpaceWriterSplitAllocation,
};
use urid::URID;

/// A `MutSpace` that tracks the amount of allocated space in an atom header.
pub struct AtomSpaceWriter<'a> {
    atom_header_index: usize,
    parent: &'a mut (dyn SpaceWriterImpl),
}

impl<'a> AtomSpaceWriter<'a> {
    #[inline]
    pub fn atom_header(&self) -> AtomHeader {
        let previous = self
            .parent
            .allocated_bytes()
            .get(self.atom_header_index..)
            .unwrap();
        let space = AtomSpace::from_bytes(previous).unwrap();

        unsafe { space.assume_init_slice()[0] }
    }

    #[inline]
    fn atom_header_mut(&mut self) -> &mut AtomHeader {
        let previous = self
            .parent
            .allocated_bytes_mut()
            .get_mut(self.atom_header_index..)
            .unwrap();
        let space = AtomSpace::from_bytes_mut(previous).unwrap();

        unsafe { &mut space.assume_init_slice_mut()[0] }
    }

    /// Create a new framed space with the given parent and type URID.
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

impl<'a> SpaceWriterImpl for AtomSpaceWriter<'a> {
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
    fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        self.parent.allocated_bytes_mut()
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.parent.remaining_bytes()
    }

    #[inline]
    fn remaining_bytes_mut(&mut self) -> &mut [u8] {
        self.parent.remaining_bytes_mut()
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
            let mut frame = AtomSpaceWriter::write_new(&mut root, URID::new(1).unwrap()).unwrap();
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
