use crate::port_event::{PortEvent, PortProtocol};
use lv2_core::port::index::PortIndex;
use lv2_core::port::PortHandle;
use lv2_core::prelude::FeatureCollection;
use std::marker::PhantomData;
use urid::URID;

pub struct UiInfo;

pub struct UiController<'a> {
    controller: lv2_sys::LV2UI_Controller,
    write_function: lv2_sys::LV2UI_Write_Function,
    _lifetime: PhantomData<&'a lv2_sys::LV2UI_Controller>,
}

impl<'a> UiController<'a> {
    #[inline]
    pub fn write_to_port<'b, P: PortHandle, E: PortProtocol<'b, P>>(
        &'b self,
        port_index: PortIndex<P>,
        event_type: URID<E>,
        value: E::WriteParameter,
    ) {
        unsafe {
            self.write_function.unwrap()(
                self.controller,
                port_index.get(),
                E::size_of_write_parameter(&value) as u32,
                event_type.get(),
                &value as *const _ as *const _,
            )
        }
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

pub mod extensions;
pub mod features;
