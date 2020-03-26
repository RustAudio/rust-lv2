use lv2_core::feature::*;
//use std::ffi::CString;
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

/// Marker for URID representing the nature of a log message
// Note : it's may be better to have a URID trait to define a common interface
pub unsafe trait EntryType {
    fn get(self) -> u32;
}

unsafe impl EntryType for URID<ErrorClass>{ fn get(self) -> u32 {URID::<ErrorClass>::get(self)}}
unsafe impl EntryType for URID<NoteClass>{ fn get(self) -> u32 {URID::<NoteClass>::get(self)}}
unsafe impl EntryType for URID<TraceClass>{ fn get(self) -> u32 {URID::<TraceClass>::get(self)}}
unsafe impl EntryType for URID<WarningClass>{ fn get(self) -> u32 {URID::<WarningClass>::get(self)}}

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

impl<'a> Log<'a> {
    pub fn print(&self, entry_type: impl EntryType, message: &[u8]) -> Result<(),()> {
        let printf = if let Some(printf) = self.internal.printf {
            printf
        } else {
            return Err(());
        };
        let res = unsafe { (printf)(self.internal.handle, entry_type.get(), message as *const _ as *const c_char )};
        if res > 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// A URID cache containing all time properties.
#[derive(URIDCollection,Debug)]
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
