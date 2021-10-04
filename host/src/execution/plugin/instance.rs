use crate::execution::PluginExecutionContext;
use std::marker::PhantomData;

struct PluginInstance<'a> {
    context: PhantomData<&'a PluginExecutionContext>,
}

pub struct InactivePluginInstance<'a> {
    inner: PluginInstance<'a>,
}

impl<'a> InactivePluginInstance<'a> {
    pub fn activate(self) -> ActivePluginInstance<'a> {
        todo!()
    }
}

pub struct ActivePluginInstance<'a> {
    inner: PluginInstance<'a>,
}

pub mod raw;
