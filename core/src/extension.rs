/// Generate the body of a plugin's `extension_data` function.
///
/// This macro takes a URI as it's first argument, followed by a list of extension descriptors. This will
/// create a match expression that matches the given URI with the URIs of the extension descriptors. If one of the extension URIs matches, the statement returns the interface of the descriptor.
///
/// The generated statement returns a value of `Option<&'static dyn std::any::Any>`.
///
/// See the documentation of the `extension` module for more information on how to use this macro.
#[macro_export]
macro_rules! match_extensions {
    () => {
        {
            extern crate lv2_core as _lv2_core;
            //extern crate urid as _urid;

            //todo!()
            /*match ($uri).to_bytes_with_nul() {
                $(
                    <$descriptor as _urid::UriBound>::URI => Some(_lv2_core::extension::ExtensionInterface::new_for::<$descriptor>()),
                )*
                _ => None,
            }*/
        }
    };
}

// pub use crate::match_extensions;
