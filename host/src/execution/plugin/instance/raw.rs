use crate::repository::descriptor::PluginDescriptor;
use std::ffi::c_void;
use std::ptr::NonNull;

struct RawPluginInstance {
    descriptor: lv2_sys::LV2_Descriptor,
    handle: NonNull<c_void>,
}

impl RawPluginInstance {
    pub fn new(descriptor: &PluginDescriptor) -> Option<Self> {
        let descriptor = *descriptor.raw_descriptor();

        let features = todo!();
        let sample_rate = todo!();
        let bundle_path = todo!();

        // SAFETY: here we trust the plugin to provide valid function pointers.
        let instance_ptr =
            unsafe { descriptor.instantiate?(&descriptor, sample_rate, bundle_path, features) };

        let handle = NonNull::new(instance_ptr)?;
        Some(RawPluginInstance { handle, descriptor })
    }

    pub unsafe fn connect_ports() {
        todo!()
    }

    pub unsafe fn activate(&mut self) {
        if let Some(activate) = self.descriptor.activate {
            activate(self.handle.as_ptr())
        }
    }

    pub unsafe fn deactivate(&mut self) {
        if let Some(deactivate) = self.descriptor.deactivate {
            deactivate(self.handle.as_ptr())
        }
    }
}
