use lv2_atom::atoms::scalar::Int;
use lv2_core::prelude::*;
use lv2_options::collection::Options;
use lv2_options::prelude::*;
use lv2_urid::{HostMap, LV2Map};
use std::any::Any;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::pin::Pin;
use urid::{uri, HashURIDMapper, Map, Uri, UriBound};

impl lv2_options::OptionType for MyIntOption {
    type AtomType = lv2_atom::atoms::scalar::Int;

    fn from_option_value(value: &i32) -> Option<Self> {
        Some(Self(*value))
    }

    fn as_option_value(&self) -> &i32 {
        &self.0
    }
}

#[derive(FeatureCollection)]
pub struct PluginFeatures<'a> {
    map: LV2Map<'a>,
    options: OptionsList<'a>,
}

#[uri("urn:lv2_options:test:SomeIntOption")]
pub struct MyIntOption(pub i32);

#[uri("urn:lv2_options:test:OptionablePlugin")]
pub struct OptionablePlugin {
    options: Options<MyPluginOptions>,
}

#[derive(OptionsCollection)]
pub struct MyPluginOptions {
    some_int_option: MyIntOption,
}

impl OptionsInterface<'static> for OptionablePlugin {
    fn get<'a>(&'a self, mut requests: OptionRequestList<'a>) -> Result<(), OptionsError> {
        self.options.respond_to_requests(&mut requests)
    }

    fn set(&mut self, options: OptionsList) -> Result<(), OptionsError> {
        assert_eq!(self.options.values.some_int_option.0, 42);
        self.options.deserialize(&options)?;
        assert_eq!(self.options.values.some_int_option.0, 69);

        Ok(())
    }
}

impl Plugin<'static> for OptionablePlugin {
    fn new(_plugin_info: &PluginInfo, features: Self::Features) -> Option<Self> {
        let options: Options<MyPluginOptions> =
            Options::deserialize_new(&features.map, &features.options).unwrap();

        assert_eq!(options.values.some_int_option.0, 42);
        Some(OptionablePlugin { options })
    }

    type Ports = ();
    type Features = PluginFeatures<'static>;

    fn run(&mut self, _ports: &mut Self::Ports, _sample_count: u32) {
        unimplemented!()
    }

    fn extension_data(uri: &Uri) -> Option<&'static dyn Any> {
        match_extensions![uri, OptionsDescriptor<Self>]
    }
}

lv2_descriptors! {
    OptionablePlugin
}

#[test]
pub fn test_optionable_plugin() {
    use lv2_sys::*;
    use urid::UriBound;

    // Instantiating all features.
    let mut mapper: Pin<Box<HostMap<HashURIDMapper>>> = Box::pin(HashURIDMapper::new().into());
    let map_interface = Box::pin(mapper.as_mut().make_map_interface());
    let map = LV2Map::new(map_interface.as_ref().get_ref());

    let mut map_feature_interface = Box::pin(mapper.as_mut().make_map_interface());
    let map_feature = Box::pin(lv2_sys::LV2_Feature {
        URI: LV2Map::URI.as_ptr() as *const i8,
        data: map_feature_interface.as_mut().get_mut() as *mut _ as *mut c_void,
    });

    let option_value = 42;

    let option = lv2_sys::LV2_Options_Option {
        context: lv2_sys::LV2_Options_Context_LV2_OPTIONS_INSTANCE,
        subject: 0,
        key: map.map_type::<MyIntOption>().unwrap().get(),
        size: ::core::mem::size_of::<i32>() as u32,
        type_: map.map_type::<Int>().unwrap().get(),
        value: &option_value as *const _ as *const _,
    };

    let end = lv2_sys::LV2_Options_Option {
        context: 0,
        subject: 0,
        key: 0,
        size: 0,
        type_: 0,
        value: core::ptr::null(),
    };

    let options = &mut [option, end];

    let options_feature = Box::pin(lv2_sys::LV2_Feature {
        URI: OptionsList::URI.as_ptr() as *const i8,
        data: options.as_mut() as *mut _ as *mut c_void,
    });

    let features_list: &[*const lv2_sys::LV2_Feature] = &[
        map_feature.as_ref().get_ref(),
        options_feature.as_ref().get_ref(),
        std::ptr::null(),
    ];

    unsafe {
        // Retrieving the descriptor.
        let descriptor: &LV2_Descriptor = lv2_descriptor(0).as_ref().unwrap();
        let option_interface: &LV2_Options_Interface =
            descriptor.extension_data.unwrap()(lv2_sys::LV2_OPTIONS__interface.as_ptr().cast())
                .cast::<LV2_Options_Interface>()
                .as_ref()
                .unwrap();

        // Constructing the plugin.
        let plugin: LV2_Handle = (descriptor.instantiate.unwrap())(
            descriptor,
            44100.0,
            "/home/lv2/amp.lv2/\0".as_ptr() as *const c_char,
            features_list.as_ptr(),
        );
        assert_ne!(plugin, std::ptr::null_mut());

        // Activating the plugin.
        (descriptor.activate.unwrap())(plugin);

        // Getting option
        options[0].value = core::ptr::null();
        options[0].type_ = 0;
        options[0].size = 0;

        let ret = (option_interface.get.unwrap())(plugin, options.as_mut_ptr());
        assert_eq!(lv2_sys::LV2_Options_Status_LV2_OPTIONS_SUCCESS, ret);
        assert_eq!(map.map_type::<Int>().unwrap().get(), options[0].type_);
        assert_eq!(::core::mem::size_of::<i32>() as u32, options[0].size);
        assert_ne!(::core::ptr::null(), options[0].value);
        assert_eq!(42, *(options[0].value as *const i32));

        // Setting option
        let new_value = 69;
        options[0].value = &new_value as *const i32 as *const _;

        let ret = (option_interface.set.unwrap())(plugin, options.as_mut_ptr());
        assert_eq!(lv2_sys::LV2_Options_Status_LV2_OPTIONS_SUCCESS, ret);
        assert_eq!(69, *(options[0].value as *const i32));

        // Getting new option back
        options[0].value = core::ptr::null();
        options[0].type_ = 0;
        options[0].size = 0;

        let ret = (option_interface.get.unwrap())(plugin, options.as_mut_ptr());

        assert_eq!(lv2_sys::LV2_Options_Status_LV2_OPTIONS_SUCCESS, ret);
        assert_eq!(map.map_type::<Int>().unwrap().get(), options[0].type_);
        assert_eq!(::core::mem::size_of::<i32>() as u32, options[0].size);
        assert_ne!(::core::ptr::null(), options[0].value);
        assert_eq!(69, *(options[0].value as *const i32));

        // Deactivating the plugin.
        (descriptor.deactivate.unwrap())(plugin);

        // Destroying the plugin.
        (descriptor.cleanup.unwrap())(plugin)
    }
}
