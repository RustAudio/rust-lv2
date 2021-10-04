use crate::execution::plugin::instance::InactivePluginInstance;
use crate::repository::descriptor::PluginDescriptor;

pub struct PluginExecutionContext;

#[derive(Debug)]
pub struct PluginInstanciationError;

impl PluginExecutionContext {
    pub fn new() -> Self {
        todo!()
    }

    pub fn instanciate(
        &mut self,
        _descriptor: &PluginDescriptor,
        _sample_rate: f32,
    ) -> Result<InactivePluginInstance, PluginInstanciationError> {
        todo!()
    }
}

mod plugin;
