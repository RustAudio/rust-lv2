use crate::repository::descriptor::PluginDescriptor;

pub struct PluginRepository {}

impl PluginRepository {
    pub fn discover() -> Self {
        todo!()
    }

    pub fn find_plugin(&self, _plugin_uri: &str) -> Option<&PluginDescriptor> {
        todo!()
    }
}

pub struct PluginDiscoveryOptions {}

impl PluginDiscoveryOptions {}

pub mod descriptor;
