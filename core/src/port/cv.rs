use crate::port::{InPlacePortType, PortType};
use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;
use urid::UriBound;

/// A port connected to an array of float control values. Using this port **requires** the `inPlaceBroken` feature.
///
/// Ports of this type are connected to a buffer of float control values, represented as a slice.
/// They have the same buffer format as [`Audio`](crate::port::Audio) ports, except the buffer represents
/// audio-rate control data rather than audio.
/// Like a [`Control`](crate::port::Control) port, a CV port SHOULD have properties describing its value, in particular minimum, maximum, and default.
///
/// Hosts may present CV ports to users as controls in the same way as control ports.
/// Conceptually, aside from the buffer format, a CV port is the same as a control port, so hosts can use all the same properties and expectations.
///
/// In particular, this port type does not imply any range, unit, or meaning for its values.
/// However, if there is no inherent unit to the values, for example if the port is used to modulate some other value, then plugins SHOULD use a normalized range, either from -1.0 to 1.0, or from 0.0 to 1.0.
///
/// It is generally safe to connect an audio output to a CV input, but not vice-versa.
/// Hosts must take care to prevent data from a CVPort port from being used as audio.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#CVPort) for more information.
///
/// # Example
///
/// This very simple amplifier plugin multiplies the input sample by the input CV signal and outputs the result.
///
/// ```
/// # use lv2_core::prelude::*;
/// # use urid::*;
/// # #[uri("http://lv2plug.in/plugins.rs/simple_amp")]
/// # struct CVAmp;
/// #[derive(PortCollection)]
/// struct CVAmpPorts {
///     factor: InputPort<CV>,
///     input: InputPort<Audio>,
///     output: OutputPort<Audio>,
/// }
///
/// impl Plugin for CVAmp {
///     type Ports = CVAmpPorts;
/// # type InitFeatures = ();
/// # type AudioFeatures = ();
/// # fn new(plugin_info: &PluginInfo,features: &mut Self::InitFeatures) -> Option<Self> {
/// #         unimplemented!()
/// # }
///     // some implementation details elided…
///
///     fn run(&mut self, ports: &mut CVAmpPorts, _: &mut (), _: u32) {
///         // Input and Output dereference to `&[f32]` and `&mut [f32]`, respectively.
///         let factor = ports.factor.iter();
///
///         let input = ports.input.iter();
///         let output = ports.output.iter_mut();
///
///         for ((input_sample, output_sample), amp_factor) in input.zip(output).zip(factor) {
///             *output_sample = *input_sample * *amp_factor;
///         }
///     }
/// }
///
///
/// ```
pub struct CV;

unsafe impl UriBound for CV {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__CVPort;
}

impl PortType for CV {
    type Input = &'static [f32];
    type Output = &'static mut [f32];

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::Input {
        std::slice::from_raw_parts(pointer.as_ptr() as *const f32, sample_count as usize)
    }

    #[inline]
    unsafe fn output_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::Output {
        std::slice::from_raw_parts_mut(pointer.as_ptr() as *mut f32, sample_count as usize)
    }
}

impl InPlacePortType for CV {
    type InputOutput = (&'static [Cell<f32>], &'static [Cell<f32>]);

    #[inline]
    unsafe fn input_output_from_raw(
        input: NonNull<c_void>,
        output: NonNull<c_void>,
        sample_count: u32,
    ) -> Self::InputOutput {
        let input =
            core::slice::from_raw_parts_mut(input.as_ptr() as *mut f32, sample_count as usize);
        let output =
            core::slice::from_raw_parts_mut(output.as_ptr() as *mut f32, sample_count as usize);

        (
            Cell::from_mut(input).as_slice_of_cells(),
            Cell::from_mut(output).as_slice_of_cells(),
        )
    }
}
