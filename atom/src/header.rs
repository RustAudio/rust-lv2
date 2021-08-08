#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct AtomHeader {
    inner: lv2_sys::LV2_Atom
}

impl AtomHeader {
    #[inline]
    pub fn from_raw(inner: lv2_sys::LV2_Atom) -> Self {
        Self { inner }
    }

    #[inline]
    pub(crate) fn as_raw_mut(&mut self) -> &mut lv2_sys::LV2_Atom {
        // SAFETY: AtomHeader is repr(C) and has LV2_Atom as its only field, so transmuting between the two is safe.
        unsafe { &mut *(self as *mut Self as *mut _) }
    }

    #[inline]
    pub fn size(self) -> usize {
        self.inner.size as usize
    }

    #[inline]
    pub fn urid(self) -> u32 {
        self.inner.type_
    }
}

