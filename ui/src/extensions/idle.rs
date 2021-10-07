use lv2_core::extension::ExtensionDescriptor;
use std::marker::PhantomData;
use std::os::raw::c_int;
use urid::UriBound;

pub trait Idle {
    fn idle(&self) -> IdleStatus;
}

pub enum IdleStatus {
    Continue,
    WindowClosed,
}

pub struct IdleDescriptor<U: Idle> {
    ui: PhantomData<U>,
}

unsafe impl<P: Idle> UriBound for IdleDescriptor<P> {
    const URI: &'static [u8] = lv2_sys::LV2_UI__idleInterface;
}

impl<P: Idle> IdleDescriptor<P> {
    unsafe extern "C" fn idle(ui: lv2_sys::LV2UI_Handle) -> c_int {
        let ui = match ui.cast::<P>().as_ref() {
            Some(ui) => ui,
            None => return -1,
        };

        match ui.idle() {
            IdleStatus::Continue => 0,
            IdleStatus::WindowClosed => 1,
        }
    }
}

impl<U: Idle> ExtensionDescriptor for IdleDescriptor<U> {
    type ExtensionInterface = lv2_sys::LV2UI_Idle_Interface;
    const INTERFACE: &'static Self::ExtensionInterface = &lv2_sys::LV2UI_Idle_Interface {
        idle: Some(Self::idle),
    };
}
