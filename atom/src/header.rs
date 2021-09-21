use crate::space::error::AtomReadError;
use crate::{Atom, UnidentifiedAtom};
use urid::URID;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct AtomHeader {
    inner: lv2_sys::LV2_Atom,
}

impl AtomHeader {
    #[inline]
    pub(crate) fn new<T: ?Sized>(atom_type: URID<T>) -> Self {
        Self {
            inner: lv2_sys::LV2_Atom {
                size: 0,
                type_: atom_type.get(),
            },
        }
    }

    #[inline]
    pub(crate) fn from_raw(raw: &lv2_sys::LV2_Atom) -> &Self {
        // SAFETY: AtomHeader is repr(C) and has LV2_Atom as its only field, so transmuting between the two is safe.
        unsafe { &*(raw as *const lv2_sys::LV2_Atom as *const _) }
    }

    #[inline]
    pub(crate) fn from_raw_mut(raw: &mut lv2_sys::LV2_Atom) -> &mut Self {
        // SAFETY: AtomHeader is repr(C) and has LV2_Atom as its only field, so transmuting between the two is safe.
        unsafe { &mut *(raw as *mut lv2_sys::LV2_Atom as *mut _) }
    }

    #[inline]
    pub unsafe fn assume_full_atom(&self) -> &UnidentifiedAtom {
        UnidentifiedAtom::from_header(self)
    }

    #[inline]
    pub unsafe fn assume_full_atom_mut(&mut self) -> &mut UnidentifiedAtom {
        UnidentifiedAtom::from_header_mut(self)
    }

    #[inline]
    pub(crate) unsafe fn set_size_of_body(&mut self, size: usize) {
        self.inner.size = size as u32;
    }

    #[inline]
    pub fn size_of_body(self) -> usize {
        self.inner.size as usize
    }

    #[inline]
    pub fn size_of_atom(self) -> usize {
        self.size_of_body() + ::core::mem::size_of::<AtomHeader>()
    }

    #[inline]
    pub fn urid(self) -> URID {
        URID::new(self.inner.type_).unwrap()
    }

    #[inline]
    pub(crate) fn check_urid<A: Atom>(self, other: URID<A>) -> Result<(), AtomReadError> {
        if other == self.urid() {
            Ok(())
        } else {
            Err(AtomReadError::InvalidAtomUrid {
                expected_uri: A::uri(),
                expected_urid: other.into_general(),
                found_urid: self.urid(),
            })
        }
    }
}
