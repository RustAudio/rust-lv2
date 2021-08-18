use crate::header::AtomHeader;
use crate::space::{Space, SpaceAllocator};
use urid::URID;

/// A `MutSpace` that tracks the amount of allocated space in an atom header.
pub struct AtomSpaceWriter<'handle, 'space: 'handle> {
    atom_header_index: usize,
    parent: &'handle mut dyn SpaceAllocator<'space>,
}

impl<'handle, 'space> AtomSpaceWriter<'handle, 'space> {
    #[inline]
    pub fn atom_header(&self) -> AtomHeader {
        let previous = self
            .parent
            .allocated_bytes()
            .get(self.atom_header_index..)
            .unwrap();
        let space = Space::try_from_bytes(previous).unwrap();

        unsafe { *space.assume_init_value().unwrap() }
    }

    fn atom_header_mut(&mut self) -> &mut AtomHeader {
        let previous = self
            .parent
            .allocated_bytes_mut()
            .get_mut(self.atom_header_index..)
            .unwrap();
        let space = Space::<AtomHeader>::try_from_bytes_mut(previous).unwrap();

        unsafe { space.assume_init_value_mut().unwrap() }
    }

    /// Create a new framed space with the given parent and type URID.
    pub fn write_new<A: ?Sized>(
        parent: &'handle mut impl SpaceAllocator<'space>,
        urid: URID<A>,
    ) -> Option<Self> {
        let atom = AtomHeader::new(urid);

        let atom_header_index = parent.allocated_bytes().len();
        crate::space::write_value(parent, atom)?;
        Some(Self {
            atom_header_index,
            parent,
        })
    }
}

impl<'handle, 'space: 'handle> SpaceAllocator<'space> for AtomSpaceWriter<'handle, 'space> {
    #[inline]
    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])> {
        let (previous, current) = self.parent.allocate_and_split(size)?;

        let space =
            Space::<AtomHeader>::try_from_bytes_mut(previous.get_mut(self.atom_header_index..)?)?;
        let header = unsafe { space.assume_init_value_mut() }?;

        // SAFETY: We just allocated `size` additional bytes for the body, we know they are properly allocated
        unsafe { header.set_size_of_body(header.size_of_body() + size) };

        Some((previous, current))
    }

    #[inline]
    unsafe fn rewind(&mut self, byte_count: usize) -> bool {
        let rewound = self.parent.rewind(byte_count);
        let header = self.atom_header_mut();

        if rewound {
            // SAFETY: Reducing the size of the atom is fine
            header.set_size_of_body(header.size_of_body() - byte_count);
        }

        rewound
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
    use crate::prelude::AtomSpaceWriter;
    use crate::space::cursor::SpaceCursor;
    use crate::space::AtomSpace;
    use core::mem::size_of;
    use urid::URID;

    #[test]
    fn test_padding_inside_frame() {
        const MEMORY_SIZE: usize = 256;
        let mut space = AtomSpace::boxed(256);
        let raw_space = space.as_bytes_mut();

        // writing
        {
            let mut root = SpaceCursor::new(raw_space);
            let mut frame = AtomSpaceWriter::write_new(&mut root, URID::new(1).unwrap()).unwrap();
            crate::space::write_value(&mut frame, 42u32).unwrap();
            crate::space::write_value(&mut frame, 17u32).unwrap();
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
