use lv2_core::extension::ExtensionDescriptor;
use lv2_core::feature::*;
use lv2_core::plugin::Plugin;
use lv2_core::prelude::*;
use lv2_sys;
use std::marker::PhantomData;
use std::os::raw::*; //get all common c_type

/// Trait to provide worker extension to LV2 plugins.
///
/// The worker extension allows plugins to schedule work that must be performed in another thread.
/// Plugins can use this interface to safely perform work that is not real-time safe, and receive
/// the result in the run context. The details of threading are managed by the host, allowing
/// plugins to be simple and portable while using resources more efficiently.

/// Host feature allowing plugins to schedule work that must be performed in another thread.
/// Plugins can use this interface to safely perform work that is not real-time safe, and receive
/// the result in the run context.
//Marker feature to signal that the plugin use the worker:schedule feature.
#[repr(transparent)]
pub struct Schedule<'a> {
    pub internal: &'a lv2_sys::LV2_Worker_Schedule,
}

//lying on sync and send
unsafe impl Sync for Schedule<'_> {}
unsafe impl Send for Schedule<'_> {}

unsafe impl<'a> UriBound for Schedule<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_WORKER__schedule;
}

unsafe impl<'a> Feature for Schedule<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void) -> Option<Self> {
        (feature as *const lv2_sys::LV2_Worker_Schedule)
            .as_ref()
            .map(|internal| Self { internal })
    }
}

/// Errors potentially generated by worker methods
pub enum WorkerError {
    /// Unknown or general error
    Unknown,
    /// Failure due to a lack of space
    NoSpace,
}

// Worker Traits
pub trait Worker: Plugin {
    /// The work to do in a non-real-time thread. The spec require plugins to implment this method.
    ///
    /// This is called by the host in a non-realtime context as requested, possibly with an
    /// arbitrary message to handle.
    ///
    /// A response can be sent to run() using respond. The plugin MUST NOT make any assumptions
    /// about which thread calls this method, except that there are no real-time requirements and
    /// only one call may be executed at a time. That is, the host MAY call this method from any
    /// non-real-time thread, but MUST NOT make concurrent calls to this method from several
    /// threads.
    fn work(
        &mut self,
        response_function: lv2_sys::LV2_Worker_Respond_Function,
        respond_handle: lv2_sys::LV2_Worker_Respond_Handle,
        size: u32,
        data: *const c_void,
    ) -> Result<(), WorkerError>;

    /// Handle a response from the worker. The spec require plugins to implement this method even if
    /// many host support to not have it.
    ///
    /// This is called by the host in the run() context when a response from the worker is ready.
    fn work_response(&mut self, size: u32, body: *const c_void) -> Result<(), WorkerError>;

    ///Called when all responses for this cycle have been delivered. (optional)
    ///
    ///Since work_response() may be called after run() finished, this provides a hook for code that
    ///must run after the cycle is completed.
    fn end_run(&mut self) -> Result<(), WorkerError> {
        Ok(())
    }
}

// A descriptor for the plugin. This is just a marker type to associate constants and methods with.
pub struct WorkerDescriptor<P: Worker> {
    plugin: PhantomData<P>,
}

unsafe impl<P: Worker> UriBound for WorkerDescriptor<P> {
    const URI: &'static [u8] = lv2_sys::LV2_WORKER__interface;
}

impl<P: Worker> WorkerDescriptor<P> {
    /// Extern unsafe version of `work` method actually called by the host
    unsafe extern "C" fn extern_work(
        handle: lv2_sys::LV2_Handle,
        response_function: lv2_sys::LV2_Worker_Respond_Function,
        respond_handle: lv2_sys::LV2_Worker_Respond_Handle,
        size: u32,
        data: *const c_void,
    ) -> lv2_sys::LV2_Worker_Status {
        let plugin = (handle as *mut P).as_mut().unwrap();
        match plugin.work(response_function, respond_handle, size, data) {
            Ok(()) => lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS,
            Err(WorkerError::Unknown) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
            Err(WorkerError::NoSpace) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
        }
    }

    /// Extern unsafe version of `work_response` method actually called by the host
    unsafe extern "C" fn extern_work_response(
        handle: lv2_sys::LV2_Handle,
        size: u32,
        body: *const c_void,
    ) -> lv2_sys::LV2_Worker_Status {
        let plugin = (handle as *mut P).as_mut().unwrap();
        match plugin.work_response(size, body) {
            Ok(()) => lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS,
            Err(WorkerError::Unknown) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
            Err(WorkerError::NoSpace) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
        }
    }

    /// Extern unsafe version of `end_run` method actually called by the host
    // This throw a warning if it's not in `INTERFACE`
    unsafe extern "C" fn extern_end_run(handle: lv2_sys::LV2_Handle) -> lv2_sys::LV2_Worker_Status {
        let plugin = (handle as *mut P).as_mut().unwrap();
        match plugin.end_run() {
            Ok(()) => lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS,
            Err(WorkerError::Unknown) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
            Err(WorkerError::NoSpace) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
        }
    }
}

// Implementing the trait that contains the interface.
impl<P: Worker> ExtensionDescriptor for WorkerDescriptor<P> {
    type ExtensionInterface = lv2_sys::LV2_Worker_Interface;

    const INTERFACE: &'static lv2_sys::LV2_Worker_Interface = &lv2_sys::LV2_Worker_Interface {
        work: Some(Self::extern_work),
        work_response: Some(Self::extern_work_response),
        //i want to have `None` here when the plugin doesn't implements the `end_run` trait method
        end_run: Some(Self::extern_end_run),
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
