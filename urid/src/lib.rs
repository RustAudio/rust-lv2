//! Library for idiomatic URID support.
//!
//! In the world of [RDF](https://en.wikipedia.org/wiki/Resource_Description_Framework), resources are described using [URIs](https://en.wikipedia.org/wiki/Uniform_Resource_Identifier). In Rust, this concept can be adapted to describe types with URIs using the [`UriBound`](trait.UriBound.html) trait. Then, other crates might use these URIs to describe relationships between types or values using URIs.
//!
//! However, comparing URIs isn't necessarily fast. Therefore, another concept was introduced: The [URID](struct.URID.html). A URID is basically a `u32` which represents a URI. These URIDs are assigned by a [`Map`](trait.Map.html) and can be "dereferenced" by an [`Unmap`](trait.Unmap.html).
//!
//! This library also supports connecting URIDs to their `UriBound` via a generics argument. This can be used, for example, to request the URID of a certain bound as a parameter of a function. If someone would try to call this function with the wrong URID, the compiler will raise an error before the code is even compiled.
//!
//! This may seem a bit minor to you now, but the audio plugin framework [rust-lv2](https://github.com/RustAudio/rust-lv2) heavily relies on this crate for fast, portable and dynamic data identification and exchange.
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::sync::Mutex;

pub use urid_derive::*;

/// Representation of a borrowed Uri.
pub type Uri = ::std::ffi::CStr;
/// Representation of an owned Uri.
pub type UriBuf = ::std::ffi::CString;

/// A trait for types that can be identified by a URI.
///
/// Every type that can be identified by a URI implements this trait, which makes the retrieval of these URIs as easy as the following:
///
///     use urid::UriBound;
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

/// Representation of a URI for fast comparisons.
///
/// A URID is basically a number which represents a URI, which makes the identification of other features faster and easier. The mapping of URIs to URIDs is handled by a something that implements the [`Map`](trait.Map.html) trait. A given URID can also be converted back to a URI with an implementation of the [`Unmap`](trait.Unmap.html) trait. However, these implementations should obviously be linked.
///
/// This struct has an optional type parameter `T` which defaults to `()`. In this case, the type can represent any URID at all, but if `T` is a `UriBound`, the instance of `URID<T>` can only describe the URID of the given bound. This makes creation easier and also turns it into an atomic [`URIDCollection`](trait.URIDCollection.html), which can be used to build bigger collections.
#[repr(transparent)]
pub struct URID<T = ()>(NonZeroU32, PhantomData<T>)
where
    T: ?Sized;

/// A store of pre-mapped URIDs
///
/// This trait can be used to easily cache URIDs. The usual way of creating such a collection is to define a struct of `URID<T>`s, where `T` implements `UriBound`, and then using the derive macro to implement `URIDCollection` for it. Then, you can populate it with a map and access it any time, even in a real-time-sensitive context.
///
/// # Usage example:
///
///     # use urid::*;
///     // Defining all URI bounds.
///     struct MyTypeA;
///     
///     unsafe impl UriBound for MyTypeA {
///         const URI: &'static [u8] = b"urn:my-type-a\0";
///     }
///     
///     struct MyTypeB;
///     
///     unsafe impl UriBound for MyTypeB {
///         const URI: &'static [u8] = b"urn:my-type-b\0";
///     }
///
///     // Defining the collection.
///     #[derive(URIDCollection)]
///     struct MyCollection {
///         my_type_a: URID<MyTypeA>,
///         my_type_b: URID<MyTypeB>,
///     }
///
///     // Creating a mapper and collecting URIDs.
///     let map = HashURIDMapper::new();
///     let collection = MyCollection::from_map(&map).unwrap();
///
///     // Asserting.
///     assert_eq!(1, collection.my_type_a);
///     assert_eq!(2, collection.my_type_b);
pub trait URIDCollection: Sized {
    /// Construct the collection from the mapper.
    fn from_map<M: Map + ?Sized>(map: &M) -> Option<Self>;
}

impl URID<()> {
    /// Creates a new URID from a raw number.
    ///
    /// URIDs may never be zero. If the given number is zero, `None` is returned.
    #[inline]
    pub fn new(raw_urid: u32) -> Option<Self> {
        NonZeroU32::new(raw_urid).map(|inner| Self(inner, PhantomData))
    }
}

impl<T: ?Sized> URID<T> {
    /// Create a URID without checking for type or value validity.
    ///
    /// This value may only be a URID the mapper actually produced and that is recognised by a compatible unmapper. Therefore, it should only be used by [`Map::map_uri`](trait.Map.html#tymethod.map_uri) or [`Map::map_type`](trait.Map.html#method.map_type).
    ///
    /// # Safety
    ///
    /// A URID may not be 0 since this value is reserved for the `None` value of `Option<URID<T>>`, which therefore has the same size as a `URID<T>`. If `T` is also a URI bound, the URID may only be the one that is mapped to the bounded URI.
    ///
    /// Since these constraints aren't checked by this method, it is unsafe. Using this method is technically sound as long as `raw_urid` is not zero, but might still result in bad behaviour if its the wrong URID for the bound `T`.
    pub unsafe fn new_unchecked(raw_urid: u32) -> Self {
        Self(NonZeroU32::new_unchecked(raw_urid), PhantomData)
    }

    /// Return the raw URID number.
    pub fn get(self) -> u32 {
        self.0.get()
    }

    /// Transform the type-specific URID into a generalized one.
    pub fn into_general(self) -> URID<()> {
        unsafe { URID::new_unchecked(self.get()) }
    }
}

impl<T: UriBound + ?Sized> URIDCollection for URID<T> {
    fn from_map<M: Map + ?Sized>(map: &M) -> Option<Self> {
        map.map_type()
    }
}

impl<T: ?Sized> fmt::Debug for URID<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized> Clone for URID<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T: ?Sized> Copy for URID<T> {}

impl<T1: ?Sized, T2: ?Sized> PartialEq<URID<T1>> for URID<T2> {
    fn eq(&self, other: &URID<T1>) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized> PartialEq<u32> for URID<T> {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl<T: ?Sized> PartialEq<URID<T>> for u32 {
    fn eq(&self, other: &URID<T>) -> bool {
        *self == other.get()
    }
}

impl<T: ?Sized> Eq for URID<T> {}

impl<T1: ?Sized, T2: ?Sized> PartialOrd<URID<T1>> for URID<T2> {
    fn partial_cmp(&self, other: &URID<T1>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: ?Sized> PartialOrd<u32> for URID<T> {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl<T: ?Sized> PartialOrd<URID<T>> for u32 {
    fn partial_cmp(&self, other: &URID<T>) -> Option<Ordering> {
        self.partial_cmp(&other.get())
    }
}

impl<T: ?Sized> Ord for URID<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: ?Sized> Hash for URID<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl std::convert::TryFrom<u32> for URID {
    type Error = ();

    fn try_from(value: u32) -> Result<URID, ()> {
        if value == 0 {
            Err(())
        } else {
            Ok(unsafe { URID::new_unchecked(value) })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::URID;

    #[test]
    fn test_urid_size() {
        use std::mem::size_of;

        let size = size_of::<u32>();

        assert_eq!(size, size_of::<URID>());
        assert_eq!(size, size_of::<Option<URID>>());
    }
}

/// A handle to map URIs to URIDs.
pub trait Map {
    /// Maps an URI to an `URID` that corresponds to it.
    ///
    /// If the URI has not been mapped before, a new URID will be assigned.
    ///
    /// # Errors
    /// This method may return `None` in the exceptional case that an ID for that URI could not be
    /// created for whatever reason.
    /// However, implementations SHOULD NOT return `None` from this function in non-exceptional
    /// circumstances (i.e. the URI map SHOULD be dynamic).
    ///
    /// # Realtime usage
    /// This action may not be realtime-safe since it may involve locking mutexes or allocating dynamic memory. If you are working in a realtime environment, you should cache mapped URIDs in a [`URIDCollection`](trait.URIDCollection.html) and use it instead.
    fn map_uri(&self, uri: &Uri) -> Option<URID>;

    /// Retrieve the URI of the bound and map it to a URID.
    ///
    /// The rules of [`map_uri`](#tymethod.map_uri) apply here too.
    fn map_type<T: UriBound + ?Sized>(&self) -> Option<URID<T>> {
        self.map_uri(T::uri())
            .map(|urid| unsafe { URID::new_unchecked(urid.get()) })
    }

    /// Populate a URID collection.
    ///
    /// This is basically an alias for [`T::from_map(self)`](trait.URIDCollection.html#tymethod.from_map) that simplifies the derive macro for `URIDCollection`.
    fn populate_collection<T: URIDCollection>(&self) -> Option<T> {
        T::from_map(self)
    }
}

/// A handle to map URIDs to URIs.
pub trait Unmap {
    /// Get the URI of a previously mapped URID.
    ///
    /// This method may return `None` if the given `urid` is not mapped to URI yet.
    ///
    /// # Realtime usage
    /// This action may not be realtime-safe since it may involve locking mutexes or allocating dynamic memory. If you are working in a realtime environment, you should cache mapped URIDs in a [`URIDCollection`](trait.URIDCollection.html) and use it instead.
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri>;
}

/// A simple URI → URID mapper, backed by a standard `HashMap` and a `Mutex` for multi-thread
/// access.
#[derive(Default)]
pub struct HashURIDMapper(Mutex<HashMap<UriBuf, URID>>);

impl Map for HashURIDMapper {
    fn map_uri(&self, uri: &Uri) -> Option<URID<()>> {
        let mut map = self.0.lock().ok()?; // Fail if the Mutex got poisoned
        match map.get(uri) {
            Some(urid) => Some(*urid),
            None => {
                let map_length: u32 = map.len().try_into().ok()?; // Fail if there are more items into the HashMap than an u32 can hold
                let next_urid = map_length.checked_add(1)?; // Fail on overflow when adding 1 for the next URID

                // This is safe, because we just added 1 to the length and checked for overflow, therefore the number can never be 0.
                let next_urid = unsafe { URID::new_unchecked(next_urid) };
                map.insert(uri.into(), next_urid);
                Some(next_urid)
            }
        }
    }
}

impl Unmap for HashURIDMapper {
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri> {
        let map = self.0.lock().ok()?;
        for (uri, contained_urid) in map.iter() {
            if *contained_urid == urid {
                // Here we jump through some hoops to return a reference that bypasses the mutex.
                // This is safe because the only way this reference might become invalid is if an
                // entry gets overwritten, which is not something that we allow through this
                // interface.
                return Some(unsafe {
                    let bytes = uri.as_bytes_with_nul();
                    Uri::from_bytes_with_nul_unchecked(std::slice::from_raw_parts(
                        bytes.as_ptr(),
                        bytes.len(),
                    ))
                });
            }
        }

        None
    }
}

impl HashURIDMapper {
    /// Create a new URID map store.
    pub fn new() -> Self {
        Default::default()
    }
}
