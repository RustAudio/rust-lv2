extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_units as units;
extern crate lv2_urid as urid;

use atom::prelude::*;
use core::prelude::*;
use units::UnitURIDCache;
use urid::feature::Map;
use urid::URIDCache;

#[derive(PortContainer)]
struct Ports {
    input: InputPort<AtomPort>,
    output: OutputPort<AtomPort>,
}

#[derive(FeatureCollection)]
struct Features<'a> {
    map: &'a Map<'a>,
}

#[derive(URIDCache)]
struct URIDs {
    atom: AtomURIDCache,
    units: UnitURIDCache,
}

struct AtomPlugin {
    urids: URIDs,
}

impl Plugin for AtomPlugin {
    type Ports = Ports;
    type Features = Features<'static>;

    fn new(_plugin_info: &PluginInfo, features: Features) -> Option<Self> {
        Some(Self {
            urids: features.map.populate_cache()?,
        })
    }

    fn run(&mut self, ports: &mut Ports) {
        let sequence_reader = ports
            .input
            .read::<Sequence>(self.urids.atom.sequence, self.urids.units.bpm)
            .unwrap();
        let mut sequence_writer = ports
            .output
            .write::<Sequence>(
                self.urids.atom.sequence,
                TimeStampURID::Frames(self.urids.units.frame),
            )
            .unwrap();

        for (time_stamp, atom) in sequence_reader {
            match atom.read(self.urids.atom.int, ()) {
                Some(number) => {
                    sequence_writer
                        .write::<Int>(time_stamp, self.urids.atom.int, number * 2)
                        .unwrap();
                }
                None => {
                    sequence_writer.forward(time_stamp, atom).unwrap();
                }
            }
        }
    }
}

lv2_descriptors! {
    AtomPlugin: "urn:rust-lv2:atom-plugin"
}
