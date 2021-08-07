use crate::space::AllocateSpace;
use urid::URID;

/// A `MutSpace` that notes the amount of allocated space in an atom header.
pub struct AtomSpace<'a> {
    atom: &'a mut sys::LV2_Atom,
    parent: &'a mut dyn AllocateSpace<'a>,
}

impl<'a> AtomSpace<'a> {
    #[inline]
    pub fn atom(&self) -> &'a mut sys::LV2_Atom {
        self.atom
    }

    /// Create a new framed space with the given parent and type URID.
    pub fn write_new<A: ?Sized>(parent: &mut dyn AllocateSpace<'a>, urid: URID<A>) -> Option<Self> {
        let atom = sys::LV2_Atom {
            size: 0,
            type_: urid.get(),
        };

        let atom: &'a mut sys::LV2_Atom = crate::space::write_value(parent, atom)?;
        Some(Self { atom, parent })
    }
}

impl<'a, 'b> AllocateSpace<'a> for AtomSpace<'a> {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        let result = self.parent.allocate_unaligned(size);
        if result.is_some() {
            self.atom.size += size as u32;
        }

        result
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.parent.as_bytes()
    }
}