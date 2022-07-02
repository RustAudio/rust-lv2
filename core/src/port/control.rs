pub mod inplace {
    //! Control ports supporting inplace processing
    use crate::port::{PortHandle, RCell, RwCell};
    use core::ffi::c_void;
    use core::ops::Deref;

    pub struct ControlInput {
        ptr: *const RCell<f32>,
    }

    impl PortHandle for ControlInput {
        unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: pointer as *const RCell<f32>,
            })
        }
    }

    impl Deref for ControlInput {
        type Target = RCell<f32>;
        fn deref(&self) -> &RCell<f32> {
            unsafe { &*self.ptr }
        }
    }

    pub struct ControlOutput {
        ptr: *mut RwCell<f32>,
    }

    impl PortHandle for ControlOutput {
        unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: pointer as *mut RwCell<f32>,
            })
        }
    }

    impl Deref for ControlOutput {
        type Target = RwCell<f32>;
        fn deref(&self) -> &RwCell<f32> {
            unsafe { &*self.ptr }
        }
    }
}

pub mod not_inplace {
    //! Control ports that doesn't support inplace processing
    use crate::port::PortHandle;
    use core::ffi::c_void;
    use core::ops::{Deref, DerefMut};

    pub struct ControlInput {
        ptr: *const f32,
    }

    impl PortHandle for ControlInput {
        unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: pointer as *const f32,
            })
        }
    }

    impl Deref for ControlInput {
        type Target = f32;
        fn deref(&self) -> &f32 {
            unsafe { &*self.ptr }
        }
    }

    pub struct ControlOutput {
        ptr: *mut f32,
    }

    impl PortHandle for ControlOutput {
        unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: pointer as *mut f32,
            })
        }
    }

    impl Deref for ControlOutput {
        type Target = f32;
        fn deref(&self) -> &f32 {
            unsafe { &*self.ptr }
        }
    }

    impl DerefMut for ControlOutput {
        fn deref_mut(&mut self) -> &mut f32 {
            unsafe { &mut *self.ptr }
        }
    }
}

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
///             *output_sample = *input_sample * factor;
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
///             output_sample.set(input_sample.get() * factor);
///         }
///     }
/// }
///
///
/// ```
pub struct InPlaceControl;
