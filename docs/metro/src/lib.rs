// Now, we put it all together:
use iterpipes::*;
use lv2::prelude::*;

mod pipes;
use pipes::*;

// In future iterations of the plugin, these values could be parameters, but for now, the're constants:
const ATTACK_DURATION: f64 = 0.005;
const DECAY_DURATION: f64 = 0.075;
const NOTE_FREQUENCY: f64 = 440.0 * 2.0;

#[derive(URIDCollection)]
struct URIDs {
    atom: AtomURIDCollection,
    unit: UnitURIDCollection,
    time: TimeURIDCollection,
}

#[derive(PortCollection)]
pub struct Ports {
    control: InputPort<AtomPort>,
    output: OutputPort<Audio>,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

// This plugin struct contains the URID collection and two pre-constructed pipes. These are later used to construct the complete pipeline.
#[uri("https://github.com/RustAudio/rust-lv2/tree/master/docs/metro")]
pub struct Metro {
    urids: URIDs,
    envelope: Connector<Enumerate<PulseGenerator>, Envelope>,
    sampler: Connector<Counter<usize>, Sampler<f32>>,
}

impl Plugin for Metro {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        let attack_len = (ATTACK_DURATION * plugin_info.sample_rate()) as usize;
        let decay_len = (DECAY_DURATION * plugin_info.sample_rate()) as usize;

        // Pre-construct the envelope pipe. Pipes can be enumerated, just like iterators, and connected.
        let envelope = PulseGenerator::new(plugin_info.sample_rate() as f32)
            .enumerate()
            .connect(Envelope::new(attack_len, decay_len));

        // Calculate the sample and pre-construct the sampler pipe.
        let sample_len = (plugin_info.sample_rate() / NOTE_FREQUENCY) as usize;
        let mut sample: Vec<f32> = Vec::with_capacity(sample_len);
        for i in 0..sample_len {
            sample.push(
                (i as f64 * 2.0 * std::f64::consts::PI * NOTE_FREQUENCY / plugin_info.sample_rate())
                    .sin() as f32,
            );
        }
        let sampler = Counter::<usize>::new(0, 1).connect(Sampler::new(sample));

        Some(Self {
            urids: features.map.populate_collection()?,
            envelope,
            sampler,
        })
    }

    fn activate(&mut self, _: &mut Features<'static>) {
        self.envelope.reset();
        self.sampler.reset();
    }

    fn run(&mut self, ports: &mut Ports, _: &mut (), _: u32) {
        if let Some(control) = ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
        {
            // Here, the final assembly of the pipeline is done. First, the event iterator is pre-processed to only emit an index and an `UnidentifiedAtom`. Then, the event iterator is wrapped into an `EventAtomizer`, which is then connected to an `EventReader` and the envelope. The resulting pipe consumes a `()` and emits the next frame of the envelope; It's already a compact pipeline.
            //
            // Then, the final pipeline is constructed using some lazy pipes: The first one splits a `()` to a tuple of `()`, which is then connected to a tuple of the envelope and the pre-constructed sampler. A tuple of two pipes is also a pipe; The two pipes are processed in parallel. Then, the emitted envelope and sample frame are multiplied to one frame.
            let control =
                control.map(|(timestamp, event)| (timestamp.as_frames().unwrap() as usize, event));

            let complete_envelope = EventAtomizer::new(control).compose()
                >> EventReader::new(&self.urids.atom, &self.urids.time)
                >> &mut self.envelope;

            let mut pipeline = Lazy::new(|_: ()| ((), ())).compose()
                >> (complete_envelope, &mut self.sampler)
                >> Lazy::new(|(env, sample)| env * sample);

            // Generate a frame for every frame in the output buffer. All of the processing is done by the single call to `next`!
            for frame in ports.output.iter_mut() {
                *frame = pipeline.next(());
            }
        }
    }
}

lv2_descriptors!(Metro);
