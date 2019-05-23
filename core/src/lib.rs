pub extern crate lv2_core_sys as sys;

mod feature;

pub mod plugin;
pub mod port;
pub mod uri;

pub use self::feature::*;
pub use self::port::*;

#[macro_export]
macro_rules! make_extension_interface {
    ($uri:expr, $interface:ty, $instance:expr) => {
        fn extension_uri() -> &'static ::lv2_core::uri::Uri {
            const URI: &[u8] = $uri;
            unsafe { ::lv2_core::uri::Uri::from_bytes_with_nul_unchecked(URI) }
        }

        const INTERFACE: $interface = $instance;
    };
}

#[macro_export]
macro_rules! export_extension_interfaces {
    ($uri:expr, $($extension:ident),*) => {
        $(
        if <Self as $extension>::extension_uri() == $uri {
            return Some(&<Self as $extension>::INTERFACE);
        }
        )*
    }
}
