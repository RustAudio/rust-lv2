use iterpipes::*;
use lv2::prelude::*;

mod pipes;
use pipes::*;

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

#[uri("https://github.com/Janonard/rust-lv2-book#metro")]
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

        let envelope = PulseGenerator::new(plugin_info.sample_rate() as f32)
            .enumerate()
            .connect(Envelope::new(attack_len, decay_len));

        let sample_len = (plugin_info.sample_rate() / NOTE_FREQUENCY) as usize;
        let mut sample: Vec<f32> = Vec::with_capacity(sample_len);
        for i in 0..sample_len {
            sample.push(
                (i as f64 * 2.0 * std::f64::consts::PI * NOTE_FREQUENCY / plugin_info.sample_rate())
                    .sin() as f32,
            );
        }
        let sampler = Counter::<usize>::new(0, 1).compose() >> Sampler::new(sample);

        Some(Self {
            urids: features.map.populate_collection()?,
            envelope,
            sampler: sampler.unwrap(),
        })
    }

    fn activate(&mut self, _: &mut Features<'static>) {
        self.envelope.reset();
        self.sampler.reset();
    }

    fn run(&mut self, ports: &mut Ports, _: &mut ()) {
        if let Some(control) = ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
        {
            let control =
                control.map(|(timestamp, event)| (timestamp.as_frames().unwrap() as usize, event));

            let complete_envelope = EventAtomizer::new(control).compose()
                >> EventReader::new(&self.urids.atom, &self.urids.time)
                >> &mut self.envelope;

            let mut pipeline = Lazy::new(|_: ()| ((), ())).compose()
                >> (complete_envelope, &mut self.sampler)
                >> Lazy::new(|(env, sample)| env * sample);

            for frame in ports.output.iter_mut() {
                *frame = pipeline.next(());
            }
        }
    }
}

lv2_descriptors!(Metro);
