//! LV2 specification to describe position in time and passage of time, in both real and musical
//! terms.
//!
//! The original [specification](https://lv2plug.in/ns/ext/time/time.html) contains means to
//! describe time for LV2 values in RDF files. This implementation is focused on the stock time
//! descriptions defined by the specification by binding them to marker types.
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

use urid::prelude::*;

/// All time URI bounds
///
/// All Struct suffixed by `Class` are time Classes, others are time properties.
pub mod time {
    use urid::UriBound;

    ///A point in time and/or the speed at which time is passing.
    ///
    ///A position is both a point and a speed, which precisely defines a time within a timeline.
    pub struct PositionClass;
    unsafe impl UriBound for PositionClass {
        const URI: &'static [u8] = sys::LV2_TIME__Position;
    }

    ///The rate of passage of time in terms of one unit with respect to another.
    pub struct RateClass;
    unsafe impl UriBound for RateClass {
        const URI: &'static [u8] = sys::LV2_TIME__Rate;
    }

    ///A point in time in some unit/dimension
    pub struct TimeClass;
    unsafe impl UriBound for TimeClass {
        const URI: &'static [u8] = sys::LV2_TIME__Time;
    }

    pub struct Bar;
    unsafe impl UriBound for Bar {
        const URI: &'static [u8] = sys::LV2_TIME__bar;
    }

    ///The beat number within the bar, from 0 to beatPerBar.
    pub struct BarBeat;
    unsafe impl UriBound for BarBeat {
        const URI: &'static [u8] = sys::LV2_TIME__barBeat;
    }

    ///The global running beat number.
    ///
    ///This is not the beat within a bar like barBeat, but relative
    ///to the same origin as time:bar and monotonically increases unless the transport is
    ///repositioned.
    pub struct Beat;
    unsafe impl UriBound for Beat {
        const URI: &'static [u8] = sys::LV2_TIME__beat;
    }

    ///Beat unit, the note value that counts as one beat.
    ///
    ///This is the bottom number in a time signature: 2 for half note, 4 for quarter note, and so
    ///on.
    pub struct BeatUnit;
    unsafe impl UriBound for BeatUnit {
        const URI: &'static [u8] = sys::LV2_TIME__beatUnit;
    }

    ///Beats per bar
    ///
    ///This is the top number in a time signature
    pub struct BeatsPerBar;
    unsafe impl UriBound for BeatsPerBar {
        const URI: &'static [u8] = sys::LV2_TIME__beatsPerBar;
    }

    ///Tempo in beats per minute.
    pub struct BeatsPerMinute;
    unsafe impl UriBound for BeatsPerMinute {
        const URI: &'static [u8] = sys::LV2_TIME__beatsPerMinute;
    }

    pub struct Frame;
    unsafe impl UriBound for Frame {
        const URI: &'static [u8] = sys::LV2_TIME__frame;
    }

    ///Frame per second
    ///
    ///Frame rate in frames per second.
    pub struct FramesPerSecond;
    unsafe impl UriBound for FramesPerSecond {
        const URI: &'static [u8] = sys::LV2_TIME__framesPerSecond;
    }

    ///The rate of the progress of time as a fraction of normal speed.
    ///
    ///For example, a rate of 0.0
    ///is stopped, 1.0 is rolling at normal speed, 0.5 is rolling at half speed, -1.0 is reverse,
    ///and so on.
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

/// A URID cache containing all time properties.
#[derive(URIDCollection)]
pub struct TimeURIDCollection {
    pub position_class: URID<PositionClass>,
    pub rate_class: URID<RateClass>,
    pub time_class: URID<TimeClass>,
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

/// Prelude of `lv2_time` for wildcard usage.
pub mod prelude {
    pub use crate::time::*;
    pub use crate::TimeURIDCollection;
}
