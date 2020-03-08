//! LV2 specification centered around the Map feature.
//!
//! The URID specification provides a host feature that can be used by plugins to map URIs to integers, so-called URIDs. These URIDs are used by many other specifications to identify other URI bounds and combine the flexibility of URIs with the comparison speed of integers.
//!
//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
extern crate lv2_sys as sys;

mod feature;
pub mod mapper;
mod urid;

pub use lv2_urid_derive::*;

pub use feature::*;
pub use urid::*;

pub type Uri = ::std::ffi::CStr;
pub type UriBuf = ::std::ffi::CString;

/// Trait for types that can be identified by a URI.
///
/// LV2 makes heavy use of URIs to identify resources. This is where this trait comes in: Every type that can be identified by a URI implements this trait, which makes the retrieval of these URIs as easy as the following:
///
///     use lv2_urid::UriBound;
///
///     // Defining the struct
///     pub struct MyStruct {
///         a: f32,
///     }
///
///     // Implementing `UriBound`
///     unsafe impl UriBound for MyStruct {
///         const URI: &'static [u8] = b"urn:my-struct\0";
///     }
///
///     // Retrieving the URI
///     assert_eq!("urn:my-struct", MyStruct::uri().to_str().unwrap());
///
/// # Unsafety
///
/// This trait is unsafe to implement since the [`URI`](#associatedconstant.URI) constant has some requirements that can not be enforced with Rust's type system.
pub unsafe trait UriBound {
    /// The URI of the type, safed as a byte slice
    ///
    /// Currently, there is no way to express a `CStr` in a constant way. Therefore, the URI has to be stored as a null-terminated byte slice.
    ///
    /// The slice must be a valid URI and must have the null character, expressed as `\0`, at the end. Otherwise, other code might produce a segmentation fault or read a faulty URI while looking for the null character.
    const URI: &'static [u8];

    /// Construct a `CStr` reference to the URI.
    ///
    /// Assuming that [`URI`](#associatedconstant.URI) is correct, this method constructs a `CStr` reference from the byte slice referenced by `URI`.
    fn uri() -> &'static Uri {
        unsafe { Uri::from_bytes_with_nul_unchecked(Self::URI) }
    }
}

/// Prelude of `lv2_urid` for wildcard usage.
pub mod prelude {
    pub use crate::feature::{URIDMap, URIDUnmap};
    pub use crate::{URIDCollection, Uri, UriBound, UriBuf, URID};
    pub use lv2_urid_derive::*;
}
