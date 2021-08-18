// We cover the individual pipes of the plugin before putting it all together:
use iterpipes::*;
use lv2::prelude::*;

// `Sampler` is a simple sampler that plays back the contents of a pre-recorded sample. It simply returns a frame for every index it receives as an input, which means that it can also be played backward or at a different speed. The actual type of frames isn't important and therefore, this sampler is generic.

pub struct Sampler<T> {
    sample: Box<[T]>,
}

impl<T> Sampler<T> {
    pub fn new<S>(sample: S) -> Self
    where
        S: Into<Box<[T]>>,
    {
        Self {
            sample: sample.into(),
        }
    }
}

impl<T> Pipe for Sampler<T>
where
    T: Copy,
{
    type InputItem = usize;
    type OutputItem = T;

    fn next(&mut self, index: usize) -> T {
        self.sample[index % self.sample.len()]
    }
}

impl<T> ResetablePipe for Sampler<T>
where
    T: Copy,
{
    fn reset(&mut self) {}
}

// We try to test as much of the individual parts as possible to reduce the error cases.
#[test]
fn test_sampler() {
    let sample: Vec<u8> = vec![1, 2, 3, 4];
    let mut sampler = Sampler::new(sample);
    for i in (0..32).chain(32..0) {
        assert_eq!((i % 4 + 1) as u8, sampler.next(i));
    }
}

// `Envelope` receives a pulse and an index and creates an envelope after every pulse. This envelope is multiplied with the sample output to generate the sound signal.

pub struct Envelope {
    attack_len: usize,
    decay_len: usize,
    impulse_index: usize,
}

impl Envelope {
    pub fn new(attack_len: usize, decay_len: usize) -> Self {
        Self {
            attack_len,
            decay_len,
            impulse_index: std::usize::MAX,
        }
    }
}

impl Pipe for Envelope {
    type InputItem = (usize, bool);
    type OutputItem = f32;

    fn next(&mut self, (index, impulse): (usize, bool)) -> f32 {
        if impulse {
            self.impulse_index = index;
        }

        if index < self.impulse_index {
            0.0
        } else if index < self.impulse_index + self.attack_len {
            (index - self.impulse_index) as f32 / (self.attack_len) as f32
        } else if index < self.impulse_index + self.attack_len + self.decay_len {
            1.0 - ((index - self.impulse_index - self.attack_len) as f32 / (self.decay_len) as f32)
        } else {
            0.0
        }
    }
}

impl ResetablePipe for Envelope {
    fn reset(&mut self) {
        self.impulse_index = std::usize::MAX;
    }
}

#[test]
fn test_envelope() {
    let mut pipe =
        Envelope::new(4, 4).compose() >> Lazy::new(|frame: f32| (frame * 4.0).round() as u8);
    for i in 0..32 {
        assert_eq!(0, pipe.next((i, false)));
    }
    assert_eq!(0, pipe.next((32, true)));
    assert_eq!(1, pipe.next((33, false)));
    assert_eq!(2, pipe.next((34, false)));
    assert_eq!(3, pipe.next((35, false)));
    assert_eq!(4, pipe.next((36, false)));
    assert_eq!(3, pipe.next((37, false)));
    assert_eq!(2, pipe.next((38, false)));
    assert_eq!(1, pipe.next((39, false)));
    assert_eq!(0, pipe.next((40, false)));
    for i in 41..64 {
        assert_eq!(0, pipe.next((i, false)));
    }
}

// The `PulseGenerator` interprets the settings of the host and creates a pulse every time a new note should be played. This pulse is a `bool` that flips from `false` to `true`.
//
// The host settings are updated via `PulseInput` objects, which contain new BPM and speed measures as well as the number of the current beat in the current bar for synchronization.
//
// Note that the `elapsed_frames` counter is only used internally to generate pulses. The index counters for the envelope and the samples are separate, which means that the audio won't stutter after a hard update.
pub struct PulseGenerator {
    sample_rate: f32,

    beats_per_minute: f32,
    speed_coefficient: f32,
    frames_per_beat: usize,

    elapsed_frames: usize,
}

impl PulseGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,

            beats_per_minute: 120.0,
            speed_coefficient: 0.0,
            frames_per_beat: 0,

            elapsed_frames: 0,
        }
    }
}

impl Pipe for PulseGenerator {
    type InputItem = PulseInput;
    type OutputItem = bool;

    fn next(&mut self, input: PulseInput) -> bool {
        self.elapsed_frames += 1;

        let mut parameters_changed = false;
        if let Some(new_bpm) = input.bpm_update {
            self.beats_per_minute = new_bpm;
            parameters_changed = true;
        }
        if let Some(new_speed) = input.speed_update {
            self.speed_coefficient = new_speed;
            parameters_changed = true;
        }

        if parameters_changed {
            self.frames_per_beat =
                (self.speed_coefficient * (60.0 / self.beats_per_minute) * self.sample_rate).abs()
                    as usize;
        }

        if let Some(new_beat) = input.beat_update {
            self.elapsed_frames = (new_beat * self.frames_per_beat as f64) as usize;
        }

        self.frames_per_beat != 0 && self.elapsed_frames % self.frames_per_beat == 0
    }
}

impl ResetablePipe for PulseGenerator {
    fn reset(&mut self) {
        self.beats_per_minute = 120.0;
        self.speed_coefficient = 0.0;
        self.frames_per_beat = 0;
        self.elapsed_frames = 0;
    }
}

#[test]
fn test_pulse_generator() {
    let mut pipe = PulseGenerator::new(44100.0);
    assert!(pipe.next(PulseInput {
        beat_update: Some(0.0),
        bpm_update: Some(120.0),
        speed_update: Some(1.0)
    }));

    for i in 1..88100 {
        let input = PulseInput {
            beat_update: None,
            bpm_update: None,
            speed_update: None,
        };
        if i % 22050 == 0 {
            assert!(pipe.next(input));
        } else {
            assert!(!pipe.next(input));
        }
    }
}

// This is the input type for the pulse generator. The `bpm_update` and `speed_update` fields tell the pulse generator of the new number of beats per second and playback speed. The `beat_update` contains the number of the current beat in the current bar and is used to synchronize the plugin with the host.

#[derive(Clone, Copy, Debug)]
pub struct PulseInput {
    pub beat_update: Option<f64>,
    pub bpm_update: Option<f32>,
    pub speed_update: Option<f32>,
}

// The `EventAtomizer` wraps an iterator over events and transforms them into frames, which either contain an event or don't. This iterator will be the atom event iterator later, but for now, it's good to be generic.
//
// Internally, it stores the next event of the event sequence. Every time `next` is called, this counter is increased and once it hits this next event, it is yielded and the next "next event" is retrieved. This is continued as long as the sequence contains events. Once it is depleted, this pipe only emits `None`s.
//
// Since every frame can only contain one event and frames must be emitted chronologically, it drops every event that has the same or an earlier timestamp than a previous event.
pub struct EventAtomizer<T, I>
where
    I: Iterator<Item = (usize, T)>,
{
    sequence: I,
    next_event: Option<(usize, T)>,
    index: usize,
}

impl<T, I> EventAtomizer<T, I>
where
    I: Iterator<Item = (usize, T)>,
{
    pub fn new(sequence: I) -> Self {
        let mut instance = Self {
            sequence,
            next_event: None,
            index: 0,
        };
        instance.retrieve_next_event();
        instance
    }

    fn retrieve_next_event(&mut self) {
        self.next_event = None;
        if let Some((index, item)) = self.sequence.next() {
            if index >= self.index {
                self.next_event = Some((index, item));
            }
        }
    }
}

impl<T, I> Pipe for EventAtomizer<T, I>
where
    I: Iterator<Item = (usize, T)>,
{
    type InputItem = ();
    type OutputItem = Option<T>;

    fn next(&mut self, _: ()) -> Option<T> {
        match self.next_event.take() {
            Some((event_index, event_atom)) => {
                let event_is_due = event_index == self.index;
                self.index += 1;
                if event_is_due {
                    self.retrieve_next_event();
                    Some(event_atom)
                } else {
                    self.next_event = Some((event_index, event_atom));
                    None
                }
            }
            None => None,
        }
    }
}

#[test]
fn test_atomizer() {
    let events: Box<[(usize, u32)]> = Box::new([(4, 1), (10, 5)]);
    let mut pipe = EventAtomizer::new(events.iter().cloned());

    for i in 0..15 {
        let output = pipe.next(());
        match i {
            4 => assert_eq!(Some(1), output),
            10 => assert_eq!(Some(5), output),
            _ => assert_eq!(None, output),
        }
    }
}

// In the final plugin, the `EventAtomizer` emits `Option<UnidentifiedAtom>`s, which might be any atom at all, and the `PulseGenerator` consumes `PulseInput`s. The `EventReader` bridges the gap between these two pipes by identifying the atom, reading it and emitting an appropriate `PulseInput`.
//
// This is the only pipe that isn't tested since creating a testbed for it would require too much code for this book.

pub struct EventReader<'a> {
    atom_urids: &'a AtomURIDCollection,
    time_urids: &'a TimeURIDCollection,
}

impl<'a> EventReader<'a> {
    pub fn new(atom_urids: &'a AtomURIDCollection, time_urids: &'a TimeURIDCollection) -> Self {
        Self {
            atom_urids,
            time_urids,
        }
    }
}

impl<'a> Pipe for EventReader<'a> {
    type InputItem = Option<&'a UnidentifiedAtom>;
    type OutputItem = PulseInput;

    fn next(&mut self, atom: Option<&'a UnidentifiedAtom>) -> PulseInput {
        let mut updates = PulseInput {
            beat_update: None,
            bpm_update: None,
            speed_update: None,
        };

        if let Some(atom) = atom {
            if let Some((object_header, object_reader)) = atom
                .read(self.atom_urids.object, ())
                .or_else(|| atom.read(self.atom_urids.blank, ()))
            {
                if object_header.otype == self.time_urids.position_class {
                    for (property_header, property) in object_reader {
                        if property_header.key == self.time_urids.bar_beat {
                            updates.beat_update = property
                                .read(self.atom_urids.float, ())
                                .map(|float| float as f64);
                        }
                        if property_header.key == self.time_urids.beats_per_minute {
                            updates.bpm_update = property.read(self.atom_urids.float, ());
                        }
                        if property_header.key == self.time_urids.speed {
                            updates.speed_update = property.read(self.atom_urids.float, ());
                        }
                    }
                }
            }
        }

        updates
    }
}
