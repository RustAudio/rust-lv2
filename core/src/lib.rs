extern crate lv2_sys as sys;

pub mod extension;
pub mod prelude;
pub mod plugin {
    pub use lv2_core_derive::*;
    pub trait Foobaring {
        fn foo(&self);
    }
}
