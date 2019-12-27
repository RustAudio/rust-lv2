//! LV2 specification for measuring unit definitions.
//!
//! The original [specification](http://lv2plug.in/ns/extensions/units/units.html) contains means to describe units for LV2 values in RDF files. This implementation is focused on the stock units defined by the specification by binding them to marker types.
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

use urid::prelude::*;

/// All unit URI bounds.
pub mod units {
    use core::UriBound;

    pub struct Bar;
    unsafe impl UriBound for Bar {
        const URI: &'static [u8] = sys::LV2_UNITS__bar;
    }

    pub struct Beat;
    unsafe impl UriBound for Beat {
        const URI: &'static [u8] = sys::LV2_UNITS__beat;
    }

    pub struct BeatPerMinute;
    unsafe impl UriBound for BeatPerMinute {
        const URI: &'static [u8] = sys::LV2_UNITS__bpm;
    }

    pub struct Cent;
    unsafe impl UriBound for Cent {
        const URI: &'static [u8] = sys::LV2_UNITS__cent;
    }

    pub struct Centimeter;
    unsafe impl UriBound for Centimeter {
        const URI: &'static [u8] = sys::LV2_UNITS__cm;
    }

    pub struct Coefficient;
    unsafe impl UriBound for Coefficient {
        const URI: &'static [u8] = sys::LV2_UNITS__coef;
    }

    pub struct Decibel;
    unsafe impl UriBound for Decibel {
        const URI: &'static [u8] = sys::LV2_UNITS__db;
    }

    pub struct Degree;
    unsafe impl UriBound for Degree {
        const URI: &'static [u8] = sys::LV2_UNITS__degree;
    }

    pub struct Frame;
    unsafe impl UriBound for Frame {
        const URI: &'static [u8] = sys::LV2_UNITS__frame;
    }

    pub struct Hertz;
    unsafe impl UriBound for Hertz {
        const URI: &'static [u8] = sys::LV2_UNITS__hz;
    }

    pub struct Inch;
    unsafe impl UriBound for Inch {
        const URI: &'static [u8] = sys::LV2_UNITS__inch;
    }

    pub struct Kilohertz;
    unsafe impl UriBound for Kilohertz {
        const URI: &'static [u8] = sys::LV2_UNITS__khz;
    }

    pub struct Kilometer;
    unsafe impl UriBound for Kilometer {
        const URI: &'static [u8] = sys::LV2_UNITS__km;
    }

    pub struct Meter;
    unsafe impl UriBound for Meter {
        const URI: &'static [u8] = sys::LV2_UNITS__m;
    }

    pub struct Megahertz;
    unsafe impl UriBound for Megahertz {
        const URI: &'static [u8] = sys::LV2_UNITS__mhz;
    }

    pub struct MIDINote;
    unsafe impl UriBound for MIDINote {
        const URI: &'static [u8] = sys::LV2_UNITS__midiNote;
    }

    pub struct Mile;
    unsafe impl UriBound for Mile {
        const URI: &'static [u8] = sys::LV2_UNITS__mile;
    }

    pub struct Minute;
    unsafe impl UriBound for Minute {
        const URI: &'static [u8] = sys::LV2_UNITS__min;
    }

    pub struct Millimeter;
    unsafe impl UriBound for Millimeter {
        const URI: &'static [u8] = sys::LV2_UNITS__mm;
    }

    pub struct Millisecond;
    unsafe impl UriBound for Millisecond {
        const URI: &'static [u8] = sys::LV2_UNITS__ms;
    }

    pub struct Octave;
    unsafe impl UriBound for Octave {
        const URI: &'static [u8] = sys::LV2_UNITS__oct;
    }

    pub struct Percent;
    unsafe impl UriBound for Percent {
        const URI: &'static [u8] = sys::LV2_UNITS__pc;
    }

    pub struct Second;
    unsafe impl UriBound for Second {
        const URI: &'static [u8] = sys::LV2_UNITS__s;
    }

    pub struct Semitone;
    unsafe impl UriBound for Semitone {
        const URI: &'static [u8] = sys::LV2_UNITS__semitone12TET;
    }
}

use units::*;

/// A URID cache containing all units.
#[derive(URIDCache)]
pub struct UnitURIDCache {
    pub bar: URID<Bar>,
    pub beat: URID<Beat>,
    pub bpm: URID<BeatPerMinute>,
    pub cent: URID<Cent>,
    pub cm: URID<Centimeter>,
    pub coef: URID<Coefficient>,
    pub db: URID<Decibel>,
    pub degree: URID<Degree>,
    pub frame: URID<Frame>,
    pub hz: URID<Hertz>,
    pub inch: URID<Inch>,
    pub khz: URID<Kilohertz>,
    pub km: URID<Kilometer>,
    pub m: URID<Meter>,
    pub mhz: URID<Megahertz>,
    pub note: URID<MIDINote>,
    pub mile: URID<Mile>,
    pub min: URID<Minute>,
    pub mm: URID<Millimeter>,
    pub ms: URID<Millisecond>,
    pub octave: URID<Octave>,
    pub percent: URID<Percent>,
    pub s: URID<Second>,
    pub semitone: URID<Semitone>,
}

/// Prelude of `lv2_units` for wildcard usage.
pub mod prelude {
    pub use crate::units::*;
    pub use crate::UnitURIDCache;
}
