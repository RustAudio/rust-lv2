//! Thin but safe wrappers for the URID mapping features.

use crate::{URIDCollection, Uri, UriBound, URID};

pub trait URIDMap {
    /// Return the URID of the given URI.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and save it using a [`URIDCollection`](trait.URIDCollection.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
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
    ///     # let mut map = Box::pin(HostURIDMapper::new());
    ///     let urid: URID = map.map_uri(uri).unwrap();
    ///     assert_eq!(1, urid);
    fn map_uri(&self, uri: &Uri) -> Option<URID>;

    /// Return the URID of the given URI bound.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and collection it using a [`URIDCollection`](trait.URIDCollection.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
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
    ///     # let mut map = HostURIDMapper::new();
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

pub trait URIDUnmap {
    /// Return the URI of the given URID.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and save it using a [`URIDCollection`](trait.URIDCollection.html).
    ///
    /// # Usage example:
    ///     # #![cfg(feature = "host")]
    ///     # use lv2_urid::prelude::*;
    ///     # use lv2_urid::mapper::*;
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Using the `map` and `unmap` features provided by the host:
    ///     # let mut map = Box::pin(HostURIDMapper::new());
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     # let mut unmap = map;
    ///     let uri: &Uri = unmap.unmap(urid).unwrap();
    ///     assert_eq!(MyUriBound::uri(), uri);
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri>;
}
