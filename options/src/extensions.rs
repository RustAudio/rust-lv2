//! Contains the [`OptionsInterface`](crate::extensions::OptionsInterface) extension interface.
use crate::features::OptionsList;
use crate::option::request::OptionRequestList;
use crate::OptionsError;
use lv2_core::feature::Feature;
use lv2_core::plugin::PluginInstance;
use lv2_core::prelude::Plugin;
use lv2_core::prelude::{ExtensionDescriptor, ThreadingClass};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;
use urid::UriBound;

/// An interface to allow dynamically setting options from the host.
///
/// # Example
///
///
///
/// ```
/// # use lv2_core::prelude::*;
/// # use lv2_options::OptionType;
/// # use lv2_options::prelude::*;
/// #
/// # use urid::{URID, Uri, URIDCollection, uri, Map, UriBound};
/// # use std::any::Any;
/// #
/// # impl<'a> OptionType<'a> for SomeIntOption {
/// #    type AtomType = lv2_atom::scalar::Int;
/// #
/// # fn from_option_value(value: &i32) -> Option<Self> {
/// #        Some(Self(*value))
/// #    }
/// #
/// #    fn as_option_value(&'a self) -> &'a i32 {
/// #        &self.0
/// #    }
/// # }
/// #
/// # #[derive(URIDCollection)]
/// pub struct PluginUridCollection {
///     some_int_option: URID<SomeIntOption>,
///     int: URID<lv2_atom::scalar::Int>,
/// }
/// #
/// # #[derive(FeatureCollection)]
/// # pub struct PluginFeatures<'a> {
/// #    options: OptionsList<'a>,
/// # }
///
/// # #[uri("urn:lv2_options:test:OptionablePlugin")]
/// pub struct OptionablePlugin {
///     some_int: SomeIntOption,
///     urids: PluginUridCollection,
/// }
/// #
/// # impl Plugin for OptionablePlugin {
///     # type Ports = ();
///     # type InitFeatures = PluginFeatures<'static>;
///     # type AudioFeatures = ();
///    #
///     # fn new(_plugin_info: &PluginInfo, _features: &mut Self::InitFeatures) -> Option<Self> {
/// #        unimplemented!()
/// #    }
///#
/// #    fn run(
///  #       &mut self,
///  #       _ports: &mut Self::Ports,
///  #       _features: &mut Self::AudioFeatures,
///  #       _sample_count: u32,
///  #   ) {
///  #       unimplemented!()
///  #   }
///  #
///  #   fn extension_data(uri: &Uri) -> Option<&'static dyn Any> {
///  #       unimplemented!()
///  #   }
/// # }
/// #
///
/// #[uri("urn:lv2_options:test:SomeIntOption")]
/// pub struct SomeIntOption(i32);
///
/// impl OptionsInterface for OptionablePlugin {
///    fn get<'a>(&'a self, mut writer: OptionsWriter<'a>) -> Result<(), OptionsError> {
///         writer.process(|subject, options| match subject { // We will want to get/set our opions differently depending on the subject
///             Subject::Instance => { // In our case however, only our instance has an option
///                 options.handle(self.urids.some_int_option, self.urids.int, || {
///                     &self.some_int
///                 });
///             }
///             _ => {}
///         })
///     }
///
///     fn set(&mut self, options: OptionsList) -> Result<(), OptionsError> {
///         options.process(|subject, options| match subject {
///             Subject::Instance => {
///                 options.handle(self.urids.some_int_option, self.urids.int, (), |value| {
///                     self.some_int = value
///                 })
///             }
///             _ => {}
///         })
///     }
/// }
/// ```
pub trait OptionsInterface: Plugin {
    /// Allows the host to retrieve the value of the given options, as currently stored by the plugin.
    ///
    /// If the given options are unknown or somehow invalid, the appropriate [`OptionsError`] is returned.
    ///
    /// See the documentation for the [`OptionsInterface`] type for an example of how to implement this method.
    fn get(&self, requests: OptionRequestList) -> Result<(), OptionsError>;

    /// Allows the host to set the plugin's values for the given options.
    ///
    /// If the given options are unknown or somehow invalid, the appropriate [`OptionsError`] is returned.
    ///
    /// See the documentation for the [`OptionsInterface`] type for an example of how to implement this method.
    fn set(&mut self, options: OptionsList) -> Result<(), OptionsError>;
}

/// The Extension Descriptor associated to [`OptionsInterface`].
pub struct OptionsDescriptor<P: OptionsInterface> {
    plugin: PhantomData<P>,
}

unsafe impl<P: OptionsInterface> UriBound for OptionsDescriptor<P> {
    const URI: &'static [u8] = lv2_sys::LV2_OPTIONS__interface;
}

impl<P: OptionsInterface> OptionsDescriptor<P> {
    unsafe extern "C" fn get(
        instance: *mut c_void,
        options_list: *mut lv2_sys::LV2_Options_Option,
    ) -> lv2_sys::LV2_Options_Status {
        let instance = match (instance as *mut PluginInstance<P>).as_mut() {
            Some(instance) => instance,
            None => return lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
        }
        .plugin_handle();

        let options_list = match options_list.as_mut() {
            Some(options_list) => options_list,
            None => return lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
        };

        let requests = OptionRequestList::from_mut(options_list);

        match std::panic::catch_unwind(AssertUnwindSafe(|| instance.get(requests))) {
            Ok(r) => OptionsError::result_into_raw(r),
            Err(_) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
        }
    }

    unsafe extern "C" fn set(
        instance: *mut c_void,
        options_list: *const lv2_sys::LV2_Options_Option,
    ) -> lv2_sys::LV2_Options_Status {
        let instance = match (instance as *mut PluginInstance<P>).as_mut() {
            Some(instance) => instance,
            None => return lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
        }
        .plugin_handle();

        let options =
            match OptionsList::from_feature_ptr(options_list.cast(), ThreadingClass::Instantiation)
            {
                Some(options) => options,
                None => return lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
            };

        match std::panic::catch_unwind(AssertUnwindSafe(|| instance.set(options))) {
            Ok(r) => OptionsError::result_into_raw(r),
            Err(_) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
        }
    }
}

impl<P: OptionsInterface> ExtensionDescriptor for OptionsDescriptor<P> {
    type ExtensionInterface = lv2_sys::LV2_Options_Interface;
    const INTERFACE: &'static Self::ExtensionInterface = &lv2_sys::LV2_Options_Interface {
        get: Some(Self::get),
        set: Some(Self::set),
    };
}
