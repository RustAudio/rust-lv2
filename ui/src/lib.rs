extern crate lv2_atom;
extern crate lv2_core;
extern crate lv2_sys;
extern crate urid;

pub mod plugin_ui;
pub mod port;
mod space;
pub mod uris;

use urid::*;
use uris::*;

#[derive(URIDCollection)]
pub struct UIURIDCollection {
    pub scale_factor: URID<ScaleFactor>,
    pub update_rate: URID<UpdateRate>,
}

pub mod prelude {
    pub use crate::UIURIDCollection;
    use crate::*;
    pub use lv2_sys::LV2UI_Descriptor;
    pub use lv2_ui_derive::*;
    pub use plugin_ui::*;
    pub use port::*;
}
