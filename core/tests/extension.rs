use lv2_core::extension::Extension;
use lv2_core::lv2_extensions;
use lv2_core::plugin::{lv2_descriptors, Plugin, PluginInfo, PortContainer};
use lv2_core::uri::UriBound;
use std::any::Any;
use std::os::raw::c_void;

// Test extensions

trait FooExtension {
    fn foo(&self) -> f32;
}

#[derive(Copy, Clone)]
struct LV2FooInterface {
    // This is the _sys crate usually
    foo: unsafe extern "C" fn(handle: *mut c_void) -> f32,
}

unsafe extern "C" fn foo_ext_impl<P: FooExtension>(handle: *mut c_void) -> f32 {
    (&*(handle as *const P)).foo()
}

unsafe impl UriBound for FooExtension {
    const URI: &'static [u8] = b"foo\0";
}

unsafe impl<P: Plugin + FooExtension> Extension<P> for FooExtension {
    const RAW_DATA: &'static Any = &LV2FooInterface {
        foo: foo_ext_impl::<P>,
    };
}

#[derive(Copy, Clone)]
struct LV2BarInterface {
    // This is the _sys crate usually
    bar: unsafe extern "C" fn(handle: *mut c_void) -> i32,
}

unsafe extern "C" fn bar_ext_impl<P: BarExtension>(handle: *mut c_void) -> i32 {
    (&*(handle as *const P)).bar()
}

trait BarExtension {
    fn bar(&self) -> i32;
}

unsafe impl UriBound for BarExtension {
    const URI: &'static [u8] = b"bar\0";
}

unsafe impl<P: Plugin + BarExtension> Extension<P> for BarExtension {
    const RAW_DATA: &'static Any = &LV2BarInterface {
        bar: bar_ext_impl::<P>,
    };
}

// Test plugin

struct TestPlugin;

#[derive(PortContainer)]
struct TestPorts {}

impl Plugin for TestPlugin {
    type Ports = TestPorts;
    type Features = ();

    lv2_extensions![FooExtension, BarExtension];

    fn new(_plugin_info: &PluginInfo, _features: ()) -> Self {
        Self
    }
    fn run(&mut self, _ports: &TestPorts) {}
}

lv2_descriptors! {
    TestPlugin: "http://lv2plug.in/plugins.rs/example_amp"
}

// Extension implementations

impl FooExtension for TestPlugin {
    fn foo(&self) -> f32 {
        42.0
    }
}

impl BarExtension for TestPlugin {
    fn bar(&self) -> i32 {
        69
    }
}

#[test]
fn extensions_work() {
    let descriptor: *const lv2_core::sys::LV2_Descriptor = unsafe { lv2_descriptor(0) };

    let instance = unsafe {
        (*descriptor).instantiate.unwrap()(
            descriptor,
            44_100.0,
            b"foo_dir\0" as *const _ as _,
            ::std::ptr::null(),
        )
    };

    let foo_descriptor = unsafe { (*descriptor).extension_data.unwrap()(b"foo\0" as *const _ as _) }
        as *const LV2FooInterface;

    assert_eq!(
        unsafe { (foo_descriptor.as_ref().unwrap().foo)(instance) },
        42f32
    );

    let bar_descriptor = unsafe { (*descriptor).extension_data.unwrap()(b"bar\0" as *const _ as _) }
        as *const LV2BarInterface;

    assert_eq!(
        unsafe { (bar_descriptor.as_ref().unwrap().bar)(instance) },
        69i32
    );

    // This descriptor does not exist
    let baz_descriptor = unsafe { (*descriptor).extension_data.unwrap()(b"baz\0" as *const _ as _) }
        as *const LV2BarInterface;

    assert!(unsafe { baz_descriptor.as_ref() }.is_none());
}
