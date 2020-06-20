use std::collections::HashMap;
use std::slice;

use core::feature::Feature;
use core::prelude::*;
use std::ffi::c_void;
use urid::*;

/// Host feature to communicate options to the plugin
///
/// https://lv2plug.in/ns/ext/options
///
/// Add an `LV2Options` field to your plugin's `Feature` struct. Then use the `::retrieve_option()` method to get the information about the option.
pub struct LV2Options {
    slice_map: HashMap<u32, (usize, usize)>,
    data: Vec<u8>,
}

unsafe impl UriBound for LV2Options {
    const URI: &'static [u8] = sys::LV2_OPTIONS__options;
}

unsafe impl Feature for LV2Options {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        if class != ThreadingClass::Audio {
            Self::new(feature as *const sys::LV2_Options_Option)
        } else {
            panic!("The URID mapping feature isn't allowed in the audio threading class");
        }
    }
}

impl LV2Options {
    unsafe fn new(options: *const sys::LV2_Options_Option) -> Option<Self> {
        let mut ptr = options;
        let mut data = Vec::new();
        let mut slice_map = HashMap::new();
        while (*ptr).key != 0 {
            let start = data.len();
            data.extend_from_slice(slice::from_raw_parts(
                &(*ptr).size as *const u32 as *const u8,
                std::mem::size_of::<u32>(),
            ));
            data.extend_from_slice(slice::from_raw_parts(
                &(*ptr).type_ as *const u32 as *const u8,
                std::mem::size_of::<u32>(),
            ));
            data.extend_from_slice(slice::from_raw_parts(
                (*ptr).value as *const u8,
                (*ptr).size as usize,
            ));
            slice_map.insert((*ptr).key, (start, data.len()));

            ptr = ptr.offset(1);
        }

        Some(Self { slice_map, data })
    }

    /// Tries to retrieve the option specified by `urid`.
    ///
    /// Returns an `lv2_atom::UnidentifiedAtom` from which the option can be read using the `::read()` method.
    pub fn retrieve_option<'a, T>(&'a self, urid: URID<T>) -> Option<atom::UnidentifiedAtom<'a>> {
        self.slice_map.get(&urid.get()).and_then(|(start, end)| {
            let space = atom::space::Space::from_slice(&self.data[*start..*end]);
            Some(atom::UnidentifiedAtom::new(space))
        })
    }
}
