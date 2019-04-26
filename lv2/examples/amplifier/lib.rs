use lv2::core::ports::{Audio, Control};
use lv2::core::plugin::{InputPort, OutputPort, Lv2Ports, Plugin, lv2_descriptors};

struct Amp;

#[derive(Lv2Ports)]
struct AmpPorts {
    gain: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>
}

#[inline]
fn db_co(g: f32) -> f32 {
    if g > -90.0 { 10f32.powf(g * 0.05) } else { 0.0 }
}

impl Plugin for Amp {
    type Ports = AmpPorts;
    type Features = ();

    #[inline]
    fn new(_features: ()) -> Self {
        Amp
    }

    #[inline]
    fn run(&mut self, ports: &AmpPorts) {
        let coef = db_co(ports.gain.value());

        ports.output.collect_from(ports.input.iter().map(|v| *v * coef));
    }
}

lv2_descriptors! {
    Amp: "http://lv2plug.in/plugins.rs/example_amp"
}
