use crate::raw::*;
use crate::StateErr;
use core::extension::ExtensionDescriptor;
use core::prelude::*;
use std::marker::PhantomData;

/// A plugin extension that lets a plugins save and restore it's state.
///
/// This extension contains two new methods: [`save`](#tymethod.save) and [`restore`](#tymethod.restore). These are called by the host to save and restore the state of the plugin, which is done with a handle.
///
/// You can also add a feature collection to retrieve host features; It works just like the plugin's feature collection: You create a struct with multiple `Feature`s, derive `FeatureCollection` for it, and set the [`StateFeatures`](#associatedtype.StateFeatures) type to it. Then, the framework will try to populate it with the features supplied by the host and pass it to the method.
pub trait State: Plugin {
    /// The feature collection to populate for the [`save`](#tymethod.save) and [`restore`](#tymethod.restore) methods.
    type StateFeatures: FeatureCollection<'static>;

    /// Save the state of the plugin.
    ///
    /// The storage is done with the store handle. You draft a property, write it using the property handle, and then commit it to the store.
    fn save(&self, store: StoreHandle, features: Self::StateFeatures) -> Result<(), StateErr>;

    /// Restore the state of the plugin.
    ///
    /// The properties you have previously written can be retrieved with the store handle.
    fn restore(
        &mut self,
        store: RetrieveHandle,
        features: Self::StateFeatures,
    ) -> Result<(), StateErr>;
}

/// Raw wrapper of the [`State`](trait.State.html) extension.
pub struct StateDescriptor<P: State> {
    plugin: PhantomData<P>,
}

unsafe impl<P: State> UriBound for StateDescriptor<P> {
    const URI: &'static [u8] = sys::LV2_STATE__interface;
}

impl<P: State> StateDescriptor<P> {
    pub unsafe extern "C" fn extern_save(
        instance: sys::LV2_Handle,
        store: sys::LV2_State_Store_Function,
        handle: sys::LV2_State_Handle,
        flags: u32,
        features: *const *const sys::LV2_Feature,
    ) -> sys::LV2_State_Status {
        if flags & sys::LV2_State_Flags_LV2_STATE_IS_POD == 0 {
            return sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS;
        }

        let plugin: &P = if let Some(plugin) = (instance as *const P).as_ref() {
            plugin
        } else {
            return sys::LV2_State_Status_LV2_STATE_ERR_UNKNOWN;
        };

        let store = StoreHandle::new(store, handle);

        let mut feature_container = core::feature::FeatureCache::from_raw(features);
        let features = if let Ok(features) = P::StateFeatures::from_cache(&mut feature_container) {
            features
        } else {
            return sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE;
        };

        StateErr::into(plugin.save(store, features))
    }

    pub unsafe extern "C" fn extern_restore(
        instance: sys::LV2_Handle,
        retrieve: sys::LV2_State_Retrieve_Function,
        handle: sys::LV2_State_Handle,
        flags: u32,
        features: *const *const sys::LV2_Feature,
    ) -> sys::LV2_State_Status {
        if flags & sys::LV2_State_Flags_LV2_STATE_IS_POD == 0 {
            return sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS;
        }

        let plugin: &mut P = if let Some(plugin) = (instance as *mut P).as_mut() {
            plugin
        } else {
            return sys::LV2_State_Status_LV2_STATE_ERR_UNKNOWN;
        };

        let store = RetrieveHandle::new(retrieve, handle);

        let mut feature_container = core::feature::FeatureCache::from_raw(features);
        let features = if let Ok(features) = P::StateFeatures::from_cache(&mut feature_container) {
            features
        } else {
            return sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE;
        };

        StateErr::into(plugin.restore(store, features))
    }
}

impl<P: State> ExtensionDescriptor for StateDescriptor<P> {
    type ExtensionInterface = sys::LV2_State_Interface;

    const INTERFACE: &'static sys::LV2_State_Interface = &sys::LV2_State_Interface {
        save: Some(Self::extern_save),
        restore: Some(Self::extern_restore),
    };
}

#[cfg(test)]
mod tests {
    use crate::*;
    use lv2_core::prelude::*;
    use lv2_urid::prelude::*;

    struct Stateful;

    unsafe impl UriBound for Stateful {
        const URI: &'static [u8] = b"urn:null\0";
    }

    impl Plugin for Stateful {
        type Features = ();
        type Ports = ();

        #[cfg_attr(tarpaulin, skip)]
        fn new(_: &PluginInfo, _: ()) -> Option<Self> {
            Some(Self)
        }

        #[cfg_attr(tarpaulin, skip)]
        fn run(&mut self, _: &mut ()) {}
    }

    #[derive(FeatureCollection)]
    struct Features<'a> {
        _map: Map<'a>,
    }

    impl State for Stateful {
        type StateFeatures = Features<'static>;

        #[cfg_attr(tarpaulin, skip)]
        fn save(&self, _: StoreHandle, _: Features<'static>) -> Result<(), StateErr> {
            Ok(())
        }

        #[cfg_attr(tarpaulin, skip)]
        fn restore(&mut self, _: RetrieveHandle, _: Features<'static>) -> Result<(), StateErr> {
            Ok(())
        }
    }

    #[test]
    fn test_illegal_paths() {
        type Descriptor = StateDescriptor<Stateful>;
        let mut plugin = Stateful;

        assert_eq!(sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS, unsafe {
            Descriptor::extern_save(
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            )
        });

        assert_eq!(sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS, unsafe {
            Descriptor::extern_restore(
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            )
        });

        assert_eq!(sys::LV2_State_Status_LV2_STATE_ERR_UNKNOWN, unsafe {
            Descriptor::extern_save(
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                sys::LV2_State_Flags_LV2_STATE_IS_POD,
                std::ptr::null_mut(),
            )
        });

        assert_eq!(sys::LV2_State_Status_LV2_STATE_ERR_UNKNOWN, unsafe {
            Descriptor::extern_restore(
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                sys::LV2_State_Flags_LV2_STATE_IS_POD,
                std::ptr::null_mut(),
            )
        });

        assert_eq!(sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE, unsafe {
            Descriptor::extern_save(
                &mut plugin as *mut Stateful as sys::LV2_Handle,
                None,
                std::ptr::null_mut(),
                sys::LV2_State_Flags_LV2_STATE_IS_POD,
                std::ptr::null_mut(),
            )
        });

        assert_eq!(sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE, unsafe {
            Descriptor::extern_restore(
                &mut plugin as *mut Stateful as sys::LV2_Handle,
                None,
                std::ptr::null_mut(),
                sys::LV2_State_Flags_LV2_STATE_IS_POD,
                std::ptr::null_mut(),
            )
        });
    }
}
