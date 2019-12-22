//! Implementation of the mapping feature for testing purposes.
use crate::URID;
use core::{Uri, UriBuf};
use std::collections::HashMap;
use std::convert::TryInto;
use std::os::raw::*;
use std::pin::Pin;
use std::ptr::null;
use std::sync::Mutex;

/// A trait to represent an implementation of an URI <-> URID mapper, i.e. that can map an URI
/// (or any C string) to an URID, and vice-versa.
///
/// This trait allows the `Map` and `Unmap` features to be agnostic to the underlying
/// implementation, both on the plugin-side and the host-side.
///
/// # Realtime usage
/// As per the LV2 specification, please note that URID mappers are allowed to perform non-realtime
/// operations, such as memory allocation or Mutex locking.
///
/// Therefore, these methods should never be called in a realtime context (such as a plugin's
/// `run()` method). Plugins and other realtime or performance-critical contexts *should* cache IDs
/// they might need at initialization time. See the `URIDCache` for more information on how to
/// achieve this.
pub trait URIDMapper: Unpin + Sized {
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
    ///
    /// If the `map` method returns `None`, this method will return `0`.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it has to dereference a raw pointer and since it's part of the C interface.
    unsafe extern "C" fn extern_map(
        handle: crate::sys::LV2_URID_Map_Handle,
        uri: *const c_char,
    ) -> crate::sys::LV2_URID {
        match (*(handle as *const Self)).map(Uri::from_ptr(uri)) {
            Some(urid) => urid.get(),
            _ => 0,
        }
    }

    /// Create a raw map interface.
    fn make_map_interface(self: Pin<&mut Self>) -> sys::LV2_URID_Map {
        sys::LV2_URID_Map {
            handle: self.get_mut() as *mut Self as *mut c_void,
            map: Some(Self::extern_map),
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

    /// Unsafe wrapper of the `unmap` method, used by the feature interface.
    ///
    /// If the given URID is invalid or `unmap` returns `None`, this method returns a null pointer.
    ///
    /// # Safety
    ///
    /// The method is unsafe since it has to dereference raw pointers and it is part of the C interface.
    unsafe extern "C" fn extern_unmap(
        handle: crate::sys::LV2_URID_Map_Handle,
        urid: crate::sys::LV2_URID,
    ) -> *const c_char {
        match URID::new(urid) {
            Some(urid) => match (*(handle as *const Self)).unmap(urid) {
                Some(uri) => uri.as_ptr(),
                None => null(),
            },
            None => null(),
        }
    }

    /// Create an unmap interface.
    ///
    /// This method clones the mapper and creates a self-contained `UnmapInterface`.
    fn make_unmap_interface(self: Pin<&mut Self>) -> sys::LV2_URID_Unmap {
        sys::LV2_URID_Unmap {
            handle: self.get_mut() as *mut Self as *mut c_void,
            unmap: Some(Self::extern_unmap),
        }
    }
}

/// A simple URI → URID mapper, backed by a standard `HashMap` and a `Mutex` for multi-thread
/// access.
#[derive(Default)]
pub struct HashURIDMapper(Mutex<HashMap<UriBuf, URID>>);

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
