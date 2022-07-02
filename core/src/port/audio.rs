//! TODO!
//! Audio ports definition. Audio ports are connected to an array of float audio samples.
//!
//! Ports of this type are connected to a buffer of float audio samples, represented as a slice.
//!
//! Audio samples are normalized between -1.0 and 1.0, though there is no requirement for samples to be strictly within this range.
//!
//! See the [LV2 reference](https://lv2plug.in/ns/lv2core#AudioPort) for more information.
//!
//! # Example
//!
//! This very simple amplifier plugin multiplies the input sample by 2 and outputs the result.
//!
//! ```
//! # use lv2_core::prelude::*;
//! # use urid::*;
//! # #[uri("http://lv2plug.in/plugins.rs/simple_amp")]
//! # struct SimpleAmp;
//! #[derive(PortCollection)]
//! struct SimpleAmpPorts {
//!     input: InputPort<Audio>,
//!     output: OutputPort<Audio>,
//! }
//!
//! impl Plugin for SimpleAmp {
//!     type Ports = SimpleAmpPorts;
//! # type InitFeatures = ();
//! # type AudioFeatures = ();
//! # fn new(plugin_info: &PluginInfo,features: &mut Self::InitFeatures) -> Option<Self> {
//! #         unimplemented!()
//! # }
//!     // some implementation details elided…
//!
//!     fn run(&mut self, ports: &mut SimpleAmpPorts, _: &mut (), _: u32) {
//!         // Input and Output dereference to `&[f32]` and `&mut [f32]`, respectively.
//!         let input = ports.input.iter();
//!         let output = ports.output.iter_mut();
//!
//!         for (input_sample, output_sample) in input.zip(output) {
//!             *output_sample = *input_sample * 2.0;
//!         }
//!     }
//! }
//!
//!
//! ```

pub mod inplace {
    //! Audio ports supporting inplace processing.
    use crate::port::{PortHandle, RCell, RwCell};
    use core::ffi::c_void;
    use core::ops::Deref;
    use core::ptr::*;

    pub struct AudioInput {
        ptr: *const [RCell<f32>],
    }

    impl PortHandle for AudioInput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts(pointer as *const RCell<f32>, sample_count as usize),
            })
        }
    }

    impl Deref for AudioInput {
        type Target = [RCell<f32>];
        fn deref(&self) -> &[RCell<f32>] {
            unsafe { &*self.ptr }
        }
    }

    pub struct AudioOutput {
        ptr: *mut [RwCell<f32>],
    }

    impl PortHandle for AudioOutput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts_mut(pointer as *mut RwCell<f32>, sample_count as usize),
            })
        }
    }

    impl Deref for AudioOutput {
        type Target = [RwCell<f32>];
        fn deref(&self) -> &[RwCell<f32>] {
            unsafe { &*self.ptr }
        }
    }
}

pub mod not_inplace {
    //! Audio ports that doesn't support inplace processing
    use crate::port::PortHandle;
    use core::ffi::c_void;
    use core::ops::{Deref, DerefMut};
    use core::ptr::*;

    pub struct AudioInput {
        ptr: *const [f32],
    }

    impl PortHandle for AudioInput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts(pointer as *const f32, sample_count as usize),
            })
        }
    }

    impl Deref for AudioInput {
        type Target = [f32];
        fn deref(&self) -> &[f32] {
            unsafe { &*self.ptr }
        }
    }

    pub struct AudioOutput {
        ptr: *mut [f32],
    }

    impl PortHandle for AudioOutput {
        unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
            Some(Self {
                ptr: slice_from_raw_parts_mut(pointer as *mut f32, sample_count as usize),
            })
        }
    }

    impl Deref for AudioOutput {
        type Target = [f32];
        fn deref(&self) -> &[f32] {
            unsafe { &*self.ptr }
        }
    }

    impl DerefMut for AudioOutput {
        fn deref_mut(&mut self) -> &mut [f32] {
            unsafe { &mut *self.ptr }
        }
    }
}
