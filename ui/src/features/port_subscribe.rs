use crate::port_event::PortProtocol;
use lv2_core::feature::{Feature, ThreadingClass};
use lv2_core::port::index::PortIndex;
use lv2_core::port::PortHandle;
use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use urid::{UriBound, URID};

pub struct PortSubscribe<'a> {
    inner: &'a lv2_sys::LV2UI_Port_Subscribe,
}

unsafe impl<'a> UriBound for PortSubscribe<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_UI__portSubscribe;
}

unsafe impl<'a> Feature for PortSubscribe<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        feature
            .cast::<lv2_sys::LV2UI_Port_Subscribe>()
            .as_ref()
            .map(|inner| Self { inner })
    }
}

impl<'a> PortSubscribe<'a> {
    #[inline]
    pub fn subscribe_to_port<'b, P: PortHandle, C, E: PortProtocol<'b, P>>(
        &self,
        port_index: PortIndex<P, C>,
        port_protocol: URID<E>,
    ) -> Result<(), SubscribeError> {
        let res = unsafe {
            self.inner.subscribe.unwrap()(
                self.inner.handle,
                port_index.get(),
                port_protocol.get(),
                ::core::ptr::null(),
            )
        };

        if res == 0 {
            Ok(())
        } else {
            Err(SubscribeError)
        }
    }

    #[inline]
    pub fn unsubscribe_from_port<'b, P: PortHandle, C, E: PortProtocol<'b, P>>(
        &self,
        port_index: PortIndex<P, C>,
        port_protocol: URID<E>,
    ) -> Result<(), SubscribeError> {
        let res = unsafe {
            self.inner.unsubscribe.unwrap()(
                self.inner.handle,
                port_index.get(),
                port_protocol.get(),
                ::core::ptr::null(),
            )
        };

        if res == 0 {
            Ok(())
        } else {
            Err(SubscribeError)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SubscribeError;

impl Display for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LV2 UI Resize failed")
    }
}

impl Error for SubscribeError {}
