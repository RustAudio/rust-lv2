// Use the prelude and the `wmidi` crate.
use lv2::prelude::*;
use wmidi::*;

#[derive(PortCollection)]
pub struct Ports {
    control: InputPort<AtomPort>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}
// Now, an additional host feature is needed. A feature is something that implements the `Feature` trait and usually wraps a certain functionality of the host; In this case mapping URIs to URIDs. The discovery and validation of features is done by the framework.
#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}
// Retrieving URIDs from the host isn't guaranteed to be real-time safe or even fast. Therefore, all URIDs that may be needed should be retrieved when the plugin is instantiated. The `URIDCollection` trait makes this easy: It provides a single method that creates an instance of itself from the mapping feature, which can also be generated using this `derive` macro.
#[derive(URIDCollection)]
pub struct URIDs {
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    unit: UnitURIDCollection,
}

#[uri("https://github.com/RustAudio/rust-lv2/tree/master/docs/midigate")]
pub struct Midigate {
    n_active_notes: u64,
    program: u8,
    urids: URIDs,
}

impl Midigate {
    // A function to write a chunk of output, to be called from `run()`. If the gate is high, then the input will be passed through for this chunk, otherwise silence is written.
    fn write_output(&mut self, ports: &mut Ports, offset: usize, mut len: usize) {
        if ports.input.len() < offset + len {
            len = ports.input.len() - offset;
        }

        let active = if self.program == 0 {
            self.n_active_notes > 0
        } else {
            self.n_active_notes == 0
        };

        let input = &ports.input[offset..offset + len];
        let output = &mut ports.output[offset..offset + len];

        if active {
            output.copy_from_slice(input);
        } else {
            for frame in output.iter_mut() {
                *frame = 0.0;
            }
        }
    }
}

impl Plugin for Midigate {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        Some(Self {
            n_active_notes: 0,
            program: 0,
            urids: features.map.populate_collection()?,
        })
    }

    // This plugin works through the cycle in chunks starting at offset zero. The `offset` represents the current time within this this cycle, so the output from 0 to `offset` has already been written.
    //
    // MIDI events are read in a loop. In each iteration, the number of active notes (on note on and note off) or the program (on program change) is updated, then the output is written up until the current event time. Then `offset` is updated and the next event is processed. After the loop the final chunk from the last event to the end of the cycle is emitted.
    //
    // There is currently no standard way to describe MIDI programs in LV2, so the host has no way of knowing that these programs exist and should be presented to the user. A future version of LV2 will address this shortcoming.
    //
    // This pattern of iterating over input events and writing output along the way is a common idiom for writing sample accurate output based on event input.
    //
    // Note that this simple example simply writes input or zero for each sample based on the gate. A serious implementation would need to envelope the transition to avoid aliasing.
    fn run(&mut self, ports: &mut Ports, _: &mut (), _: u32) {
        let mut offset: usize = 0;

        let control_sequence = ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
            .unwrap();

        for (timestamp, message) in control_sequence {
            let timestamp: usize = if let Some(timestamp) = timestamp.as_frames() {
                timestamp as usize
            } else {
                continue;
            };

            let message = if let Some(message) = message.read(self.urids.midi.wmidi, ()) {
                message
            } else {
                continue;
            };

            match message {
                MidiMessage::NoteOn(_, _, _) => self.n_active_notes += 1,
                MidiMessage::NoteOff(_, _, _) => self.n_active_notes -= 1,
                MidiMessage::ProgramChange(_, program) => {
                    let program: u8 = program.into();
                    if program == 0 || program == 1 {
                        self.program = program;
                    }
                }
                _ => (),
            }

            self.write_output(ports, offset, timestamp + offset);
            offset = timestamp;
        }

        self.write_output(ports, offset, ports.input.len() - offset);
    }

    // During it's runtime, the host might decide to deactivate the plugin. When the plugin is reactivated, the host calls this method which gives the plugin an opportunity to reset it's internal state.
    fn activate(&mut self, _features: &mut Features<'static>) {
        self.n_active_notes = 0;
        self.program = 0;
    }
}

lv2_descriptors!(Midigate);
