extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

pub mod interface;
pub mod raw;

#[cfg(feature = "host")]
mod storage;
#[cfg(feature = "host")]
pub use storage::Storage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateErr {
    Unknown,
    BadCallback,
    BadData,
    BadType,
    BadFlags,
    NoFeature,
    NoProperty,
    NoSpace,
}

impl StateErr {
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

pub mod prelude {
    pub use crate::interface::{State, StateDescriptor};
    pub use crate::raw::{RetrieveHandle, StatePropertyReader, StatePropertyWriter, StoreHandle};
    pub use crate::StateErr;
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
