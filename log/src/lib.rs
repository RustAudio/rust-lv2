use lv2_core::feature::{Feature, ThreadingClass};
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::os::raw::*; //get all common c_type
use urid::*;

/// URID marker. The corresponding URID is used to log an error on the host.
pub struct Error;
unsafe impl UriBound for Error {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Error;
}

/// URID marker. The corresponding URID is used to log an informative on the host.
pub struct Note;
unsafe impl UriBound for Note {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Note;
}

/// URID marker. The corresponding URID is used to log a debbuging trace on the host.
pub struct Trace;
unsafe impl UriBound for Trace {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Trace;
}

/// URID marker. The corresponding URID is used to log a warning on the host.
pub struct Warning;
unsafe impl UriBound for Warning {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Warning;
}

/// Trait for URID marker. Plugin implementers shouldn't care about it.
///
/// # Safety
///
/// This trait is used to check at compile time if an URID indicate the nature of a log message,
/// Plugin implementers should not implement it.
pub unsafe trait Entry {}

unsafe impl Entry for Error {}
unsafe impl Entry for Note {}
unsafe impl Entry for Trace {}
unsafe impl Entry for Warning {}

/// Returned if an error occured when sending a log to the host.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PrintError;

impl fmt::Display for PrintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occured when sending a message to the host")
    }
}
impl error::Error for PrintError {}

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
    //placeholder, not useable yet
    vprintf: unsafe extern "C" fn(
        handle: lv2_sys::LV2_Log_Handle,
        type_: lv2_sys::LV2_URID,
        fmt: *const c_char,
        ap: *mut core::ffi::c_void, //should be *mut ffi::VaList but it's not yet stable
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
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        (feature as *const LogInternal)
            .as_ref()
            .map(|internal| Self { internal })
    }
}

impl<'a> Log<'a> {
    /// Send a log message to the host.
    ///
    /// The `entry` parameter is an URID representing the kind of log message. There are four
    /// kind of message:
    /// * **note:** an informative message.
    /// * **warning:** a warning message.
    /// * **error:** an error message.
    /// * **trace:** a debugging trace. These entries should not be displayed during normal
    /// operation, but the host may implement an option to display them for debugging purposes.
    /// This entry type is special in that it may be written to in a real-time thread. It is
    /// assumed that if debug tracing is enabled, real-time considerations are not a concern.
    pub fn print_cstr(&self, entry: URID<impl Entry>, message: &CStr) -> Result<(), PrintError> {
        let res = unsafe {
            (self.internal.printf)(
                self.internal.handle,
                entry.get(),
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

/// A URID cache containing all usefull log properties.
#[derive(URIDCollection, Debug)]
pub struct LogURIDCollection {
    pub error: URID<Error>,
    pub note: URID<Note>,
    pub trace: URID<Trace>,
    pub warning: URID<Warning>,
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
        let urid = unsafe { URID::<Error>::new_unchecked(42) };
        let message = CStr::from_bytes_with_nul(b"message\0").unwrap();

        let _ = fake_logger.print_cstr(urid, message);
    }

    #[test]
    fn it_works() {}
}
