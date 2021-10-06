use crate::port_event::{PortEvent, PortProtocol};
use lv2_core::port::index::PortIndex;
use lv2_core::port::PortHandle;
use lv2_core::prelude::FeatureCollection;
use std::marker::PhantomData;
use urid::URID;

pub struct UiInfo;

pub struct UiController<'a>(PhantomData<&'a ()>);

impl<'a> UiController<'a> {
    pub fn write_to_port<'b, P: PortHandle, E: PortProtocol<'b, P>>(
        &'b self,
        port_index: PortIndex<P>,
        event_type: URID<E>,
        value: E::WriteParameter,
    ) {
        todo!()
    }
}

pub mod port_event;

pub struct RootWidget<'a>(&'a ());

pub trait PluginUi<'a>: Sized + 'a {
    type Features: FeatureCollection<'a>;

    fn new(
        ui_info: &UiInfo,
        controller: UiController<'a>,
        ui_features: Self::Features,
    ) -> Option<Self>;

    fn root_widget(&self) -> RootWidget<'a>;
    fn port_event(&mut self, event: &PortEvent);
}
