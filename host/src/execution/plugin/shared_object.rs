use crate::execution::plugin::shared_object::lib_descriptor::LibDescriptor;
use libloading::Library;

pub mod lib_descriptor;

struct PluginSharedObject {
    library: Library,
}

impl PluginSharedObject {
    pub fn get_descriptors(&self) -> Vec<lv2_sys::LV2_Descriptor> {
        todo!()
    }

    pub fn get_lib_descriptor(&self) -> LibDescriptor {
        todo!()
    }
}
