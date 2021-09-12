use lv2_sys::LV2_Atom_Event__bindgen_ty_1;
use lv2_units::units::{Beat, Frame};
use urid::UriBound;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct TimestampBody(LV2_Atom_Event__bindgen_ty_1);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SequenceUnitType {
    Beat,
    Frame,
}

pub trait SequenceUnit: UriBound + private::Sealed {
    type Value: Copy + PartialEq + PartialOrd + 'static;

    const TYPE: SequenceUnitType;

    #[doc(hidden)]
    unsafe fn convert_from_raw(raw: sys::LV2_Atom_Event__bindgen_ty_1) -> Self::Value;

    #[doc(hidden)]
    fn convert_into_raw(value: Self::Value) -> TimestampBody;
}

impl SequenceUnit for Beat {
    type Value = f64;
    const TYPE: SequenceUnitType = SequenceUnitType::Beat;

    #[inline]
    unsafe fn convert_from_raw(raw: LV2_Atom_Event__bindgen_ty_1) -> Self::Value {
        raw.beats
    }

    #[inline]
    fn convert_into_raw(value: Self::Value) -> TimestampBody {
        TimestampBody(LV2_Atom_Event__bindgen_ty_1 { beats: value })
    }
}

impl SequenceUnit for Frame {
    type Value = i64;
    const TYPE: SequenceUnitType = SequenceUnitType::Frame;

    #[inline]
    unsafe fn convert_from_raw(raw: LV2_Atom_Event__bindgen_ty_1) -> Self::Value {
        raw.frames
    }

    #[inline]
    fn convert_into_raw(value: Self::Value) -> TimestampBody {
        TimestampBody(LV2_Atom_Event__bindgen_ty_1 { frames: value })
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for Beat {}
    impl Sealed for Frame {}
}
