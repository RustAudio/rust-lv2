use crate::port::PortType;
use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;
use urid::UriBound;

/// A port connected to a single float ([`f32`]). Using this port **requires** the `inPlaceBroken` feature.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#ControlPort) for more information.
///
/// # Example
///
/// This very simple amplifier plugin multiplies the input sample by the input control value and outputs the result.
///
/// ```
/// # use lv2_core::prelude::*;
/// # use urid::*;
/// # #[uri("http://lv2plug.in/plugins.rs/simple_amp")]
/// # struct ControlAmp;
/// #[derive(PortCollection)]
/// struct ControlAmpPorts {
///     factor: InputPort<Control>,
///     input: InputPort<Audio>,
///     output: OutputPort<Audio>,
/// }
///
/// impl Plugin for ControlAmp {
///     type Ports = ControlAmpPorts;
/// # type InitFeatures = ();
/// # type AudioFeatures = ();
/// # fn new(plugin_info: &PluginInfo,features: &mut Self::InitFeatures) -> Option<Self> {
/// #         unimplemented!()
/// # }
///     // some implementation details elided…
///
///     fn run(&mut self, ports: &mut ControlAmpPorts, _: &mut (), _: u32) {
///         // Input and Output dereference to `&f32` and `&mut f32`, respectively.
///         let factor = *ports.factor;
///
///         let input = ports.input.iter();
///         let output = ports.output.iter_mut();
///
///         for (input_sample, output_sample) in input.zip(output) {
///             *output_sample = *input_sample * *factor;
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
/// type uses shared (`&f32`) and exclusive (`&mut f32`) references to its data, LV2 hosts
/// MUST NOT use the same buffer for both the input and the output.
/// However, do note that some hosts (Ardour, Zrythm, etc.) do not support `inPlaceBroken` plugins.
///
/// Use [`InPlaceControl`] instead if you do not want to enforce this restriction on hosts,
/// and do not need references pointing into the buffer's contents.
pub struct Control;

unsafe impl UriBound for Control {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__ControlPort;
}

impl PortType for Control {
    type InputPortType = f32;
    type OutputPortType = &'static mut f32;

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> f32 {
        *(pointer.cast().as_ref())
    }

    #[inline]
    unsafe fn output_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> &'static mut f32 {
        (pointer.as_ptr() as *mut f32).as_mut().unwrap()
    }
}

/// A port connected to a single float ([`f32`]). This port type can safely operate on shared input and output buffers.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#ControlPort) for more information.
///
/// # Example
///
/// This very simple amplifier plugin multiplies the input sample by the input control value and outputs the result.
///
/// ```
/// # use lv2_core::prelude::*;
/// # use urid::*;
/// # #[uri("http://lv2plug.in/plugins.rs/simple_amp")]
/// # struct ControlAmp;
/// #[derive(PortCollection)]
/// struct ControlAmpPorts {
///     factor: InputPort<InPlaceControl>,
///     input: InputPort<InPlaceAudio>,
///     output: OutputPort<InPlaceAudio>,
/// }
///
/// impl Plugin for ControlAmp {
///     type Ports = ControlAmpPorts;
/// # type InitFeatures = ();
/// # type AudioFeatures = ();
/// # fn new(plugin_info: &PluginInfo,features: &mut Self::InitFeatures) -> Option<Self> {
/// #         unimplemented!()
/// # }
///     // some implementation details elided…
///
///     fn run(&mut self, ports: &mut ControlAmpPorts, _: &mut (), _: u32) {
///         // Input and Output dereference to `&Cell<f32>`.
///         let factor = ports.factor.get();
///
///         let input = ports.input.iter();
///         let output = ports.output.iter();
///
///         for (input_sample, output_sample) in input.zip(output) {
///             output_sample.set(input_sample.get()) * factor;
///         }
///     }
/// }
///
///
/// ```
pub struct InPlaceControl;

unsafe impl UriBound for InPlaceControl {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__ControlPort;
}

impl PortType for InPlaceControl {
    type InputPortType = &'static Cell<f32>;
    type OutputPortType = &'static Cell<f32>;

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> Self::InputPortType {
        Cell::from_mut(&mut *(pointer.as_ptr() as *mut f32))
    }

    #[inline]
    unsafe fn output_from_raw(
        pointer: NonNull<c_void>,
        _sample_count: u32,
    ) -> Self::OutputPortType {
        Cell::from_mut(&mut *(pointer.as_ptr() as *mut f32))
    }
}
