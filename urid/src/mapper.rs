//! Implementation of the mapping feature for testing purposes.
use crate::URID;
use core::{Uri, UriBuf};
use std::collections::HashMap;
use std::convert::TryInto;
use std::os::raw::*;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Interface container for the map feature.
///
/// Since the `map` and `feature` fields contain raw pointers to the `mapper` and `map` fields, respectively, these have to be pinned in memory.
pub struct MapInterface<T: URIDMapper> {
    pub mapper: Pin<Box<T>>,
    pub map: Pin<Box<sys::LV2_URID_Map>>,
    pub feature: Pin<Box<core::sys::LV2_Feature>>,
}

/// Interface container for the unmap feature.
///
/// Since the `map` and `feature` fields contain raw pointers to the `mapper` and `map` fields, respectively, these have to be pinned in memory.
pub struct UnmapInterface<T: URIDMapper> {
    pub mapper: Pin<Box<T>>,
    pub unmap: Pin<Box<sys::LV2_URID_Unmap>>,
    pub feature: Pin<Box<core::sys::LV2_Feature>>,
}

/// A trait to represent an implementation of an URI <-> URID mapper, i.e. that can map an URI
/// (or any C string) to an URID, and vice-versa.
///
/// This trait allows the `Map` and `Unmap` features to be agnostic to the underlying
/// implementation, both on the plugin-side and the host-side.
///
/// # Cloning
///
/// Implementors of this trait have to be clonable, since every instance of a [`MapInterface`](struct.MapInterface.html) or [`UnmapInterface`](struct.UnmapInterface.html) has to own it's own clone of the mapper. This means that, although cloned, the different clones of the mapper have to be consistent: If one clone maps the URI `https://rustup.rs/` to the URID 42, then all of the other clones have to do that too! One way of doing that is to have a common map store contained in a Mutex and shared pointer. See the [`HashURIDMapper`](struct.HashURIDMapper.html) for an example.
///
/// # Realtime usage
/// As per the LV2 specification, please note that URID mappers are allowed to perform non-realtime
/// operations, such as memory allocation or Mutex locking.
///
/// Therefore, these methods should never be called in a realtime context (such as a plugin's
/// `run()` method). Plugins and other realtime or performance-critical contexts *should* cache IDs
/// they might need at initialization time. See the `URIDCache` for more information on how to
/// achieve this.
pub trait URIDMapper: Clone + Unpin + Sized {
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
    /// As per the LV2 specification, please note that URID mappers are allowed to perform non-realtime
    /// operations, such as memory allocation or Mutex locking.
    ///
    /// Therefore, these methods should never be called in a realtime context (such as a plugin's
    /// `run()` method). Plugins and other realtime or performance-critical contexts *should* cache IDs
    /// they might need at initialization time. See the `URIDCache` for more information on how to
    /// achieve this.
    fn map(&self, uri: &Uri) -> Option<URID>;

    /// Unsafe wrapper of the `map` method, used by the feature interface.
    unsafe extern "C" fn extern_map(
        handle: crate::sys::LV2_URID_Map_Handle,
        uri: *const c_char,
    ) -> crate::sys::LV2_URID {
        let result = ::std::panic::catch_unwind(|| {
            (&*(handle as *const Self))
                .map(::std::ffi::CStr::from_ptr(uri))
                .map(URID::get)
        });

        match result {
            Ok(Some(urid)) => urid,
            _ => 0, // FIXME: mapper panics should not be silenced
        }
    }

    /// Create a map interface.
    ///
    /// This method clones the mapper and creates a self-contained `MapInterface`.
    fn make_map_interface(&self) -> MapInterface<Self> {
        let mut mapper = Box::pin(self.clone());
        let mut map = Box::pin(sys::LV2_URID_Map {
            handle: mapper.as_mut().get_mut() as *mut Self as *mut c_void,
            map: Some(Self::extern_map),
        });
        let feature = Box::pin(core::sys::LV2_Feature {
            URI: sys::LV2_URID__map.as_ptr() as *const c_char,
            data: map.as_mut().get_mut() as *mut sys::LV2_URID_Map as *mut c_void,
        });
        MapInterface {
            mapper,
            map,
            feature,
        }
    }

    /// Gets the URId for a previously mapped `URID`.
    ///
    /// This method may return `None` if the given `urid` is not yet mapped.
    ///
    /// # Realtime usage
    /// As per the LV2 specification, please note that URID mappers are allowed to perform non-realtime
    /// operations, such as memory allocation or Mutex locking.
    ///
    /// Therefore, these methods should never be called in a realtime context (such as a plugin's
    /// `run()` method). Plugins and other realtime or performance-critical contexts *should* cache IDs
    /// they might need at initialization time. See the `URIDCache` for more information on how to
    /// achieve this.
    fn unmap(&self, urid: URID) -> Option<&Uri>;

    /// Unsafe wrapper of the `map` method, used by the feature interface.
    unsafe extern "C" fn extern_unmap(
        handle: crate::sys::LV2_URID_Map_Handle,
        urid: crate::sys::LV2_URID,
    ) -> *const c_char {
        let urid = match URID::new(urid) {
            None => return ::std::ptr::null(),
            Some(urid) => urid,
        };

        let result = ::std::panic::catch_unwind(|| (&*(handle as *const Self)).unmap(urid));

        match result {
            Ok(Some(uri)) => uri.as_ptr(),
            _ => ::std::ptr::null(), // FIXME: mapper panics should not be silenced
        }
    }

    /// Create an unmap interface.
    ///
    /// This method clones the mapper and creates a self-contained `UnmapInterface`.
    fn make_unmap_interface(&self) -> UnmapInterface<Self> {
        let mut mapper = Box::pin(self.clone());
        let mut unmap = Box::pin(sys::LV2_URID_Unmap {
            handle: mapper.as_mut().get_mut() as *mut Self as *mut c_void,
            unmap: Some(Self::extern_unmap),
        });
        let feature = Box::pin(core::sys::LV2_Feature {
            URI: sys::LV2_URID__unmap.as_ptr() as *const c_char,
            data: unmap.as_mut().get_mut() as *mut sys::LV2_URID_Unmap as *mut c_void,
        });
        UnmapInterface {
            mapper,
            unmap,
            feature,
        }
    }
}

/// A simple URI → URID mapper, backed by a standard `HashMap` and a `Mutex` for multi-thread
/// access.
#[derive(Default, Clone)]
pub struct HashURIDMapper(Arc<Mutex<HashMap<UriBuf, URID>>>);

impl URIDMapper for HashURIDMapper {
    fn map(&self, uri: &Uri) -> Option<URID<()>> {
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

    fn unmap(&self, urid: URID<()>) -> Option<&Uri> {
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
