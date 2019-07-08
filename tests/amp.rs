use lv2::core::plugin::port::{Audio, Control, InputPort, OutputPort};
use lv2::core::plugin::{lv2_descriptors, Plugin, PluginInfo, PortContainer};

struct Amp;

#[derive(PortContainer)]
struct AmpPorts {
    gain: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}

#[inline]
fn db_co(g: f32) -> f32 {
    if g > -90.0 {
        10f32.powf(g * 0.05)
    } else {
        0.0
    }
}

impl Plugin for Amp {
    type Ports = AmpPorts;
    type Features = ();

    #[inline]
    fn new(_plugin_info: &PluginInfo, _features: ()) -> Self {
        Amp
    }

    #[inline]
    fn run(&mut self, ports: &mut AmpPorts) {
        let coef = db_co(*(ports.gain));

        let input = ports.input.iter();
        let output = ports.output.iter_mut();

        for (input_sample, output_sample) in input.zip(output) {
            *output_sample = (*input_sample) * coef;
        }
    }
}

lv2_descriptors! {
    Amp: "http://lv2plug.in/plugins.rs/example_amp"
}
