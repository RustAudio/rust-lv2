use lv2_core::feature::{Feature, ThreadingClass};
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::os::raw::*; //get all common c_type
use urid::*;

pub struct EntryClass;
unsafe impl UriBound for EntryClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Entry;
}

/// UriBound for an error message.
pub struct ErrorClass;
unsafe impl UriBound for ErrorClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Error;
}

/// UriBound for an informative message.
pub struct NoteClass;
unsafe impl UriBound for NoteClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Note;
}

/// UriBound for a debuging message.
pub struct TraceClass;
unsafe impl UriBound for TraceClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Trace;
}

/// Uribound for an error message.
pub struct WarningClass;
unsafe impl UriBound for WarningClass {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Warning;
}

/// Marker trait for Uribound type representing nature of a log message.
pub unsafe trait EntryType {}

unsafe impl EntryType for ErrorClass {}
unsafe impl EntryType for NoteClass {}
unsafe impl EntryType for TraceClass {}
unsafe impl EntryType for WarningClass {}

/// Returned if an error occured when sending a log to the host.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PrintError;

impl fmt::Display for PrintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occured when sending a message to the host")
    }
}
impl Error for PrintError {}

//replace the lv2_sys::Log_Log to avoid checking if function pointers are null
#[repr(C)]
struct LogInternal {
    handle: lv2_sys::LV2_Log_Handle,
    printf: unsafe extern "C" fn(
        handle: lv2_sys::LV2_Log_Handle,
        type_: lv2_sys::LV2_URID,
        fmt: *const c_char,
        ...
    ) -> c_int,
    vprintf: unsafe extern "C" fn(
        handle: lv2_sys::LV2_Log_Handle,
        type_: lv2_sys::LV2_URID,
        fmt: *const c_char,
        ap: *mut lv2_sys::__va_list_tag,
    ) -> c_int,
}

/// The Log feature.
#[repr(transparent)]
pub struct Log<'a> {
    internal: &'a LogInternal,
}

unsafe impl<'a> UriBound for Log<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__log;
}

unsafe impl<'a> Feature for Log<'a> {
    // Note: this feature can be used in any threading class but:
    // * i have a doubt about it's thread safety, can we assume the host provide this thread safety
    // since this feature can be used anywhere ?.
    // * i shouldn't be used in context where rt is a concern, that mean is audiothreadclass in
    // practice, but it's acceptable to use it for debugging purpose
    // So, at this time, i just giving access to it instanciation class
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        match class {
            ThreadingClass::Audio => {
                panic!("The log feature is not allowed in the audio threading class")
            }
            _ => (feature as *const LogInternal)
                .as_ref()
                .map(|internal| Self { internal }),
        }
    }
}

impl<'a> Log<'a> {
    /// Send a log message to the host.
    ///
    /// The `entry_type` parameter is an URID representing the kind of log message. There are four
    /// kind of message:
    /// * **note:** an informative message.
    /// * **warning:** a warning message.
    /// * **error:** an error message.
    /// * **trace:** a debugging trace. These entries should not be displayed during normal
    /// operation, but the host may implement an option to display them for debugging purposes.
    /// This entry type is special in that it may be written to in a real-time thread. It is
    /// assumed that if debug tracing is enabled, real-time considerations are not a concern.
    pub fn print_cstr(
        &self,
        entry_type: URID<impl EntryType>,
        message: &CStr,
    ) -> Result<(), PrintError> {
        let res = unsafe {
            (self.internal.printf)(
                self.internal.handle,
                entry_type.get(),
                "%s\0" as *const _ as *const c_char,
                message.as_ptr(),
            )
        };
        if res > 0 {
            Ok(())
        } else {
            Err(PrintError)
        }
    }
}

/// A URID cache containing all log properties.
#[derive(URIDCollection, Debug)]
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
    use super::*;
    fn _should_compile() {
        let fake_logger = unsafe {
            Log {
                internal: &*(0xDEFEC8 as *const LogInternal),
            }
        };
        let urid = unsafe { URID::<ErrorClass>::new_unchecked(42) };
        let message = CStr::from_bytes_with_nul(b"message\0").unwrap();

        let _ = fake_logger.print_cstr(urid, message);
    }

    #[test]
    fn it_works() {}
}
