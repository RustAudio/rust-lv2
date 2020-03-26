use lv2_core::feature::*;
use std::os::raw::*; //get all common c_type
use urid::*;

pub struct EntryClass;
unsafe impl UriBound for EntryClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Entry;
}

pub struct ErrorClass;
unsafe impl UriBound for ErrorClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Error;
}

pub struct NoteClass;
unsafe impl UriBound for NoteClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Note;
}

pub struct TraceClass;
unsafe impl UriBound for TraceClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Trace;
}

pub struct WarningClass;
unsafe impl UriBound for WarningClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Warning;
}

/// The Log feature
#[repr(transparent)]
pub struct Log<'a> {
    internal: &'a lv2_sys::LV2_Log_Log,
}

unsafe impl<'a> UriBound for Log<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__log;
}

unsafe impl<'a> Feature for Log<'a> {
    // Note: this feature can be used in any threading class and doesn't seems to have thready
    // unsafty
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        (feature as *const lv2_sys::LV2_Log_Log)
            .as_ref()
            .map(|internal| Self { internal })
    }
}

/// A URID cache containing all time properties.
#[derive(URIDCollection)]
pub struct LogURIDCollection {
    pub entry_class: URID<EntryClass>,
    pub error_class: URID<ErrorClass>,
    pub note_class: URID<NoteClass>,
    pub trace_class: URID<TraceClass>,
    pub warning_class: URID<WarningClass>,
    //pub log: URID<Log<'a>>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
