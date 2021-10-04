use lv2_core::prelude::FeatureCollection;

pub struct UiInfo;

pub struct UiController;

pub struct PortEvent;

pub trait PluginUi<'a>: Sized + 'a {
    type Features: FeatureCollection<'a>;

    fn new(ui_info: &UiInfo, controller: UiController, ui_features: Self::Features)
        -> Option<Self>;

    fn root_widget(&self) -> ();
    fn port_event(&mut self, event: &PortEvent);
}
