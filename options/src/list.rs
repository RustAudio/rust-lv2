use crate::OptionValue;
use core::ffi::c_void;
use lv2_core::feature::{Feature, ThreadingClass};
use urid::UriBound;

/// A read-only list of [`OptionValue`]s, sent from the host.
///
/// This list type doesn't own the contained values: they are simply borrowed from the host.
/// Cloning this struct only clones the list, not its contents.
#[derive(Copy, Clone)]
pub struct OptionsList<'a> {
    options_list: &'a lv2_sys::LV2_Options_Option,
}

impl<'f> OptionsList<'f> {
    /// Returns an iterator over the slice.
    #[inline]
    pub fn iter(&self) -> OptionsListIter<'f> {
        OptionsListIter { current: self.options_list }
    }
}

unsafe impl<'a> UriBound for OptionsList<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_OPTIONS__options;
}

unsafe impl<'a> Feature for OptionsList<'a> {
    #[inline]
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        // SAFETY: Type and contents of feature data pointer is guaranteed by caller
        Some(OptionsList {
            options_list: (feature as *const _ as *mut lv2_sys::LV2_Options_Option).as_ref()?,
        })
    }
}

impl<'a> IntoIterator for &'a OptionsList<'a> {
    type Item = &'a OptionValue;
    type IntoIter = OptionsListIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct OptionsListIter<'a> {
    current: &'a lv2_sys::LV2_Options_Option
}

impl<'a> Iterator for OptionsListIter<'a> {
    type Item = &'a OptionValue;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.current;
        if item.key == 0 {
            return None;
        }

        // SAFETY: list is guaranteed by the host to end with a zeroed struct.
        // Therefore the pointer always points to a valid lv2_sys::LV2_Options_Option value
        // (only its contents may be incorrect if zeroed)
        unsafe {
            self.current = &*((self.current as *const lv2_sys::LV2_Options_Option).add(1));
        }

        Some(OptionValue::from_ref(item))
    }
}
