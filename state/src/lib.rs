//! Extension for LV2 plugins to store their state.
//!
//! This is a rather classic extension to LV2 plugins: There is a trait called [`State`](trait.State.html) which requires the methods [`save`](trait.State.html#tymethod.save) and [`restore`](trait.State.html#tymethiod.restore) to be implemented. These methods will be called by the host to save and restore the state of the plugin.
//!
//! ## Example usage
//!
//! ```
//! use lv2_atom::prelude::*;
//! use lv2_core::prelude::*;
//! use lv2_state::*;
//! use lv2_urid::prelude::*;
//!
//! /// A plugin that stores a float value.
//! struct Stateful {
//!     internal: f32,
//!     urids: AtomURIDCollection,
//! }
//!
//! /// `Stateful`s implementation of `State`.
//! impl State for Stateful {
//!     type StateFeatures = ();
//!
//!     fn save(&self, mut store: StoreHandle, _: ()) -> Result<(), StateErr> {
//!         // Try to draft a new property and store the float inside it.
//!         store
//!             .draft(URID::new(1000).unwrap())
//!             .init(self.urids.float, self.internal)?;
//!
//!         // Commit the written property.
//!         // Otherwise, it will discarded.
//!         store.commit_all()
//!     }
//!
//!     fn restore(&mut self, store: RetrieveHandle, _: ()) -> Result<(), StateErr> {
//!         // Try to restore the property.
//!         self.internal = store
//!             .retrieve(URID::new(1000).unwrap())?
//!             .read(self.urids.float, ())?;
//!
//!         // We're done.
//!         Ok(())
//!     }
//! }
//!
//! impl Plugin for Stateful {
//!     type Ports = ();
//!     type InitFeatures = Features<'static>;
//!     type AudioFeatures = ();
//!
//!     fn new(_: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
//!         Some(Stateful {
//!             internal: 42.0,
//!             urids: features.map.populate_collection()?,
//!         })
//!     }
//!
//!     fn run(&mut self, _: &mut (), _: &mut ()) {
//!         // Set the float to a different value than the previous one.
//!         self.internal += 1.0;
//!     }
//!
//!     fn extension_data(uri: &Uri) -> Option<&'static dyn std::any::Any> {
//!         // Export the store extension. Otherwise, the host won't use it.
//!         match_extensions!(uri, StateDescriptor<Self>)
//!     }
//! }
//!
//! #[derive(FeatureCollection)]
//! pub struct Features<'a> {
//!     map: Map<'a>,
//! }
//!
//! unsafe impl UriBound for Stateful {
//!     const URI: &'static [u8] = b"urn:lv2_atom:stateful\0";
//! }
//! ```
extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

mod interface;
pub use interface::*;

mod raw;
pub use raw::*;

mod storage;
pub use storage::Storage;

/// Kinds of errors that may occur in the crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateErr {
    /// The kind of the error is unknown or doesn't have a representation.
    Unknown,
    /// A callback function pointer of a method is bad.
    BadCallback,
    /// Retrieved data is invalid.
    BadData,
    /// The retrieved data doesn't have the correct type.
    BadType,
    /// The flags a method was called with are invalid.
    BadFlags,
    /// A feature the plugin requested is missing.
    NoFeature,
    /// A property the plugin is requesting doesn't exist.
    NoProperty,
    /// There isn't enough memory available to execute the task.
    NoSpace,
}

impl StateErr {
    /// Convert a raw status flag to a result or possible error value.
    pub fn from(value: u32) -> Result<(), StateErr> {
        match value {
            sys::LV2_State_Status_LV2_STATE_SUCCESS => Ok(()),
            sys::LV2_State_Status_LV2_STATE_ERR_BAD_TYPE => Err(StateErr::BadType),
            sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS => Err(StateErr::BadFlags),
            sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE => Err(StateErr::NoFeature),
            sys::LV2_State_Status_LV2_STATE_ERR_NO_PROPERTY => Err(StateErr::NoProperty),
            sys::LV2_State_Status_LV2_STATE_ERR_NO_SPACE => Err(StateErr::NoSpace),
            _ => Err(StateErr::Unknown),
        }
    }

    /// Convert a result to a raw status flag.
    pub fn into(result: Result<(), StateErr>) -> u32 {
        match result {
            Ok(()) => sys::LV2_State_Status_LV2_STATE_SUCCESS,
            Err(StateErr::BadType) => sys::LV2_State_Status_LV2_STATE_ERR_BAD_TYPE,
            Err(StateErr::BadFlags) => sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS,
            Err(StateErr::NoFeature) => sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE,
            Err(StateErr::NoProperty) => sys::LV2_State_Status_LV2_STATE_ERR_NO_PROPERTY,
            Err(StateErr::NoSpace) => sys::LV2_State_Status_LV2_STATE_ERR_NO_SPACE,
            Err(_) => sys::LV2_State_Status_LV2_STATE_ERR_UNKNOWN,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::StateErr;

    #[test]
    fn test_state_conversion() {
        assert_eq!(
            Ok(()),
            StateErr::from(sys::LV2_State_Status_LV2_STATE_SUCCESS)
        );
        assert_eq!(
            Err(StateErr::BadType),
            StateErr::from(sys::LV2_State_Status_LV2_STATE_ERR_BAD_TYPE)
        );
        assert_eq!(
            Err(StateErr::BadFlags),
            StateErr::from(sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS)
        );
        assert_eq!(
            Err(StateErr::NoFeature),
            StateErr::from(sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE)
        );
        assert_eq!(
            Err(StateErr::NoProperty),
            StateErr::from(sys::LV2_State_Status_LV2_STATE_ERR_NO_PROPERTY)
        );
        assert_eq!(
            Err(StateErr::NoSpace),
            StateErr::from(sys::LV2_State_Status_LV2_STATE_ERR_NO_SPACE)
        );
        assert_eq!(Err(StateErr::Unknown), StateErr::from(std::u32::MAX));

        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_SUCCESS,
            StateErr::into(Ok(()))
        );
        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_ERR_BAD_TYPE,
            StateErr::into(Err(StateErr::BadType))
        );
        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_ERR_BAD_FLAGS,
            StateErr::into(Err(StateErr::BadFlags))
        );
        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_ERR_NO_FEATURE,
            StateErr::into(Err(StateErr::NoFeature))
        );
        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_ERR_NO_PROPERTY,
            StateErr::into(Err(StateErr::NoProperty))
        );
        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_ERR_NO_SPACE,
            StateErr::into(Err(StateErr::NoSpace))
        );
        assert_eq!(
            sys::LV2_State_Status_LV2_STATE_ERR_UNKNOWN,
            StateErr::into(Err(StateErr::Unknown))
        );
    }
}
