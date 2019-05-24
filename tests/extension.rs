use lv2::core::plugin::{lv2_descriptors, Plugin, PluginInfo};
use lv2::core::{export_extension_interfaces, make_extension_interface};

pub struct AddExtensionInterface {
    pub add: unsafe extern "C" fn(*const u32, usize) -> u32,
}

pub trait AddExtension {
    fn sum(data: &[u32]) -> u32;

    unsafe extern "C" fn extern_add(data: *const u32, len: usize) -> u32 {
        let data: &[u32] = std::slice::from_raw_parts(data, len);
        Self::sum(data)
    }

    make_extension_interface![
        b"urn:MyExtension#interface\0",
        AddExtensionInterface,
        AddExtensionInterface {
            add: Self::extern_add,
        }
    ];
}

pub struct ExtendedPlugin {}

impl AddExtension for ExtendedPlugin {
    fn sum(data: &[u32]) -> u32 {
        let mut sum = 0;
        for number in data {
            sum += number
        }
        sum
    }
}

impl Plugin for ExtendedPlugin {
    type Ports = ();
    type Features = ();

    #[inline]
    fn new(_plugin_info: &PluginInfo, _features: ()) -> Self {
        ExtendedPlugin {}
    }

    #[inline]
    fn run(&mut self, _ports: &()) {}

    export_extension_interfaces![AddExtension];
}

lv2_descriptors! {
    ExtendedPlugin: "urn:extended-plugin"
}

#[test]
fn test_extension() {
    use lv2::core::sys::LV2_Descriptor;

    let descriptor: &'static LV2_Descriptor = unsafe { lv2_descriptor(0).as_ref() }.unwrap();
    let extension_data_fn = descriptor.extension_data.unwrap();

    let interface: *const std::ffi::c_void =
        unsafe { extension_data_fn(b"urn:MyExtension#interface\0".as_ptr() as *const i8) };
    let interface: &'static AddExtensionInterface =
        unsafe { (interface as *const AddExtensionInterface).as_ref() }.unwrap();

    let length: usize = 256;
    let mut data: Vec<u32> = Vec::with_capacity(length);
    for _ in 0..length {
        data.push(1);
    }

    unsafe { assert_eq!((interface.add)(data.as_ptr(), length), length as u32) };
}
