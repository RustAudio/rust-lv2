use crate::option::subject::Subject;
use lv2_atom::prelude::AlignedSpace;
use lv2_atom::Atom;
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
    pub fn data(&self) -> Option<&AlignedSpace<u8>> {
        // SAFETY: lifetime of the returned atom value is guaranteed by lifetime of the current Option pointer
        // And the validity of these pointers are guaranteed by the host
        let slice = unsafe {
            std::slice::from_raw_parts(
                self.inner.value.cast::<u8>().as_ref()?,
                self.inner.size as usize,
            )
        };

        Some(AlignedSpace::from_slice(slice))
    }

    #[inline]
    pub fn read<T: crate::option::OptionType>(
        &self,
        option_type: URID<T>,
        data_type: URID<T::AtomType>,
    ) -> Option<T> {
        if !self.is(option_type) {
            return None;
        }

        if self.inner.type_ != data_type {
            return None;
        }

        // SAFETY: data is guaranteed to be an atom by the host
        let atom = unsafe { self.data()?.read().next_atom()? }.read(data_type)?;

        T::from_option_value(atom)
    }
}
