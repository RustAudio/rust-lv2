// The same as before...
use lv2::prelude::*;
use wmidi::*;

#[derive(PortCollection)]
pub struct Ports {
    input: InputPort<AtomPort>,
    output: OutputPort<AtomPort>,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

#[derive(URIDCollection)]
pub struct URIDs {
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    unit: UnitURIDCollection,
}

#[uri("https://github.com/RustAudio/rust-lv2/tree/master/docs/fifths")]
pub struct Fifths {
    urids: URIDs,
}

impl Plugin for Fifths {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        Some(Self {
            urids: features.map.populate_collection()?,
        })
    }

    // This plugin works similar to the previous one: It iterates over the events in the input port. However, it only needs to write one or two messages instead of blocks of audio.
    fn run(&mut self, ports: &mut Ports, _: &mut ()) {
        // Get the reading handle of the input sequence.
        let input_sequence = ports
            .input
            .read(self.urids.atom.sequence, self.urids.unit.beat)
            .unwrap();

        // Initialise the output sequence and get the writing handle.
        let mut output_sequence = ports
            .output
            .init(
                self.urids.atom.sequence,
                TimeStampURID::Frames(self.urids.unit.frame),
            )
            .unwrap();

        for (timestamp, atom) in input_sequence {
            // Every message is forwarded, regardless of it's content.
            output_sequence.forward(timestamp, atom);

            // Retrieve the message.
            let message = if let Some(message) = atom.read(self.urids.midi.wmidi, ()) {
                message
            } else {
                continue;
            };

            match message {
                MidiMessage::NoteOn(channel, note, velocity) => {
                    // Create a note 5th (7 semitones) higher than the input.
                    if let Ok(note) = note.step(7) {
                        // Write the fifth. Writing is done after initialization.
                        output_sequence
                            .init(
                                timestamp,
                                self.urids.midi.wmidi,
                                MidiMessage::NoteOn(channel, note, velocity),
                            )
                            .unwrap();
                    }
                }
                MidiMessage::NoteOff(channel, note, velocity) => {
                    // Do the same thing for `NoteOff`.
                    if let Ok(note) = note.step(7) {
                        output_sequence
                            .init(
                                timestamp,
                                self.urids.midi.wmidi,
                                MidiMessage::NoteOff(channel, note, velocity),
                            )
                            .unwrap();
                    }
                }
                _ => (),
            }
        }
    }
}

lv2_descriptors!(Fifths);
