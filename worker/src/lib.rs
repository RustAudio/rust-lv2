use lv2_core::extension::ExtensionDescriptor;
use lv2_core::feature::*;
use lv2_core::plugin::{Plugin, PluginInstance};
use lv2_core::prelude::*;
use lv2_sys;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::*; //get all common c_type
use std::ptr;

/// Host feature to schedule a worker call.
#[repr(transparent)]
pub struct Schedule<'a> {
    internal: &'a lv2_sys::LV2_Worker_Schedule,
}

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

pub struct ScheduleHandler<P> {
    schedule_handle: lv2_sys::LV2_Worker_Schedule_Handle,
    schedule_work: Option<
        unsafe extern "C" fn(
            handle: lv2_sys::LV2_Worker_Schedule_Handle,
            size: u32,
            data: *const c_void,
        ) -> lv2_sys::LV2_Worker_Status,
    >,
    phantom: PhantomData<P>,
}

//declare Schedulehandler Send and Sync is needed to make it usable.
//Send is safe to implement because poited dated should outlive the plugin and are only managed by the host
//I don't think Sync is fully safe to implement, because i don't know what happen if a smart guy is
//multithreading the run function and try to schedule work inside a thread.
unsafe impl<P: Worker> Send for ScheduleHandler<P> {}
unsafe impl<P: Worker> Sync for ScheduleHandler<P> {}

impl<P: Worker> ScheduleHandler<P> {
    /// Request the host to call the worker thread.
    ///
    /// This function should be called from `run()` to request that the host call the `work()`
    /// method in a non-realtime context with the given arguments.
    ///
    /// This function is always safe to call from `run()`, but it is not guaranteed that the worker
    /// is actually called from a different thread. In particular, when free-wheeling (e.g. for
    /// offline rendering), the worker may be executed immediately. This allows single-threaded
    /// processing with sample accuracy and avoids timing problems when `run()` is executing much
    /// faster or slower than real-time.
    ///
    /// Plugins SHOULD be written in such a way that if the worker runs immediately, and responses
    /// from the worker are delivered immediately, the effect of the work takes place immediately
    /// with sample accuracy.
    pub fn schedule_work(&self, worker_data: P::WorkData) -> Result<(), WorkerError>
    where
        P::WorkData: 'static + Send,
    {
        unsafe {
            let worker_data = mem::ManuallyDrop::new(worker_data);
            let size = mem::size_of_val(&worker_data) as u32;
            let ptr = &worker_data as *const _ as *const c_void;
            let schedule_work = if let Some(schedule_work) = self.schedule_work {
                schedule_work
            } else {
                return Err(WorkerError::Unknown);
            };
            match (schedule_work)(self.schedule_handle, size, ptr) {
                lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS => Ok(()),
                lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN => {
                    Err(WorkerError::Unknown)
                }
                lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE => {
                    Err(WorkerError::NoSpace)
                }
                _ => {
                    Err(WorkerError::Unknown)
                }
            }
        }
    }
}

impl<P: Worker> From<Schedule<'_>> for ScheduleHandler<P> {
    fn from(schedule_feature: Schedule) -> Self {
        Self {
            schedule_handle: schedule_feature.internal.handle,
            schedule_work: schedule_feature.internal.schedule_work,
            phantom: PhantomData::<P>,
        }
    }
}

/// Response handler to use inside worker function when youwant to send response to `run()`.
pub struct ResponseHandler<P: Worker> {
    /// function provided by the host to send response to `run()`
    response_function: lv2_sys::LV2_Worker_Respond_Function,
    /// Response handler provided by the host, must be passed to the host provided
    /// response_function.
    respond_handle: lv2_sys::LV2_Worker_Respond_Handle,
    phantom: PhantomData<P>,
}

impl<P: Worker> ResponseHandler<P> {
    pub fn respond(&self, response_data: P::ResponseData) -> Result<(), WorkerError>
    where
        P::WorkData: 'static + Send,
    {
        unsafe {
            let response_data = mem::ManuallyDrop::new(response_data);
            let size = mem::size_of_val(&response_data) as u32;
            let ptr = &response_data as *const _ as *const c_void;
            let response_function = if let Some(response_function) = self.response_function {
                response_function
            } else {
                return Err(WorkerError::Unknown);
            };
            match (response_function)(self.respond_handle, size, ptr) {
                lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS => Ok(()),
                lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN => Err(WorkerError::Unknown),
                lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE => Err(WorkerError::NoSpace),
                _ => Err(WorkerError::Unknown),
            }
        }
    }
}

/// Errors potentially generated by worker methods
pub enum WorkerError {
    /// Unknown or general error
    Unknown,
    /// Failure due to a lack of space
    NoSpace,
}

/// Trait to provide worker extension to LV2 plugins.
///
/// The worker extension allows plugins to schedule work that must be performed in another thread.
/// Plugins can use this interface to safely perform work that is not real-time safe, and receive
/// the result in the run context. The details of threading are managed by the host, allowing
/// plugins to be simple and portable while using resources more efficiently.

/// Host feature allowing plugins to schedule work that must be performed in another thread.
/// Plugins can use this interface to safely perform work that is not real-time safe, and receive
/// the result in the run context.
pub trait Worker: Plugin {
    /// Data sended to the worker thread
    type WorkData: 'static + Send;
    /// Data sended by the worker thread to `work_response`
    type ResponseData: 'static + Send;
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
        response_handler: &ResponseHandler<Self>,
        data: Self::WorkData,
    ) -> Result<(), WorkerError>;

    /// Handle a response from the worker. The spec require plugins to implement this method even if
    /// many host support to not have it.
    ///
    /// This is called by the host in the run() context when a response from the worker is ready.
    fn work_response(&mut self, data: Self::ResponseData) -> Result<(), WorkerError>;

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
        //deref plugin_instance and get the plugin
        let plugin_instance =
            if let Some(plugin_instance) = (handle as *mut PluginInstance<P>).as_mut() {
                plugin_instance
            } else {
                return lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN;
            };
        let plugin = plugin_instance.instance_mut();
        //build response handler
        let response_handler = ResponseHandler {
            response_function,
            respond_handle,
            phantom: PhantomData::<P>,
        };
        //build ref to worker data from raw pointer
        let worker_data =
            ptr::read_unaligned(data as *const mem::ManuallyDrop<<P as Worker>::WorkData>);
        let worker_data = mem::ManuallyDrop::into_inner(worker_data);
        if size as usize != mem::size_of_val(&worker_data) {
            return lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN;
        }
        match plugin.work(&response_handler, worker_data) {
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
        //deref plugin_instance and get the plugin
        let plugin_instance =
            if let Some(plugin_instance) = (handle as *mut PluginInstance<P>).as_mut() {
                plugin_instance
            } else {
                return lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN;
            };
        let plugin = plugin_instance.instance_mut();
        //build ref to response data from raw pointer
        let response_data =
            ptr::read_unaligned(body as *const mem::ManuallyDrop<<P as Worker>::ResponseData>);
        let response_data = mem::ManuallyDrop::into_inner(response_data);
        if size as usize != mem::size_of_val(&response_data) {
            return lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN;
        }

        match plugin.work_response(response_data) {
            Ok(()) => lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS,
            Err(WorkerError::Unknown) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
            Err(WorkerError::NoSpace) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
        }
    }

    /// Extern unsafe version of `end_run` method actually called by the host
    // This throw a warning if it's not in `INTERFACE`
    unsafe extern "C" fn extern_end_run(handle: lv2_sys::LV2_Handle) -> lv2_sys::LV2_Worker_Status {
        if let Some(plugin_instance) = (handle as *mut PluginInstance<P>).as_mut() {
            let plugin = plugin_instance.instance_mut();
            match plugin.end_run() {
                Ok(()) => lv2_sys::LV2_Worker_Status_LV2_WORKER_SUCCESS,
                Err(WorkerError::Unknown) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN,
                Err(WorkerError::NoSpace) => lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_NO_SPACE,
            }
        } else {
            lv2_sys::LV2_Worker_Status_LV2_WORKER_ERR_UNKNOWN
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
    use super::*;
    use lv2_sys::*;
    use std::fmt;
    use std::mem;
    use std::ops;
    use std::ptr;

    // structure to test drooping issue
    struct HasDrop {
        drop_count: u32,
        drop_limit: u32,
    }

    impl HasDrop {
        fn new(val: u32) -> Self {
            Self {
                drop_count: 0,
                drop_limit: val,
            }
        }
    }

    impl ops::Drop for HasDrop {
        fn drop(&mut self) {
            if self.drop_count >= self.drop_limit {
                panic!("Dropped more than {} time", self.drop_limit);
            } else {
                self.drop_count += 1;
            }
        }
    }

    impl fmt::Display for HasDrop {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "HasDrop variable")
        }
    }
    #[derive(PortContainer)]
    struct Ports {}
    struct TestDropWorker;
    // URI identifier
    unsafe impl<'a> UriBound for TestDropWorker {
        const URI: &'static [u8] = b"not relevant\0";
    }
    impl Plugin for TestDropWorker {
        type Ports = Ports;
        type Features = ();

        fn new(_plugin_info: &PluginInfo, _features: ()) -> Option<Self> {
            Some(Self {})
        }

        fn activate(&mut self) {}

        fn deactivate(&mut self) {}

        fn run(&mut self, _ports: &mut Ports) {}
    }

    impl Worker for TestDropWorker {
        type WorkData = HasDrop;
        type ResponseData = HasDrop;

        fn work(
            &mut self,
            _response_handler: &ResponseHandler,
            _data: HasDrop,
        ) -> Result<(), WorkerError> {
            Ok(())
        }

        fn work_response(&mut self, _data: HasDrop) -> Result<(), WorkerError> {
            Ok(())
        }
    }

    extern "C" fn extern_schedule(
        _handle: LV2_Worker_Schedule_Handle,
        _size: u32,
        _data: *const c_void,
    ) -> LV2_Worker_Status {
        LV2_Worker_Status_LV2_WORKER_SUCCESS
    }

    extern "C" fn extern_respond(
        _handle: LV2_Worker_Respond_Handle,
        _size: u32,
        _data: *const c_void,
    ) -> LV2_Worker_Status {
        LV2_Worker_Status_LV2_WORKER_SUCCESS
    }

    #[test]
    fn schedule_must_not_drop() {
        let hd = HasDrop::new(0);
        let schedule = Schedule {
            internal: &lv2_sys::LV2_Worker_Schedule {
                handle: ptr::null_mut(),
                schedule_work: Some(extern_schedule),
            },
        };
        let _ = schedule.schedule_work::<TestDropWorker>(hd);
    }

    #[test]
    fn respond_must_not_drop() {
        let hd = HasDrop::new(0);
        let respond = ResponseHandler {
            response_function: Some(extern_respond),
            respond_handle: ptr::null_mut(),
        };
        let _ = respond.respond::<TestDropWorker>(hd);
    }

    #[test]
    #[should_panic(expected = "Dropped")]
    fn extern_work_should_drop() {
        unsafe {
            let hd = mem::ManuallyDrop::new(HasDrop::new(0));
            let ptr_hd = &hd as *const _ as *const c_void;
            let size = mem::size_of_val(&hd) as u32;
            let mut tdw = TestDropWorker {};

            let ptr_tdw = &mut tdw as *mut _ as *mut c_void;
            //trash trick i use Plugin ptr insteas of Pluginstance ptr
            WorkerDescriptor::<TestDropWorker>::extern_work(
                ptr_tdw,
                Some(extern_respond),
                ptr::null_mut(),
                size,
                ptr_hd,
            );
        }
    }

    #[test]
    fn extern_work_should_not_drop_twice() {
        unsafe {
            let hd = mem::ManuallyDrop::new(HasDrop::new(1));
            let ptr_hd = &hd as *const _ as *const c_void;
            let size = mem::size_of_val(&hd) as u32;
            let mut tdw = TestDropWorker {};

            let ptr_tdw = &mut tdw as *mut _ as *mut c_void;
            //trash trick i use Plugin ptr insteas of Pluginstance ptr
            WorkerDescriptor::<TestDropWorker>::extern_work(
                ptr_tdw,
                Some(extern_respond),
                ptr::null_mut(),
                size,
                ptr_hd,
            );
        }
    }

    #[test]
    #[should_panic(expected = "Dropped")]
    fn extern_work_response_should_drop() {
        unsafe {
            let hd = mem::ManuallyDrop::new(HasDrop::new(0));
            let ptr_hd = &hd as *const _ as *const c_void;
            let size = mem::size_of_val(&hd) as u32;
            let mut tdw = TestDropWorker {};

            let ptr_tdw = &mut tdw as *mut _ as *mut c_void;
            //trash trick i use Plugin ptr insteas of Pluginstance ptr
            WorkerDescriptor::<TestDropWorker>::extern_work_response(ptr_tdw, size, ptr_hd);
        }
    }

    #[test]
    fn extern_work_response_should_not_drop_twice() {
        unsafe {
            let hd = mem::ManuallyDrop::new(HasDrop::new(1));
            let ptr_hd = &hd as *const _ as *const c_void;
            let size = mem::size_of_val(&hd) as u32;
            let mut tdw = TestDropWorker {};

            let ptr_tdw = &mut tdw as *mut _ as *mut c_void;
            //trash trick i use Plugin ptr insteas of Pluginstance ptr
            WorkerDescriptor::<TestDropWorker>::extern_work_response(ptr_tdw, size, ptr_hd);
        }
    }
}
