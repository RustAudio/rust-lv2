extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

pub mod interface;
pub mod raw;
#[cfg(feature = "host")]
pub mod storage;

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
