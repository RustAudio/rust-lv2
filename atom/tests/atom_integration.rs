extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_units as units;

use atom::prelude::*;
use core::prelude::*;
use lv2_urid::*;
use units::prelude::*;
use urid::*;

#[derive(PortCollection)]
struct Ports {
    input: InputPort<AtomPort>,
    output: OutputPort<AtomPort>,
}

#[derive(FeatureCollection)]
struct Features<'a> {
    map: LV2Map<'a>,
}

#[derive(URIDCollection)]
struct URIDs {
    atom: AtomURIDCollection,
    units: UnitURIDCollection,
}

#[uri("urn:rust-lv2:atom-plugin")]
struct AtomPlugin {
    urids: URIDs,
}

impl Plugin for AtomPlugin {
    type Ports = Ports;
    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, features: &mut Features) -> Option<Self> {
        Some(Self {
            urids: features.map.populate_collection()?,
        })
    }

    fn run(&mut self, ports: &mut Ports, _: &mut (), _: u32) {
        let sequence_reader = ports
            .input
            .read::<Sequence>(self.urids.atom.sequence, self.urids.units.beat)
            .unwrap();
        let mut sequence_writer = ports
            .output
            .init::<Sequence>(
                self.urids.atom.sequence,
                TimeStampURID::Frames(self.urids.units.frame),
            )
            .unwrap();

        for (time_stamp, atom) in sequence_reader {
            match atom.read(self.urids.atom.int, ()) {
                Some(number) => {
                    sequence_writer
                        .init::<Int>(time_stamp, self.urids.atom.int, number * 2)
                        .unwrap();
                }
                None => {
                    sequence_writer.forward(time_stamp, atom).unwrap();
                }
            }
        }
    }
}

lv2_descriptors![AtomPlugin];

#[test]
fn main() {
    use atom::space::*;
    use lv2_urid::*;
    use std::ffi::{c_void, CStr};
    use std::mem::size_of;
    use std::pin::Pin;
    use urid::*;

    // Instantiating all features.
    let mut mapper: Pin<Box<HostMap<HashURIDMapper>>> = Box::pin(HashURIDMapper::new().into());
    let map_interface = Box::pin(mapper.as_mut().make_map_interface());
    let map = LV2Map::new(map_interface.as_ref().get_ref());

    let mut map_feature_interface = Box::pin(mapper.as_mut().make_map_interface());
    let map_feature = Box::pin(sys::LV2_Feature {
        URI: LV2Map::URI.as_ptr() as *const i8,
        data: map_feature_interface.as_mut().get_mut() as *mut _ as *mut c_void,
    });
    let features_list: &[*const sys::LV2_Feature] =
        &[map_feature.as_ref().get_ref(), std::ptr::null()];

    // Retrieving URIDs.
    let urids: URIDs = map.populate_collection().unwrap();

    // Preparing the input atom.
    let mut input_atom_space: Box<[u8]> = Box::new([0; 256]);
    {
        let mut space = RootMutSpace::new(input_atom_space.as_mut());
        let mut writer = (&mut space as &mut dyn MutSpace)
            .init(
                urids.atom.sequence,
                TimeStampURID::Frames(urids.units.frame),
            )
            .unwrap();
        writer
            .init(TimeStamp::Frames(0), urids.atom.int, 42)
            .unwrap();
        writer
            .init(TimeStamp::Frames(1), urids.atom.long, 17)
            .unwrap();
        writer
            .init(TimeStamp::Frames(2), urids.atom.int, 3)
            .unwrap();
    }

    // preparing the output atom.
    let mut output_atom_space: Box<[u8]> = Box::new([0; 256]);
    {
        let mut space = RootMutSpace::new(output_atom_space.as_mut());
        (&mut space as &mut dyn MutSpace)
            .init(urids.atom.chunk, ())
            .unwrap()
            .allocate(256 - size_of::<sys::LV2_Atom>(), false)
            .unwrap();
    }

    unsafe {
        // retrieving the descriptor.
        let plugin_descriptor = &*lv2_descriptor(0);
        assert_eq!(
            CStr::from_ptr(plugin_descriptor.URI).to_str().unwrap(),
            "urn:rust-lv2:atom-plugin"
        );

        // Instantiating the plugin.
        let plugin = (plugin_descriptor.instantiate.unwrap())(
            plugin_descriptor,
            44100.0,
            b"\0".as_ptr() as *const i8,
            features_list.as_ptr(),
        );

        // connecting the ports.
        (plugin_descriptor.connect_port.unwrap())(
            plugin,
            0,
            input_atom_space.as_mut_ptr() as *mut c_void,
        );
        (plugin_descriptor.connect_port.unwrap())(
            plugin,
            1,
            output_atom_space.as_mut_ptr() as *mut c_void,
        );

        // Activate, run, deactivate.
        (plugin_descriptor.activate.unwrap())(plugin);
        (plugin_descriptor.run.unwrap())(plugin, 256);
        (plugin_descriptor.deactivate.unwrap())(plugin);

        // Cleanup.
        (plugin_descriptor.cleanup.unwrap())(plugin);
    }

    // Asserting the result
    let (sequence, _) = Space::from_slice(output_atom_space.as_ref())
        .split_atom_body(urids.atom.sequence)
        .unwrap();
    for (stamp, atom) in Sequence::read(sequence, urids.units.beat).unwrap() {
        let stamp = stamp.as_frames().unwrap();
        match stamp {
            0 => assert_eq!(atom.read(urids.atom.int, ()).unwrap(), 84),
            1 => assert_eq!(atom.read(urids.atom.long, ()).unwrap(), 17),
            2 => assert_eq!(atom.read(urids.atom.int, ()).unwrap(), 6),
            _ => panic!("Invalid time stamp in sequence!"),
        }
    }
}
