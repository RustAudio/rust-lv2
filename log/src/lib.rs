//! Logging library allowing LV2 plugins to print log message through host.
//!
//! This crate allow plugins to print log message through the host. Different log levels are
//! defined by URI and passed as a lv2 URID. The core of this crate is the [Log
//! Feature](struct.Log.html).
//!
//! # Example
//!
//! This following plugin log an error message each time the input control is switched on.
//!
//! Note: Logging an error is not real-time safe. This following plugin should not ask
//! `hardRTCapable` in its .ttl files.
//!
//! ```
//! use core::any::Any;
//! use lv2_core::feature::*;
//! use lv2_core::prelude::*;
//! use lv2_log::*;
//! use lv2_urid::*;
//! use std::ffi::CStr;
//! use urid::*;
//!
//! #[derive(URIDCollection)]
//! struct LoggerUrids {
//!     error: URID<Error>,
//! }
//!
//! #[derive(PortCollection)]
//! struct Ports {
//!     toggle: InputPort<Control>,
//! }
//! #[derive(FeatureCollection)]
//! pub struct InitFeatures<'a> {
//!     map: LV2Map<'a>,
//! }
//!
//! #[derive(FeatureCollection)]
//! pub struct AudioFeatures<'a> {
//!     log: Log<'a>,
//! }
//!
//! #[repr(C)]
//! #[uri("urn:rust-lv2-test:logger")]
//! struct logger {
//!     urids: LoggerUrids,
//!     last_toggle: bool,
//! }
//!
//! impl Plugin for logger {
//!     type Ports = Ports;
//!     type InitFeatures = InitFeatures<'static>;
//!     type AudioFeatures = AudioFeatures<'static>;
//!
//!     fn new(_plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
//!         let urids: LoggerUrids = features.map.populate_collection()?;
//!         Some(Self {
//!             urids,
//!             last_toggle: false,
//!         })
//!     }
//!
//!     fn run(&mut self, ports: &mut Ports, features: &mut Self::AudioFeatures, _: u32) {
//!         let log = &features.log;
//!         let toggle = *ports.toggle > 0.0;
//!         if self.last_toggle != toggle && toggle {
//!             let message = CStr::from_bytes_with_nul(b"error run message\n\0").unwrap();
//!             let _ = log.print_cstr(self.urids.error, message);
//!         }
//!         self.last_toggle = toggle;
//!     }
//!
//!     fn extension_data(_uri: &Uri) -> Option<&'static dyn Any> {
//!         None
//!         //match_extensions![uri, WorkerDescriptor<Self>]
//!     }
//! }
//!
//! lv2_descriptors!(logger);
//! ```

use lv2_core::feature::{Feature, ThreadingClass};
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::os::raw::*; //get all common c_type
use urid::*;

/// URID marker. The corresponding URID is used to log an error message on the host.
pub struct Error;
unsafe impl UriBound for Error {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Error;
}

/// URID marker. The corresponding URID is used to log an informative message on the host.
pub struct Note;
unsafe impl UriBound for Note {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Note;
}

/// URID marker. The corresponding URID is used to log a debugging trace on the host.
pub struct Trace;
unsafe impl UriBound for Trace {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Trace;
}

/// URID marker. The corresponding URID is used to log a warning message on the host.
pub struct Warning;
unsafe impl UriBound for Warning {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__Warning;
}

/// Trait for URID marker. URID with a marker implementing this can be used to indicate the logging
/// level.
///
/// Plugin implementers can implement this on custom URID marker to define and use additional
/// logging level.
//
pub trait Entry: UriBound {}

impl Entry for Error {}
impl Entry for Note {}
impl Entry for Trace {}
impl Entry for Warning {}

/// Returned if an error occurred when sending a log to the host.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PrintError;

impl fmt::Display for PrintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occurred when sending a message to the host")
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

/// Feature allowing to print log message through the host.
///
///
#[repr(transparent)]
pub struct Log<'a> {
    internal: &'a LogInternal,
}

unsafe impl<'a> UriBound for Log<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_LOG__log;
}

unsafe impl<'a> Feature for Log<'a> {
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
    /// standard kind of message:
    /// * **note:** an informative message.
    /// * **warning:** a warning message.
    /// * **error:** an error message.
    /// * **trace:** a debugging trace. These entries should not be displayed during normal
    /// operation, but the host may implement an option to display them for debugging purposes.
    /// This entry type is special in that it may be written to in a real-time thread. It is
    /// assumed that if debug tracing is enabled, real-time considerations are not a concern.
    ///
    /// # Real-Time safety
    ///
    /// This function is not real-time safe. Except for logging debugging trace, it should not be
    /// used in real-time context. That means `HardRTCapable` plugins should not call this function in
    /// their `run()` context.
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

/// A URID cache containing all useful log properties.
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
