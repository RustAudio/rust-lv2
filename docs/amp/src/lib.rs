// Include the prelude of `lv2`. This includes the preludes of every sub-crate and you are strongly encouraged to use it, since many macros depend on it.
use lv2::prelude::*;
// Most useful plugins will have ports for input and output data. In code, these ports are represented by a struct implementing the `PortCollection` trait. Internally, ports are referred to by index. These indices are assigned in ascending order, starting with 0 for the first port. The indices in `amp.ttl` have to match them.
#[derive(PortCollection)]
struct Ports {
    gain: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}
// Every plugin defines a struct for the plugin instance. All persistent data associated with a plugin instance is stored here, and is available to every instance method. In this simple plugin, there is no additional instance data and therefore, this struct is empty.
//
// The URI is the identifier for a plugin, and how the host associates this implementation in code with its description in data. If this URI does not match that used in the data files, the host will fail to load the plugin. This attribute internally implements the `UriBound` trait for `Amp`, which is also used to identify many other things in the Rust-LV2 ecosystem.
#[uri("https://github.com/RustAudio/rust-lv2/tree/master/docs/amp")]
struct Amp;
// Every plugin struct implements the `Plugin` trait. This trait contains both the methods that are called by the hosting application and the collection types for the ports and the used host features. This plugin does not use additional host features and therefore, we set both feature collection types to `()`. Other plugins may define separate structs with their required and optional features and set it here.
impl Plugin for Amp {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();
    // The `new` method is called by the plugin backend when it creates a new plugin instance. The host passes the plugin URI, sample rate, and bundle path for plugins that need to load additional resources (e.g. waveforms). The features parameter contains host-provided features defined in LV2 extensions, but this simple plugin does not use any. This method is in the “instantiation” threading class, so no other methods on this instance will be called concurrently with it.
    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Self)
    }
    // The `run()` method is the main process function of the plugin. It processes a block of audio in the audio context. Since this plugin is `lv2:hardRTCapable`, `run()` must be real-time safe, so blocking (e.g. with a mutex) or memory allocation are not allowed.
    fn run(&mut self, ports: &mut Ports, _features: &mut (), _: u32) {
        let coef = if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };

        for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter_mut()) {
            *out_frame = in_frame * coef;
        }
    }
}
// The `lv2_descriptors` macro creates the entry point to the plugin library. It takes structs that implement `Plugin` and exposes them. The host will load the library and call a generated function to find all the plugins defined in the library.
lv2_descriptors!(Amp);
