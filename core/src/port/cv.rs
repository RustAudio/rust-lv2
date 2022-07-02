pub mod inplace {
    //! CV ports supporting inplace processing.
    use crate::port::{PortHandle, RCell, RwCell};
    use core::ffi::c_void;
    use core::ops::Deref;
    use core::ptr::*;

    pub struct CVInput {
        ptr: *const [RCell<f32>],
    }

    impl PortHandle for CVInput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts(pointer as *const RCell<f32>, sample_count as usize),
            })
        }
    }

    impl Deref for CVInput {
        type Target = [RCell<f32>];
        fn deref(&self) -> &[RCell<f32>] {
            unsafe { &*self.ptr }
        }
    }

    pub struct CVOutput {
        ptr: *mut [RwCell<f32>],
    }

    impl PortHandle for CVOutput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts_mut(pointer as *mut RwCell<f32>, sample_count as usize),
            })
        }
    }

    impl Deref for CVOutput {
        type Target = [RwCell<f32>];
        fn deref(&self) -> &[RwCell<f32>] {
            unsafe { &*self.ptr }
        }
    }
}

pub mod not_inplace {
    //! CV ports that doesn't support inplace processing
    use crate::port::PortHandle;
    use core::ffi::c_void;
    use core::ops::{Deref, DerefMut};
    use core::ptr::*;

    pub struct CVInput {
        ptr: *const [f32],
    }

    impl PortHandle for CVInput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts(pointer as *const f32, sample_count as usize),
            })
        }
    }

    impl Deref for CVInput {
        type Target = [f32];
        fn deref(&self) -> &[f32] {
            unsafe { &*self.ptr }
        }
    }

    pub struct CVOutput {
        ptr: *mut [f32],
    }

    impl PortHandle for CVOutput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts_mut(pointer as *mut f32, sample_count as usize),
            })
        }
    }

    impl Deref for CVOutput {
        type Target = [f32];
        fn deref(&self) -> &[f32] {
            unsafe { &*self.ptr }
        }
    }

    impl DerefMut for CVOutput {
        fn deref_mut(&mut self) -> &mut [f32] {
            unsafe { &mut *self.ptr }
        }
    }
}

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
///
/// # Safety
///
/// Using this port type requires the `inPlaceBroken` LV2 feature in your plugin. Because this port
/// type uses shared (`&[f32]`) and exclusive (`&mut [f32]`) references to its data, LV2 hosts
/// MUST NOT use the same buffer for both the input and the output.
/// However, do note that some hosts (Ardour, Zrythm, etc.) do not support `inPlaceBroken` plugins.
///
/// Use [`InPlaceCV`] instead if you do not want to enforce this restriction on hosts,
/// and do not need references pointing into the buffer's contents.
pub struct CV;

/// A port connected to an array of float control values. This port type can safely operate on shared input and output buffers.
///
/// Ports of this type are connected to a buffer of float control values, represented as a slice of [`Cell`s](std::cell::Cell).
/// They have the same buffer format as [`InPlaceAudio`](crate::port::InPlaceAudio) ports, except the buffer represents
/// audio-rate control data rather than audio.
/// Like a [`InPlaceControl`](crate::port::InPlaceControl) port, a CV port SHOULD have properties describing its value, in particular minimum, maximum, and default.
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
///     factor: InputPort<InPlaceCV>,
///     input: InputPort<InPlaceAudio>,
///     output: OutputPort<InPlaceAudio>,
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
///         // Input and Output dereference to `&[Cell<f32>]`.
///         let factor = ports.factor.iter();
///
///         let input = ports.input.iter();
///         let output = ports.output.iter();
///
///         for ((input_sample, output_sample), amp_factor) in input.zip(output).zip(factor) {
///             output_sample.set(input_sample.get() * amp_factor.get());
///         }
///     }
/// }
///
///
/// ```
pub struct InPlaceCV;
