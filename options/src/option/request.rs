use urid::URID;
use lv2_atom::{Atom, BackAsSpace};
use crate::{Subject, OptionType, OptionsError};

#[repr(C)]
// SAFETY: The last fields are left uninitialized by the host, it's up to the plugin to set them
pub struct OptionRequest {
    inner: lv2_sys::LV2_Options_Option
}

impl OptionRequest {
    #[inline]
    pub(crate)fn from_mut(option: &mut lv2_sys::LV2_Options_Option) -> &mut Self {
        // SAFETY: lv2_sys::LV2_Options_Option and OptionRequest have the same memory layout
        unsafe {&mut *(option as *mut lv2_sys::LV2_Options_Option as *mut Self) }
    }

    #[inline]
    pub fn subject(&self) -> core::option::Option<Subject> {
        Subject::from_raw(self.inner.context, self.inner.subject)
    }

    #[inline]
    pub fn option_type(&self) -> Option<URID> {
        URID::new(self.inner.key)
    }

    #[inline]
    pub fn is<T>(&self, urid: URID<T>) -> bool {
        self.inner.key == urid
    }

    #[inline]
    pub fn has_response(&self) -> bool { self.inner.type_ != 0 }

    #[inline]
    pub fn try_respond<'a, T: OptionType>(&'a mut self,
                                            option_type: URID<T>,
                                            atom_type: URID<T::AtomType>,
                                            value: &'a T
    )  -> Result<(), OptionsError> where T::AtomType: BackAsSpace<'a> {
        if !self.is(option_type) {
            return Err(OptionsError::BadKey);
        }

        unsafe { self.set_value_unchecked(atom_type, value.as_option_value()) };

        Ok(())
    }

    #[inline]
    unsafe fn set_value_unchecked<'a, T: Atom<'a, 'a>>(
        &'a mut self,
        value_type: URID<T>,
        value_handle: T::ReadHandle,
    ) where
        T: BackAsSpace<'a>,
    {
        let data = T::back_as_space(value_handle);
        self.inner.type_ = value_type.get();
        self.inner.size = data.len() as u32;
        self.inner.value = data.data().unwrap().as_ptr().cast();
    }
}

pub struct OptionRequestList<'a> {
    ptr: &'a mut lv2_sys::LV2_Options_Option
}

impl<'a> OptionRequestList<'a> {
    pub fn iter_mut<'list: 'a>(&'list mut self) -> OptionRequestListIter<'a> {
        OptionRequestListIter { current: self.ptr }
    }
}

impl<'a> OptionRequestList<'a> {
    /// SAFETY: Caller must ensure pointer actually points to the start of a zero-terminated list.
    #[inline]
    pub(crate) unsafe fn from_mut(ptr: &'a mut lv2_sys::LV2_Options_Option) -> Self {
        Self { ptr }
    }
}

impl<'a, 'list: 'a> IntoIterator for &'list mut OptionRequestList<'a >{
    type Item = &'a mut OptionRequest;
    type IntoIter = OptionRequestListIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct OptionRequestListIter<'a> {
    current: &'a mut lv2_sys::LV2_Options_Option
}

impl<'a> Iterator for OptionRequestListIter<'a> {
    type Item = &'a mut OptionRequest;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.key == 0 {
            return None;
        }

        // SAFETY: list is guaranteed by the host to end with a zeroed struct.
        // Therefore the pointer always points to a valid lv2_sys::LV2_Options_Option value
        // (only its contents may be incorrect if zeroed)
        let next = unsafe {
            &mut  *((self.current as *mut lv2_sys::LV2_Options_Option).add(1))
        };

        let item = core::mem::replace(&mut self.current, next);

        Some(OptionRequest::from_mut(item))
    }
}