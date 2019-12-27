use lv2_core::feature::{FeatureCollection, FeatureContainer, MissingFeatureError};
use lv2_core::feature::{HardRTCapable, IsLive};
use lv2_core::prelude::*;
use std::ops::Drop;
use std::os::raw::c_char;

struct Amp {
    activated: bool,
}

unsafe impl UriBound for Amp {
    const URI: &'static [u8] = b"http://lv2plug.in/plugins.rs/example_amp\0";
}

#[derive(PortContainer)]
struct AmpPorts {
    gain: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}

#[derive(FeatureCollection)]
struct Features {
    _rt_capable: HardRTCapable,
    is_live: Option<IsLive>,
}

impl Plugin for Amp {
    type Ports = AmpPorts;
    type Features = Features;

    #[inline]
    fn new(plugin_info: &PluginInfo, features: Features) -> Option<Self> {
        // Verifying the plugin info.
        assert_eq!(
            plugin_info.plugin_uri().to_str().unwrap(),
            "http://lv2plug.in/plugins.rs/example_amp"
        );
        assert_eq!(
            plugin_info.bundle_path().to_str().unwrap(),
            "/home/lv2/amp.lv2/"
        );
        assert_eq!(plugin_info.sample_rate() as u32, 44100);

        // Finding and verifying all features.
        assert!(features.is_live.is_none());

        Some(Amp { activated: false })
    }

    fn activate(&mut self) {
        assert!(!self.activated);
        self.activated = true;
    }

    #[inline]
    fn run(&mut self, ports: &mut AmpPorts) {
        assert!(self.activated);

        let coef = *(ports.gain);

        let input = ports.input.iter();
        let output = ports.output.iter_mut();

        for (input_sample, output_sample) in input.zip(output) {
            *output_sample = (*input_sample) * coef;
        }
    }

    fn deactivate(&mut self) {
        assert!(self.activated);
        self.activated = false;
    }
}

impl Drop for Amp {
    fn drop(&mut self) {
        assert!(!self.activated);
    }
}

lv2_descriptors! {
    Amp
}

#[test]
fn test_discovery() {
    use lv2_sys::*;

    unsafe {
        let descriptor: &LV2_Descriptor = lv2_descriptor(0).as_ref().unwrap();
        assert_eq!(
            Uri::from_ptr(descriptor.URI),
            Uri::from_bytes_with_nul_unchecked(b"http://lv2plug.in/plugins.rs/example_amp\0")
        );
        assert_eq!(lv2_descriptor(1), std::ptr::null());
    }
}

#[test]
fn test_plugin() {
    use lv2_core::UriBound;
    use lv2_sys::*;

    // Creating the ports.
    let mut gain: f32 = 2.0;
    let mut input: Box<[f32; 128]> = Box::new([0.0; 128]);
    for i in 0..128 {
        input[i] = i as f32;
    }
    let mut output: Box<[f32; 128]> = Box::new([0.0; 128]);

    // Creating the hard-rt feature.
    let hard_rt_capable = LV2_Feature {
        URI: HardRTCapable::URI.as_ptr() as *const c_char,
        data: std::ptr::null_mut(),
    };
    let features: &[*const LV2_Feature] = &[&hard_rt_capable, std::ptr::null()];

    unsafe {
        // Retrieving the descriptor.
        let descriptor: &LV2_Descriptor = lv2_descriptor(0).as_ref().unwrap();

        // Constructing the plugin.
        let plugin: LV2_Handle = (descriptor.instantiate.unwrap())(
            descriptor,
            44100.0,
            "/home/lv2/amp.lv2/\0".as_ptr() as *const c_char,
            features.as_ptr(),
        );
        assert_ne!(plugin, std::ptr::null_mut());

        // Connecting the ports.
        let connect_port = descriptor.connect_port.unwrap();
        (connect_port)(plugin, 0, (&mut gain) as *mut f32 as *mut _);
        (connect_port)(plugin, 1, input.as_mut_ptr() as *mut _);
        (connect_port)(plugin, 2, output.as_mut_ptr() as *mut _);

        // Activating the plugin.
        (descriptor.activate.unwrap())(plugin);

        // Running the plugin.
        (descriptor.run.unwrap())(plugin, 128);

        // Deactivating the plugin.
        (descriptor.deactivate.unwrap())(plugin);

        // Destroying the plugin.
        (descriptor.cleanup.unwrap())(plugin)
    }

    // Verifying the data.
    for i in 0..128 {
        assert!((input[i] * gain - output[i]).abs() < std::f32::EPSILON);
    }
}
