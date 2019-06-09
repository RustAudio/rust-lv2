use lv2_core::lv2_extensions;
use lv2_core::plugin::{lv2_descriptors, Plugin, PluginInfo, PortContainer};

// This test uses the `lv2_foo` and `lv2_bar` fictional LV2 specifications for extension testing.

mod lv2_foo {
    use lv2_core::extension::Extension;
    use lv2_core::plugin::Plugin;
    use lv2_core::uri::UriBound;
    use std::any::Any;
    use std::os::raw::c_void;

    pub trait FooExtension {
        fn foo(&self) -> f32;
    }

    // The `-sys` sub-crate
    pub mod sys {
        use std::os::raw::c_void;
        #[derive(Copy, Clone)]
        pub struct LV2FooInterface {
            pub foo: unsafe extern "C" fn(handle: *mut c_void) -> f32,
        }
    }

    unsafe extern "C" fn foo_ext_impl<P: FooExtension>(handle: *mut c_void) -> f32 {
        (&*(handle as *const P)).foo()
    }

    unsafe impl UriBound for FooExtension {
        const URI: &'static [u8] = b"foo\0";
    }

    unsafe impl<P: Plugin + FooExtension> Extension<P> for FooExtension {
        const RAW_DATA: &'static Any = &sys::LV2FooInterface {
            foo: foo_ext_impl::<P>,
        };
    }
}

mod lv2_bar {
    use lv2_core::extension::Extension;
    use lv2_core::plugin::Plugin;
    use lv2_core::uri::UriBound;
    use std::any::Any;
    use std::os::raw::c_void;

    // The `-sys` sub-crate
    pub mod sys {
        use std::os::raw::c_void;

        #[derive(Copy, Clone)]
        pub struct LV2BarInterface {
            // This is the _sys crate usually
            pub bar: unsafe extern "C" fn(handle: *mut c_void) -> i32,
        }
    }

    unsafe extern "C" fn bar_ext_impl<P: BarExtension>(handle: *mut c_void) -> i32 {
        (&*(handle as *const P)).bar()
    }

    pub trait BarExtension {
        fn bar(&self) -> i32;
    }

    unsafe impl UriBound for BarExtension {
        const URI: &'static [u8] = b"bar\0";
    }

    unsafe impl<P: Plugin + BarExtension> Extension<P> for BarExtension {
        const RAW_DATA: &'static Any = &sys::LV2BarInterface {
            bar: bar_ext_impl::<P>,
        };
    }
}

// Test plugin

struct TestPlugin;

#[derive(PortContainer)]
struct TestPorts {}

impl Plugin for TestPlugin {
    type Ports = TestPorts;
    type Features = ();

    lv2_extensions![lv2_foo::FooExtension, lv2_bar::BarExtension];

    fn new(_plugin_info: &PluginInfo, _features: ()) -> Self {
        Self
    }
    fn run(&mut self, _ports: &TestPorts) {}
}

lv2_descriptors! {
    TestPlugin: "http://lv2plug.in/plugins.rs/example_amp"
}

// Extension implementations

impl lv2_foo::FooExtension for TestPlugin {
    fn foo(&self) -> f32 {
        42.0
    }
}

impl lv2_bar::BarExtension for TestPlugin {
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
        as *const lv2_foo::sys::LV2FooInterface;

    assert_eq!(
        unsafe { (foo_descriptor.as_ref().unwrap().foo)(instance) },
        42f32
    );

    let bar_descriptor = unsafe { (*descriptor).extension_data.unwrap()(b"bar\0" as *const _ as _) }
        as *const lv2_bar::sys::LV2BarInterface;

    assert_eq!(
        unsafe { (bar_descriptor.as_ref().unwrap().bar)(instance) },
        69i32
    );

    // This descriptor does not exist
    let baz_descriptor =
        unsafe { (*descriptor).extension_data.unwrap()(b"baz\0" as *const _ as _) };

    assert!(unsafe { baz_descriptor.as_ref() }.is_none());
}
