use crate::space::AllocateSpace;
use urid::URID;
use std::mem::transmute;
use crate::header::AtomHeader;

/// A `MutSpace` that tracks the amount of allocated space in an atom header.
pub struct AtomSpaceWriter<'a> {
    atom: &'a mut AtomHeader,
    parent: &'a mut dyn AllocateSpace<'a>,
}

impl<'a> AtomSpaceWriter<'a> {
    #[inline]
    pub fn atom(&'a self) -> &'a AtomHeader {
        self.atom
    }

    /// Create a new framed space with the given parent and type URID.
    pub fn write_new<'space: 'a, A: ?Sized>(parent: &'a mut impl AllocateSpace<'space>, urid: URID<A>) -> Option<Self> {
        let atom = AtomHeader::from_raw(sys::LV2_Atom {
            size: 0,
            type_: urid.get(),
        });

        let atom = crate::space::write_value(parent, atom)?;
        Some(Self::new(atom, parent))
    }

    #[inline]
    fn new<'space: 'a>(atom: &'a mut AtomHeader, parent: &'a mut dyn AllocateSpace<'space>) -> Self {
        // SAFETY: Here we reduce the lifetime 'space to 'a, which is safe because 'space: 'a, and also because this reference will never be changed from now on.
        let parent: &'a mut dyn AllocateSpace<'a> = unsafe { transmute(parent) };
        Self { atom, parent }
    }
}

impl<'a> AllocateSpace<'a> for AtomSpaceWriter<'a> {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        let result = self.parent.allocate_unaligned(size);
        if result.is_some() {
            self.atom.as_raw_mut().size += size as u32;
        }

        result
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.parent.as_bytes()
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.parent.as_bytes_mut()
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;
    use crate::prelude::AtomSpaceWriter;
    use urid::URID;

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
            let mut root: &mut _ = raw_space;
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