use lv2_core::port::index::PortIndexable;
use lv2_core::prelude::*;
use ui::port_event::*;
use ui::{PluginUi, RootWidget, UiController, UiInfo};
use urid::{uri, UriBound, URID};

#[uri("http://lv2plug.in/plugins.rs/example_amp")]
struct Amp {
    activated: bool,
}

#[derive(PortCollection)]
struct AmpPorts {
    gain: InputPort<InPlaceControl>,
    input: InputPort<InPlaceAudio>,
    output: OutputPort<InPlaceAudio>,
}

// Generated code
const _: () = {
    extern crate lv2_core as __lv2_core;

    #[allow(dead_code)]
    #[derive(Copy, Clone)]
    struct AmpPortsIndexes {
        pub gain: __lv2_core::port::index::PortIndex<InputPort<InPlaceControl>>,
        pub input: __lv2_core::port::index::PortIndex<InputPort<InPlaceAudio>>,
        pub output: __lv2_core::port::index::PortIndex<OutputPort<InPlaceAudio>>,
    }

    unsafe impl __lv2_core::port::index::PortIndexable for AmpPorts {
        type Indexes = AmpPortsIndexes;

        #[inline]
        fn indexes() -> Self::Indexes {
            unsafe {
                AmpPortsIndexes {
                    gain: __lv2_core::port::index::PortIndex::new_unchecked(0),
                    input: __lv2_core::port::index::PortIndex::new_unchecked(1),
                    output: __lv2_core::port::index::PortIndex::new_unchecked(2),
                }
            }
        }
    }
};

struct MyUIUrids {
    float_protocol: URID<FloatProtocol>,
}

struct MyUI<'a> {
    urids: MyUIUrids,
    controller: UiController<'a>,
}

impl<'a> PluginUi<'a> for MyUI<'a> {
    type Features = ();

    fn new(
        ui_info: &UiInfo,
        controller: UiController,
        ui_features: Self::Features,
    ) -> Option<Self> {
        todo!()
    }

    fn root_widget(&self) -> RootWidget<'a> {
        todo!()
    }

    fn port_event(&mut self, event: &PortEvent) {
        let value = event
            .read(AmpPorts::indexes().gain, self.urids.float_protocol)
            .unwrap();

        self.controller
            .write_to_port(AmpPorts::indexes().gain, self.urids.float_protocol, 42.0);
        todo!()
    }
}
