use ui::{PluginUi, PortEvent, UiController, UiInfo};

struct MyUI {}

impl<'a> PluginUi<'a> for MyUI {
    type Features = ();

    fn new(
        ui_info: &UiInfo,
        controller: UiController,
        ui_features: Self::Features,
    ) -> Option<Self> {
        todo!()
    }

    fn root_widget(&self) -> () {
        todo!()
    }

    fn port_event(&mut self, event: &PortEvent) {
        todo!()
    }
}
