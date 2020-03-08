//! Thin but safe wrappers for the URID mapping features.

use crate::{URIDCollection, URID};
use core::feature::Feature;
use core::prelude::*;
use std::ffi::c_void;
use std::os::raw::c_char;

/// Host feature to map URIs to integers
#[repr(transparent)]
pub struct Map<'a> {
    internal: &'a sys::LV2_URID_Map,
}

pub trait URIDMap {
    /// Return the URID of the given URI.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and save it using a [`URIDCollection`](trait.URIDCollection.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_core::prelude::*;
    ///     # use lv2_urid::prelude::*;
    ///     # use lv2_urid::mapper::*;
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
    ///     # let mut mapper = Box::pin(HostURIDMapper::new());
    ///     # let host_map = mapper.as_mut().make_map_interface();
    ///     # let map = Map::new(&host_map);
    ///     let urid: URID = map.map_uri(uri).unwrap();
    ///     assert_eq!(1, urid);
    fn map_uri(&self, uri: &Uri) -> Option<URID>;

    /// Return the URID of the given URI bound.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and collection it using a [`URIDCollection`](trait.URIDCollection.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_core::prelude::*;
    ///     # use lv2_urid::prelude::*;
    ///     # use lv2_urid::mapper::*;
    ///     # use std::ffi::CStr;
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Use the `map` feature provided by the host:
    ///     # let mut mapper = Box::pin(HostURIDMapper::new());
    ///     # let host_map = mapper.as_mut().make_map_interface();
    ///     # let map = Map::new(&host_map);
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     assert_eq!(1, urid);
    fn map_type<T: UriBound + ?Sized>(&self) -> Option<URID<T>> {
        self.map_uri(T::uri())
            .map(|urid| unsafe { URID::new_unchecked(urid.get()) })
    }

    /// Populate a URID collection.
    ///
    /// This is basically an alias for `T::from_map(self)` that makes the derive macro for `URIDCollection` easier.
    fn populate_collection<T: URIDCollection>(&self) -> Option<T> {
        T::from_map(self)
    }
}

unsafe impl<'a> UriBound for Map<'a> {
    const URI: &'static [u8] = sys::LV2_URID_MAP_URI;
}

unsafe impl<'a> Feature for Map<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        match class {
            ThreadingClass::Audio => {
                panic!("The URID mapping feature isn't allowed in the audio threading class")
            }
            _ => (feature as *const sys::LV2_URID_Map)
                .as_ref()
                .map(|internal| Self { internal }),
        }
    }
}

impl<'a> Map<'a> {
    pub fn new(internal: &'a sys::LV2_URID_Map) -> Self {
        Self { internal }
    }
}

impl<'a> URIDMap for Map<'a> {
    fn map_uri(&self, uri: &Uri) -> Option<URID> {
        let uri = uri.as_ptr();
        let urid = unsafe { (self.internal.map.unwrap())(self.internal.handle, uri) };
        URID::new(urid)
    }

    fn map_type<T: UriBound + ?Sized>(&self) -> Option<URID<T>> {
        let handle = self.internal.handle;
        let uri = T::URI.as_ptr() as *const c_char;
        let urid = unsafe { (self.internal.map?)(handle, uri) };
        if urid == 0 {
            None
        } else {
            Some(unsafe { URID::new_unchecked(urid) })
        }
    }
}

pub trait URIDUnmap {
    /// Return the URI of the given URID.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and save it using a [`URIDCollection`](trait.URIDCollection.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_core::prelude::*;
    ///     # use lv2_urid::prelude::*;
    ///     # use lv2_urid::mapper::*;
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Using the `map` and `unmap` features provided by the host:
    ///     # let mut mapper = Box::pin(HostURIDMapper::new());
    ///     # let host_map = mapper.as_mut().make_map_interface();
    ///     # let host_unmap = mapper.as_mut().make_unmap_interface();
    ///     # let map = Map::new(&host_map);
    ///     # let unmap = Unmap::new(&host_unmap);
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     let uri: &Uri = unmap.unmap(urid).unwrap();
    ///     assert_eq!(MyUriBound::uri(), uri);
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri>;
}

/// Host feature to revert the URI -> URID mapping.
#[repr(transparent)]
pub struct Unmap<'a> {
    internal: &'a sys::LV2_URID_Unmap,
}

unsafe impl<'a> UriBound for Unmap<'a> {
    const URI: &'static [u8] = sys::LV2_URID_UNMAP_URI;
}

unsafe impl<'a> Feature for Unmap<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        match class {
            ThreadingClass::Audio => {
                panic!("The URID unmapping feature isn't allowed in the audio threading class")
            }
            _ => (feature as *const sys::LV2_URID_Unmap)
                .as_ref()
                .map(|internal| Self { internal }),
        }
    }
}

impl<'a> Unmap<'a> {
    pub fn new(internal: &'a sys::LV2_URID_Unmap) -> Self {
        Self { internal }
    }
}

impl<'a> URIDUnmap for Unmap<'a> {
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri> {
        let uri_ptr = unsafe { (self.internal.unmap.unwrap())(self.internal.handle, urid.get()) };
        if uri_ptr.is_null() {
            None
        } else {
            Some(unsafe { Uri::from_ptr(uri_ptr) })
        }
    }
}
