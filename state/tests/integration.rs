use lv2_atom::prelude::*;
use lv2_core::feature::{FeatureCollection, FeatureContainer, MissingFeatureError};
use lv2_core::prelude::*;
use lv2_state::access::*;
use lv2_state::plugin::*;
use lv2_urid::prelude::*;

struct Stateful {
    internal: f32,
    audio: Vec<f32>,

    urids: AtomURIDCache,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: Map<'a>,
}

unsafe impl UriBound for Stateful {
    const URI: &'static [u8] = b"http://lv2plug.in/plugins.rs/example_amp\0";
}

impl Plugin for Stateful {
    type Ports = ();
    type Features = Features<'static>;

    fn new(_plugin_info: &PluginInfo, features: Features<'static>) -> Option<Self> {
        Some(Stateful {
            internal: 42.0,
            audio: Vec::new(),
            urids: features.map.populate_cache()?,
        })
    }

    fn run(&mut self, _: &mut ()) {
        self.internal = 17.0;
        self.audio.extend((0..32).map(|f| f as f32));
    }

    fn extension_data(uri: &Uri) -> Option<&'static dyn std::any::Any> {
        match_extensions!(uri, StateDescriptor<Self>)
    }
}

impl State for Stateful {
    type StateFeatures = ();

    fn save(&self, mut store: StoreHandle, _: ()) {
        store
            .draft(URID::new(1000).unwrap())
            .init(self.urids.float, self.internal)
            .unwrap();
        store
            .draft(URID::new(1001).unwrap())
            .init(self.urids.vector(), self.urids.float)
            .unwrap()
            .append(self.audio.as_ref());
    }

    fn restore(&mut self, store: RetrieveHandle, _: ()) {
        self.internal = store
            .retrieve(URID::new(1000).unwrap(), self.urids.float, ())
            .unwrap();
        self.audio = Vec::from(
            store
                .retrieve(
                    URID::new(1001).unwrap(),
                    self.urids.vector(),
                    self.urids.float,
                )
                .unwrap(),
        );
    }
}

lv2_descriptors! {
    Stateful
}
