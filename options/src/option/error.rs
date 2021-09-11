use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors raised when interacting with LV2 Options.
///
/// See the [LV2 Documentation](https://lv2plug.in/doc/html/group__options.html#ga94d649a74ab340dfc6c6751dbe92ca07)
/// for more information.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum OptionsError {
    Unknown = lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN as isize,
    BadSubject = lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_SUBJECT as isize,
    BadKey = lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_KEY as isize,
    BadValue = lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_VALUE as isize,
}

impl OptionsError {
    #[inline]
    pub(crate) fn result_into_raw(value: Result<(), OptionsError>) -> lv2_sys::LV2_Options_Status {
        match value {
            Ok(()) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_SUCCESS,
            Err(OptionsError::BadSubject) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_SUBJECT,
            Err(OptionsError::BadKey) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_KEY,
            Err(OptionsError::BadValue) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_VALUE,
            Err(_) => lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_UNKNOWN,
        }
    }

    #[inline]
    pub(crate) fn from_raw(status: lv2_sys::LV2_Options_Status) -> Result<(), OptionsError> {
        match status {
            lv2_sys::LV2_Options_Status_LV2_OPTIONS_SUCCESS => Ok(()),
            lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_SUBJECT => Err(OptionsError::BadSubject),
            lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_KEY => Err(OptionsError::BadKey),
            lv2_sys::LV2_Options_Status_LV2_OPTIONS_ERR_BAD_VALUE => Err(OptionsError::BadValue),
            _ => Err(OptionsError::Unknown),
        }
    }
}

impl Display for OptionsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            OptionsError::Unknown => "Unknown error while reading/writing Option",
            OptionsError::BadSubject => "Unknown Option subject",
            OptionsError::BadKey => "Unknown Option key",
            OptionsError::BadValue => "Invalid Option value"
        };

        write!(f, "{}", msg)
    }
}

impl Error for OptionsError {}
