//! LV2 specification to describe position in time and passage of time, in both real and musical terms.
//!
//! The original [specification](https://lv2plug.in/ns/ext/time/time.html) contains means to describe time for LV2 values in RDF files. This implementation is focused on the stock time descriptions defined by the specification by binding them to marker types.
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

use urid::prelude::*;

/// All time properties  URI bounds
pub mod time {
    use core::UriBound;

    pub struct Bar;
    unsafe impl UriBound for Bar {
        const URI: &'static [u8] = sys::LV2_TIME__bar;
    }

    pub struct BarBeat;
    unsafe impl UriBound for BarBeat {
        const URI: &'static [u8] = sys::LV2_TIME__barBeat;
    }

    pub struct Beat;
    unsafe impl UriBound for Beat {
        const URI: &'static [u8] = sys::LV2_TIME__beat;
    }

    pub struct BeatUnit;
    unsafe impl UriBound for BeatUnit {
        const URI: &'static [u8] = sys::LV2_TIME__beatUnit;
    }

    pub struct BeatsPerBar;
    unsafe impl UriBound for BeatsPerBar {
        const URI: &'static [u8] = sys::LV2_TIME__beatsPerBar;
    }

    pub struct BeatsPerMinute;
    unsafe impl UriBound for BeatsPerMinute {
        const URI: &'static [u8] = sys::LV2_TIME__beatsPerMinute;
    }

    pub struct Frame;
    unsafe impl UriBound for Frame {
        const URI: &'static [u8] = sys::LV2_TIME__frame;
    }

    pub struct FramesPerSecond;
    unsafe impl UriBound for FramesPerSecond {
        const URI: &'static [u8] = sys::LV2_TIME__framesPerSecond;
    }

    pub struct Speed;
    unsafe impl UriBound for Speed {
        const URI: &'static [u8] = sys::LV2_TIME__speed;
    }

    pub struct Position;
    unsafe impl UriBound for Position {
        const URI: &'static [u8] = sys::LV2_TIME__position;
    }
}

use time::*;

/// A URID cache containing all units.
#[derive(URIDCache)]
pub struct TimeURIDCache {
    pub bar: URID<Bar>,
    pub bar_beat: URID<BarBeat>,
    pub beat: URID<Beat>,
    pub beat_unit: URID<BeatUnit>,
    pub beats_per_bar: URID<BeatsPerBar>,
    pub beats_per_minute: URID<BeatsPerMinute>,
    pub frame: URID<Frame>,
    pub frames_per_second: URID<FramesPerSecond>,
    pub position: URID<Position>,
    pub speed: URID<Speed>,
}

/// Prelude of `lv2_units` for wildcard usage.
pub mod prelude {
    pub use crate::time::*;
    pub use crate::TimeURIDCache;
}
