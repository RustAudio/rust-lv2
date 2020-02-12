use crate::StateErr;
use crate::raw::*;
use atom::prelude::*;
use atom::space::*;
use core::extension::ExtensionDescriptor;
use core::prelude::*;
use std::marker::PhantomData;
use urid::prelude::*;

pub trait StoreHandle {
    /// Draft a new property.
    ///
    /// This will return a new handle to create a property. Once the property is completely written, you can commit it by calling [`commit`](#method.commit) or [`commit_all`](#method.commit_all). Then, and only then, it will be saved by the host.
    ///
    /// If you began to write a property and don't want the written things to be stored, you can discard it with [`discard`](#method.discard) or [`discard_all`](#method.discard_all).
    fn draft(&mut self, property_key: URID) -> StatePropertyWriter;

    /// Commit all created properties.
    ///
    /// This will also clear the property buffer.
    fn commit_all(&mut self) -> Result<(), StateErr>;

    /// Commit one specific property.
    ///
    /// This method returns `None` if the requested property was not marked for commit, `Some(Ok(()))` if the property was stored and `Some(Err(_))` if an error occured while storing the property.
    fn commit(&mut self, key: URID) -> Option<Result<(), StateErr>>;

    /// Discard all drafted properties.
    fn discard_all(&mut self);

    /// Discard a drafted property.
    ///
    /// If no property with the given key was drafted before, this is a no-op.
    fn discard(&mut self, key: URID);
}

pub struct StatePropertyWriter<'a> {
    head: SpaceHead<'a>,
}

impl<'a> StatePropertyWriter<'a> {
    pub fn new(head: SpaceHead<'a>) -> Self {
        Self { head }
    }

    pub fn init<'b, A: Atom<'a, 'b>>(
        &'b mut self,
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        (&mut self.head as &mut dyn MutSpace).init(urid, parameter)
    }
}

pub trait RetrieveHandle {
    fn retrieve(&self, key: URID) -> Option<StatePropertyReader>;
}

pub struct StatePropertyReader<'a> {
    type_: URID,
    body: Space<'a>,
}

impl<'a> StatePropertyReader<'a> {
    pub fn new(type_: URID, body: Space<'a>) -> Self {
        Self { type_, body }
    }

    pub fn read<A: Atom<'a, 'a>>(
        &self,
        urid: URID<A>,
        parameter: A::ReadParameter,
    ) -> Option<A::ReadHandle> {
        if urid == self.type_ {
            A::read(self.body, parameter)
        } else {
            None
        }
    }
}

pub trait State: Plugin {
    type StateFeatures: FeatureCollection<'static>;

    fn save(&self, store: &mut dyn StoreHandle, features: Self::StateFeatures);

    fn restore(&mut self, store: &mut dyn RetrieveHandle, features: Self::StateFeatures);
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

        let mut store = RawStoreHandle::new(store, handle);

        let mut feature_container = core::feature::FeatureContainer::from_raw(features);
        let features =
            if let Ok(features) = P::StateFeatures::from_container(&mut feature_container) {
                features
            } else {
                return sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE;
            };

        plugin.save(&mut store, features);

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

        let mut store = RawRetrieveHandle::new(retrieve, handle);

        let mut feature_container = core::feature::FeatureContainer::from_raw(features);
        let features =
            if let Ok(features) = P::StateFeatures::from_container(&mut feature_container) {
                features
            } else {
                return sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE;
            };

        plugin.restore(&mut store, features);

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
