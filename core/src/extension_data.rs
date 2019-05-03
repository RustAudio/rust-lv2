use crate::uri::Uri;

/// Represents extension data for a given feature, as returned by the `extension_data()` plugin's method.
/// # Unsafety
/// Since extension data is passed to plugin as a raw pointer,
/// structs implementing this trait must be `#[repr(C)]`.
pub unsafe trait ExtensionData: Sized {
    const URI: &'static [u8];

    #[inline]
    fn uri() -> &'static Uri {
        unsafe { Uri::from_bytes_unchecked(Self::URI) }
    }
}
