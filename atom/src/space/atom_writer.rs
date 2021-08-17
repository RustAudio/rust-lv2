use crate::space::SpaceAllocator;
use urid::URID;
use crate::header::AtomHeader;

/// A `MutSpace` that tracks the amount of allocated space in an atom header.
pub struct AtomSpaceWriter<'handle, 'space: 'handle> {
    atom_header_index: usize,
    parent: &'handle mut dyn SpaceAllocator<'space>,
}

impl<'handle, 'space> AtomSpaceWriter<'handle, 'space> {
    #[inline]
    pub fn atom_header(&self) -> AtomHeader {
        todo!()
    }

    #[inline]
    fn atom_header_mut(&self) -> &mut AtomHeader { todo!() }

    /// Create a new framed space with the given parent and type URID.
    pub fn write_new<A: ?Sized>(parent: &'handle mut impl SpaceAllocator<'space>, urid: URID<A>) -> Option<Self> {
        let atom_header_index = parent.allocated_bytes().len();
        let atom = AtomHeader::from_raw(sys::LV2_Atom {
            size: 0,
            type_: urid.get(),
        });

        let atom = crate::space::write_value(parent, atom)?;
        Some(Self { atom_header_index, parent })
    }
}

impl<'handle, 'space: 'handle> SpaceAllocator<'space> for AtomSpaceWriter<'handle, 'space> {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&mut [u8]> {
        let result = self.parent.allocate_unaligned(size);
        if result.is_some() {
            // TODO
            // self.atom_header_mut().as_raw_mut().size += size as u32;
        }

        result
    }

    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])> {
        todo!()
    }

    fn allocated_bytes(&self) -> &[u8] {
        todo!()
    }

    fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        todo!()
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
    use core::mem::size_of;
    use crate::prelude::AtomSpaceWriter;
    use urid::URID;
    use crate::space::cursor::SpaceCursor;

    #[test]
    fn test_padding_inside_frame() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let raw_space: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        };

        // writing
        {
            let mut root = SpaceCursor::new(raw_space);
            let mut frame =
                AtomSpaceWriter::write_new(&mut root, URID::<()>::new(1).unwrap())
                    .unwrap();
            crate::space::write_value(&mut frame, 42u32).unwrap();
            crate::space::write_value(&mut frame, 17u32).unwrap();
        }

        // checking
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, 1);
            assert_eq!(atom.size as usize, 12);

            let (value, space) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 42);
            let (_, space) = space.split_at(4);

            let (value, _) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 17);
        }
    }
}