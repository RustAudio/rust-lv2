pub struct PortEvent;

use lv2_core::port::index::PortIndex;
use lv2_core::port::{Control, InPlaceControl, InputPort, PortHandle};
use urid::{UriBound, URID};

impl PortEvent {
    pub fn read<'a, P: PortHandle, E: PortProtocol<'a, P>>(
        &'a self,
        port_index: PortIndex<P>,
        event_type: URID<E>,
    ) -> Option<E::ReadHandle> {
        // TODO: handle event id = 0 special case
        todo!()
    }
}

pub trait PortProtocol<'handle, T: PortHandle>: UriBound {
    type ReadHandle: Sized + 'handle;
    type WriteParameter: Sized + 'handle;
}

pub struct FloatProtocol;

unsafe impl UriBound for FloatProtocol {
    const URI: &'static [u8] = lv2_sys::LV2_UI__floatProtocol;
}

impl<'a> PortProtocol<'a, InputPort<Control>> for FloatProtocol {
    type ReadHandle = f32;
    type WriteParameter = f32;
}

impl<'a> PortProtocol<'a, InputPort<InPlaceControl>> for FloatProtocol {
    type ReadHandle = f32;
    type WriteParameter = f32;
}
