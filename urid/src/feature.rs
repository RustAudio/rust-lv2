//! Thin but safe wrappers for the URID mapping features.

use crate::{URIDCache, URID};
use core::feature::Feature;
use core::Uri;
use core::UriBound;
use std::marker::PhantomData;
use std::os::raw::c_char;

/// Host feature to map URIs to integers
pub struct Map<'a> {
    internal: sys::LV2_URID_Map,
    _mapper_lifetime: PhantomData<&'a ()>,
}

unsafe impl<'a> UriBound for Map<'a> {
    const URI: &'static [u8] = sys::LV2_URID_MAP_URI;
}

unsafe impl<'a> Feature<'a> for Map<'a> {}

#[cfg(feature = "host")]
unsafe extern "C" fn urid_map<T: crate::mapper::URIDMapper>(
    handle: crate::sys::LV2_URID_Map_Handle,
    uri: *const c_char,
) -> crate::sys::LV2_URID {
    let result = ::std::panic::catch_unwind(|| {
        (&*(handle as *const T))
            .map(::std::ffi::CStr::from_ptr(uri))
            .map(URID::get)
    });

    match result {
        Ok(Some(urid)) => urid,
        _ => 0, // FIXME: mapper panics should not be silenced
    }
}

impl<'a> Map<'a> {
    #[cfg(feature = "host")]
    pub fn new<T: crate::mapper::URIDMapper>(mapper: &'a T) -> Map<'a> {
        Map {
            _mapper_lifetime: PhantomData,
            internal: crate::sys::LV2_URID_Map {
                handle: mapper as *const T as *const _ as *mut _,
                map: Some(urid_map::<T>),
            },
        }
    }

    /// Return the URID of the given URI.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it using a [`URIDCache`](trait.URIDCache.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_core::prelude::*;
    ///     # use lv2_urid::prelude::*;
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Creating the URI and mapping it to its URID.
    ///     let uri = Uri::from_bytes_with_nul(b"http://lv2plug.in\0").unwrap();
    ///
    ///     // Use the `map` feature provided by the host:
    ///     # let mapper = lv2_urid::mapper::HashURIDMapper::new();
    ///     # let map = lv2_urid::Map::new(&mapper);
    ///     let urid: URID = map.map_uri(uri).unwrap();
    ///     assert_eq!(1, urid);
    pub fn map_uri(&self, uri: &Uri) -> Option<URID> {
        let uri = uri.as_ptr();
        let urid = unsafe { (self.internal.map.unwrap())(self.internal.handle, uri) };
        URID::new(urid)
    }

    /// Return the URID of the given URI bound.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it using a [`URIDCache`](trait.URIDCache.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_core::prelude::*;
    ///     # use lv2_urid::prelude::*;
    ///     # use std::ffi::CStr;
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Use the `map` feature provided by the host:
    ///     # let mapper = lv2_urid::mapper::HashURIDMapper::new();
    ///     # let map = lv2_urid::Map::new(&mapper);
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     assert_eq!(1, urid);
    pub fn map_type<T: UriBound + ?Sized>(&self) -> Option<URID<T>> {
        let handle = self.internal.handle;
        let uri = T::URI.as_ptr() as *const c_char;
        let urid = unsafe { (self.internal.map?)(handle, uri) };
        if urid == 0 {
            None
        } else {
            Some(unsafe { URID::new_unchecked(urid) })
        }
    }

    /// Populate a URID cache.
    ///
    /// This is basically an alias for `T::from_map(self)` that makes the derive macro for `URIDCache` easier.
    pub fn populate_cache<T: URIDCache>(&self) -> Option<T> {
        T::from_map(self)
    }
}

/// Host feature to revert the URI -> URID mapping.
pub struct Unmap<'a> {
    internal: sys::LV2_URID_Unmap,
    _mapper_lifetime: PhantomData<&'a ()>,
}

unsafe impl<'a> UriBound for Unmap<'a> {
    const URI: &'static [u8] = sys::LV2_URID_UNMAP_URI;
}

unsafe impl<'a> Feature<'a> for Unmap<'a> {}

#[cfg(feature = "host")]
unsafe extern "C" fn urid_unmap<T: crate::mapper::URIDMapper>(
    handle: crate::sys::LV2_URID_Map_Handle,
    urid: crate::sys::LV2_URID,
) -> *const c_char {
    let urid = match URID::new(urid) {
        None => return ::std::ptr::null(),
        Some(urid) => urid,
    };

    let result = ::std::panic::catch_unwind(|| (&*(handle as *const T)).unmap(urid));

    match result {
        Ok(Some(uri)) => uri.as_ptr(),
        _ => ::std::ptr::null(), // FIXME: mapper panics should not be silenced
    }
}

impl<'a> Unmap<'a> {
    #[cfg(feature = "host")]
    pub fn new<T: crate::mapper::URIDMapper>(mapper: &'a T) -> Unmap<'a> {
        Unmap {
            _mapper_lifetime: PhantomData,
            internal: crate::sys::LV2_URID_Unmap {
                handle: mapper as *const T as *const _ as *mut _,
                unmap: Some(urid_unmap::<T>),
            },
        }
    }

    /// Return the URI of the given URID.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it using a [`URIDCache`](trait.URIDCache.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_core::prelude::*;
    ///     # use lv2_urid::prelude::*;
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Using the `map` and `unmap` features provided by the host:
    ///     # let mapper = lv2_urid::mapper::HashURIDMapper::new();
    ///     # let map = lv2_urid::Map::new(&mapper);
    ///     # let unmap = lv2_urid::Unmap::new(&mapper);
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     let uri: &Uri = unmap.unmap(urid).unwrap();
    ///     assert_eq!(MyUriBound::uri(), uri);
    pub fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri> {
        let uri_ptr = unsafe { (self.internal.unmap.unwrap())(self.internal.handle, urid.get()) };
        if uri_ptr.is_null() {
            None
        } else {
            Some(unsafe { Uri::from_ptr(uri_ptr) })
        }
    }
}
