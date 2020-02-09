use crate::feature::*;
use core::UriBound;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::NonZeroU32;

/// Representation of a URI for fast comparisons.
///
/// A URID is basically a number which represents a URI, which makes the identification of other features faster and easier. The mapping of URIs to URIDs is handled by the host and plugins can retrieve them using the [`Map`](struct.Map.html) feature. A given URID can also be converted back to a URI with the [`Unmap`](struct.Unmap.html) feature.
///
/// This struct has an optional type parameter `T` which defaults to `()`. In this case, the type can represent any URID at all, but if `T` is a `UriBound`, the type can only describe the URID of the given bound. This makes creation easier and also turns it into an atomic [`URIDCache`](trait.URIDCache.html), which can be used to build bigger caches.
#[repr(transparent)]
pub struct URID<T = ()>(NonZeroU32, PhantomData<T>)
where
    T: ?Sized;

/// Abstraction of types that store URIDs.
///
/// This trait makes the creation of static URID caches easy: You simply define the cache and derive `URIDCache` for it, and you have a single method to create it.
///
/// # Usage example:
///
///     # #![cfg(feature = "host")]
///     # use lv2_core::prelude::*;
///     # use lv2_urid::prelude::*;
///     # use lv2_urid::mapper::*;
///     # use std::ffi::CStr;
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
///     // Defining the cache.
///     #[derive(URIDCache)]
///     struct MyCache {
///         my_type_a: URID<MyTypeA>,
///         my_type_b: URID<MyTypeB>,
///     }
///
///     # let mut mapper = Box::pin(HashURIDMapper::new());
///     # let host_map = mapper.as_mut().make_map_interface();
///     # let host_unmap = mapper.as_mut().make_unmap_interface();
///     # let map = Map::new(&host_map);
///     # let unmap = Unmap::new(&host_unmap);
///     // Populating the cache, Using the `map` and `unmap` features provided by the host:
///     let cache = MyCache::from_map(&map).unwrap();
///
///     // Asserting.
///     assert_eq!(1, cache.my_type_a);
///     assert_eq!(2, cache.my_type_b);
pub trait URIDCache: Sized {
    /// Construct the cache from the mapper.
    fn from_map(map: &Map) -> Option<Self>;
}

impl URID<()> {
    /// Creates a new URID from a raw number.
    ///
    /// URID may never be zero. If the given number is zero, `None` is returned.
    #[inline]
    pub fn new(raw_urid: u32) -> Option<Self> {
        NonZeroU32::new(raw_urid).map(|inner| Self(inner, PhantomData))
    }
}

impl<T: ?Sized> URID<T> {
    /// Create a URID without checking for type or value validity.
    ///
    /// First of all, the value may only be a URID the host actually recognizes. Therefore, it should only be used by [`Map::map_uri`](struct.Map.html#method.map_uri) or [`Map::map_type`](struct.Map.html#method.map_type), after the raw mapping function was called.
    ///
    /// Additionally, the value of 0 is reserved for a failed URI mapping process and therefore, is not a valid URID. If `T` is a URI bound, the URID may only be the one the host maps the bounded URI.
    ///
    /// Since all of these constraints are not checked by this method, it is unsafe.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it assumes that `raw_urid` is not zero. Using this method is sound as long as `raw_urid` is not zero.
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

impl<T: UriBound + ?Sized> URIDCache for URID<T> {
    fn from_map(map: &Map) -> Option<Self> {
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
