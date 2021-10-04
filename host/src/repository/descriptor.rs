use lv2_sys::LV2_Descriptor;

#[derive(Clone)]
pub struct PluginDescriptor {}

impl PluginDescriptor {
    pub fn uri(&self) -> &str {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }

    pub(crate) fn raw_descriptor(&self) -> &LV2_Descriptor {
        todo!()
    }
}
