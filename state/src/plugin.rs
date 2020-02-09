use crate::access::*;
use core::extension::ExtensionDescriptor;
use core::prelude::*;
use std::marker::PhantomData;

pub trait State: Plugin {
    type StateFeatures: FeatureCollection<'static>;

    fn save(&self, store: StoreHandle, features: Self::StateFeatures);

    fn restore(&mut self, store: RetrieveHandle, features: Self::StateFeatures);
}

pub struct StateDescriptor<P: State> {
    plugin: PhantomData<P>,
}

unsafe impl<P: State> UriBound for StateDescriptor<P> {
    const URI: &'static [u8] = sys::LV2_STATE__interface;
}

impl<P: State> StateDescriptor<P> {
    unsafe extern "C" fn extern_save(
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

        let mut feature_container = core::feature::FeatureContainer::from_raw(features);
        let features =
            if let Ok(features) = P::StateFeatures::from_container(&mut feature_container) {
                features
            } else {
                return sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE;
            };

        plugin.save(store, features);

        sys::LV2_State_Status_LV2_STATE_SUCCESS
    }

    unsafe extern "C" fn extern_restore(
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

        let mut feature_container = core::feature::FeatureContainer::from_raw(features);
        let features =
            if let Ok(features) = P::StateFeatures::from_container(&mut feature_container) {
                features
            } else {
                return sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE;
            };

        plugin.restore(store, features);

        sys::LV2_State_Status_LV2_STATE_SUCCESS
    }
}

impl<P: State> ExtensionDescriptor for StateDescriptor<P> {
    type ExtensionInterface = sys::LV2_State_Interface;

    const INTERFACE: &'static sys::LV2_State_Interface = &sys::LV2_State_Interface {
        save: Some(Self::extern_save),
        restore: Some(Self::extern_restore),
    };
}
