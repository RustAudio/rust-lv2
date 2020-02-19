use lv2_core::plugin::Plugin;
use lv2_core::prelude::*;
use lv2_core::feature::*;
use lv2_core::extension::ExtensionDescriptor;
use lv2_sys;
use lv2_sys::{
    LV2_Handle,
    LV2_WORKER__interface,
    LV2_Worker_Interface,
    LV2_Worker_Respond_Function,
    LV2_Worker_Respond_Handle,
    LV2_Worker_Schedule,
    LV2_Worker_Status,
    LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
    LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
    LV2_Worker_Status_LV2_WORKER_SUCCESS,
};
use lv2_urid::prelude::*;
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


//type
pub enum WorkerStatus {
    Success,
    Unknown,
    NoSpace,
}

// Worker Traits
pub trait Worker: Plugin {
    /// the work to do in a non-real-time thread
    fn work(
        &mut self,
        response_function: LV2_Worker_Respond_Function,
        respond_handle: LV2_Worker_Respond_Handle,
        size: u32,
        data: *const c_void,
    ) -> WorkerStatus;

    //fn work_response(
    //    &mut self,
    //    size: u32,
    //    body: *const c_void
    //) -> WorkerStatus;
}

// A descriptor for the plugin. This is just a marker type to associate constants and methods with.
pub struct WorkerDescriptor<P: Worker> {
    plugin: PhantomData<P>,
}

unsafe impl<P: Worker> UriBound for WorkerDescriptor<P> {
    const URI: &'static [u8] = LV2_WORKER__interface;
}

impl<P: Worker> WorkerDescriptor<P> {
    /// The extern, unsafe version of the extending method.
    ///
    /// This is actually called by the host.
    unsafe extern "C" fn extern_work(
        handle: LV2_Handle,
        response_function: LV2_Worker_Respond_Function,
        respond_handle: LV2_Worker_Respond_Handle,
        size: u32,
        data: *const c_void,
    ) -> LV2_Worker_Status {
        let plugin = (handle as *mut P).as_mut().unwrap();
        match plugin.work(response_function, respond_handle, size, data) {
            WorkerStatus::Success => LV2_Worker_Status_LV2_WORKER_SUCCESS,
            WorkerStatus::Unknown => LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
            WorkerStatus::NoSpace => LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
        }
    }

    //unsafe extern "C" fn extern_work_response(
    //    handle: LV2_Handle,
    //    size: u32,
    //    body: *const c_void
    //)-> LV2_Worker_Status {
    //    let plugin = (handle as *mut P).as_mut().unwrap();
    //    match plugin.work_response(size, body) {
    //        WorkerStatus::Success => LV2_Worker_Status_LV2_WORKER_SUCCESS,
    //        WorkerStatus::Unknown => LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
    //        WorkerStatus::NoSpace => LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
    //    }
    //}


}

// Implementing the trait that contains the interface.
impl<P: Worker> ExtensionDescriptor for WorkerDescriptor<P> {
    type ExtensionInterface = LV2_Worker_Interface;

    const INTERFACE: &'static LV2_Worker_Interface = &LV2_Worker_Interface {
        work: Some(Self::extern_work),
        work_response: None,//Some(Self::extern_work_response),
        end_run: None,
    };
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
