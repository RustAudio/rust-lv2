use crate::{OptionType, OptionValue, OptionsError, Subject};
use lv2_atom::{Atom, AtomAsBytes, AtomHandle};
use std::marker::PhantomData;
use urid::URID;

#[repr(C)]
// SAFETY: The last fields are left uninitialized by the host, it's up to the plugin to set them
pub struct OptionRequest<'a> {
    inner: lv2_sys::LV2_Options_Option,
    lifetime: PhantomData<&'a OptionValue>,
}

impl<'a> OptionRequest<'a> {
    #[inline]
    pub(crate) fn from_mut(option: &mut lv2_sys::LV2_Options_Option) -> &mut Self {
        // SAFETY: lv2_sys::LV2_Options_Option and OptionRequest have the same memory layout
        unsafe { &mut *(option as *mut lv2_sys::LV2_Options_Option as *mut Self) }
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
    pub fn has_response(&self) -> bool {
        self.inner.type_ != 0
    }

    #[inline]
    pub fn try_respond<T: OptionType>(
        &mut self,
        option_type: URID<T>,
        atom_type: URID<T::AtomType>,
        value: &'a T,
    ) -> Result<(), OptionsError>
    where
        T::AtomType: AtomAsBytes,
    {
        if !self.is(option_type) {
            return Err(OptionsError::BadKey);
        }

        unsafe { self.set_value_unchecked(atom_type, value.as_option_value()) };

        Ok(())
    }

    #[inline]
    unsafe fn set_value_unchecked<T: Atom>(
        &mut self,
        value_type: URID<T>,
        value_handle: <<T as Atom>::ReadHandle as AtomHandle>::Handle,
    ) where
        T: AtomAsBytes,
    {
        let data = T::read_as_bytes(value_handle);
        self.inner.type_ = value_type.get();
        self.inner.size = data.len() as u32;
        self.inner.value = data.as_ptr().cast();
    }
}

pub struct OptionRequestList<'a> {
    ptr: &'a mut lv2_sys::LV2_Options_Option,
}

impl<'a> OptionRequestList<'a> {
    #[inline]
    pub fn iter_mut<'list>(&'list mut self) -> OptionRequestListIter<'a, 'list>
    where
        'a: 'list,
    {
        OptionRequestListIter {
            current: self.ptr,
            value_lifetime: PhantomData,
        }
    }
}

impl<'a> OptionRequestList<'a> {
    /// SAFETY: Caller must ensure pointer actually points to the start of a zero-terminated list.
    #[inline]
    pub(crate) unsafe fn from_mut(ptr: &'a mut lv2_sys::LV2_Options_Option) -> Self {
        Self { ptr }
    }
}

impl<'value: 'list, 'list> IntoIterator for &'list mut OptionRequestList<'value> {
    type Item = &'list mut OptionRequest<'value>;
    type IntoIter = OptionRequestListIter<'value, 'list>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct OptionRequestListIter<'value: 'list, 'list> {
    current: &'list mut lv2_sys::LV2_Options_Option,
    value_lifetime: PhantomData<&'value OptionValue>,
}

impl<'value: 'list, 'list> Iterator for OptionRequestListIter<'value, 'list> {
    type Item = &'list mut OptionRequest<'value>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.key == 0 {
            return None;
        }

        // SAFETY: list is guaranteed by the host to end with a zeroed struct.
        // Therefore the pointer always points to a valid lv2_sys::LV2_Options_Option value
        // (only its contents may be incorrect if zeroed)
        let next = unsafe { &mut *((self.current as *mut lv2_sys::LV2_Options_Option).add(1)) };

        let item = core::mem::replace(&mut self.current, next);

        Some(OptionRequest::from_mut(item))
    }
}
