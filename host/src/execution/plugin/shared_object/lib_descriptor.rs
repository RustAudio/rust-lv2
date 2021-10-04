use std::ptr::NonNull;

pub struct LibDescriptor {
    raw: NonNull<lv2_sys::LV2_Lib_Descriptor>,
}

impl LibDescriptor {
    fn get_descriptors(&self) -> Vec<lv2_sys::LV2_Descriptor> {
        todo!()
    }
}

impl Drop for LibDescriptor {
    fn drop(&mut self) {
        let raw = unsafe { self.raw.as_ref() };
        if let Some(cleanup) = raw.cleanup {
            unsafe { cleanup(raw.handle) };
        }
    }
}
