use crate::option::subject::Subject;
use crate::OptionsError;
use lv2_atom::space::AtomSpace;
use lv2_atom::{Atom, AtomHandle};
use urid::URID;

#[repr(C)]
pub struct OptionValue {
    inner: lv2_sys::LV2_Options_Option,
}

impl OptionValue {
    #[inline]
    pub(crate) fn from_ref(raw: &lv2_sys::LV2_Options_Option) -> &Self {
        // SAFETY: lv2_sys::LV2_Options_Option and OptionValue are guaranteed to have the same memory layout.
        unsafe { &*(raw as *const lv2_sys::LV2_Options_Option).cast() }
    }

    #[inline]
    pub fn subject(&self) -> core::option::Option<Subject> {
        Subject::from_raw(self.inner.context, self.inner.subject)
    }

    #[inline]
    pub fn is<T>(&self, urid: URID<T>) -> bool {
        self.inner.key == urid
    }

    #[inline]
    pub fn data(&self) -> Option<&[u8]> {
        // SAFETY: lifetime of the returned atom value is guaranteed by lifetime of the current Option pointer
        // And the validity of these pointers are guaranteed by the host
        let slice = unsafe {
            std::slice::from_raw_parts(
                self.inner.value.cast::<u8>().as_ref()?,
                self.inner.size as usize,
            )
        };

        Some(slice)
    }

    #[inline]
    pub fn read<T: crate::option::OptionType>(
        &self,
        option_type: URID<T>,
        data_type: URID<T::AtomType>,
    ) -> Result<T, OptionsError> {
        if !self.is(option_type) {
            return Err(OptionsError::BadKey);
        }

        if self.inner.type_ != data_type {
            return Err(OptionsError::BadValue);
        }

        // SAFETY: data is guaranteed to be an atom by the host, and atom type is checked above
        let atom = unsafe { self.atom_value::<T::AtomType>() }.ok_or(OptionsError::BadValue)?;

        T::from_option_value(atom).ok_or(OptionsError::BadValue)
    }

    unsafe fn atom_value<T: Atom>(
        &self,
    ) -> Option<<<T as Atom>::ReadHandle as AtomHandle>::Handle> {
        // TODO: Atoms can actually be from non-aligned spaces
        T::read(AtomSpace::from_bytes_unchecked(self.data()?)).ok()
    }
}
