pub extern crate lv2_core_sys as sys;

mod feature;

pub mod plugin;
pub mod port;
pub mod uri;

pub use self::feature::*;
pub use self::port::*;

/// Generate export symbols for an extension trait.
///
/// The [`export_extension_interfaces`](macro.export_extension_interfaces.html) requires some symbols from a trait in order to work. This macro creates these symbols. In order to work, this macro needs the URI of the interface, the type of the interface and an expression that constructs the interface. An example on how to use this macro:
///
///     use lv2_core::make_extension_interface;
///
///     unsafe extern "C" fn add(a: u32, b: u32) -> u32 {
///         a+b
///     }
///
///     #[repr(C)]
///     struct Interface {
///         add: unsafe extern "C" fn(a: u32, b: u32) -> u32,
///     }
///
///     make_extension_interface!(b"urn:interface\0", Interface, Interface { add: add });
///
/// If you look for a proper example, take a look at the `extension` test.
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

/// Generate a implementation for a plugin's
/// [`extension_data`](plugin/trait.Plugin.html#method.extension_data) function.
///
/// This macro creates an implementation for `extension_data` that exports all extension interfaces
/// that are build with the [`make_extension_interface`](macro.make_extension_interface.html)
/// macro. Take a look at the `extension` test for an example!
#[macro_export]
macro_rules! export_extension_interfaces {
    ($($extension:ident),*) => {
        fn extension_data(uri: &::lv2_core::uri::Uri) -> Option<&'static ::std::any::Any> {
            $(if <Self as $extension>::extension_uri() == uri {
                return Some(&<Self as $extension>::INTERFACE);
            })*
            None
        }
    }
}
