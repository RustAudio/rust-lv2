extern crate lv2_core as core;
extern crate lv2_urid as urid;

/// Test bench with a working URID mapper.
mod test_bench {
    use core::feature::Feature;
    use std::collections::HashMap;
    use std::ffi::{c_void, CStr};
    use std::ptr::null;
    use std::sync::{Arc, Mutex};
    use urid::*;

    /// Our URID map store.
    #[derive(Clone)]
    struct InternalMap(Arc<Mutex<HashMap<&'static CStr, u32>>>);

    impl InternalMap {
        fn new() -> Self {
            let hash_map = HashMap::new();
            let mutex = Mutex::new(hash_map);
            let arc = Arc::new(mutex);
            InternalMap(arc)
        }

        fn map(&self, uri: &'static CStr) -> u32 {
            let mut map = self.0.lock().unwrap();
            let next_urid = map.len() as u32 + 1;
            *map.entry(uri).or_insert(next_urid)
        }

        unsafe extern "C" fn extern_map(handle: *mut c_void, uri: *const i8) -> u32 {
            let handle = if let Some(handle) = (handle as *mut Self).as_ref() {
                handle
            } else {
                return 0;
            };

            if uri == null() {
                return 0;
            }
            let uri = CStr::from_ptr(uri);

            handle.map(uri)
        }

        fn unmap(&self, urid: u32) -> Option<&'static CStr> {
            let map = self.0.lock().unwrap();
            for (uri, contained_urid) in map.iter() {
                if *contained_urid == urid {
                    return Some(uri);
                }
            }
            None
        }

        unsafe extern "C" fn extern_unmap(handle: *mut c_void, urid: u32) -> *const i8 {
            let handle = if let Some(handle) = (handle as *mut Self).as_ref() {
                handle
            } else {
                return null();
            };

            handle.unmap(urid).map(|uri| uri.as_ptr()).unwrap_or(null())
        }

        pub fn map_interface(&mut self) -> urid::sys::LV2_URID_Map {
            urid::sys::LV2_URID_Map {
                handle: self as *mut _ as *mut c_void,
                map: Some(Self::extern_map),
            }
        }

        pub fn unmap_interface(&mut self) -> urid::sys::LV2_URID_Unmap {
            urid::sys::LV2_URID_Unmap {
                handle: self as *mut _ as *mut c_void,
                unmap: Some(Self::extern_unmap),
            }
        }
    }

    pub struct TestMapper {
        #[allow(dead_code)]
        map: Box<InternalMap>,
        map_interface: urid::sys::LV2_URID_Map,
        unmap_interface: urid::sys::LV2_URID_Unmap,
    }

    impl TestMapper {
        pub fn new() -> Self {
            let mut map = Box::new(InternalMap::new());
            let map_interface = map.map_interface();
            let unmap_interface = map.unmap_interface();
            Self {
                map,
                map_interface,
                unmap_interface,
            }
        }

        pub fn get_map(&mut self) -> Map {
            Map::from_raw_data(Some(&mut self.map_interface)).unwrap()
        }

        pub fn get_unmap(&mut self) -> Unmap {
            Unmap::from_raw_data(Some(&mut self.unmap_interface)).unwrap()
        }
    }
}

use core::UriBound;
use urid::*;

struct MyTypeA();

unsafe impl UriBound for MyTypeA {
    const URI: &'static [u8] = b"urn:my-type-a\0";
}

struct MyTypeB();

unsafe impl UriBound for MyTypeB {
    const URI: &'static [u8] = b"urn:my-type-b\0";
}

#[test]
fn test_map() {
    let mut test_bench = test_bench::TestMapper::new();

    let map = test_bench.get_map();

    assert_eq!(1, map.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());

    assert_eq!(2, map.map_type::<MyTypeB>().unwrap());
    assert_eq!(2, map.map_uri(MyTypeB::uri()).unwrap());

    assert_eq!(1, map.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());
}

#[test]
fn test_unmap() {
    let mut test_bench = test_bench::TestMapper::new();

    let (type_a, type_b) = {
        let map = test_bench.get_map();

        (
            map.map_type::<MyTypeA>().unwrap(),
            map.map_type::<MyTypeB>().unwrap(),
        )
    };

    let unmap = test_bench.get_unmap();
    assert_eq!(MyTypeA::uri(), unmap.unmap(type_a).unwrap());
    assert_eq!(MyTypeB::uri(), unmap.unmap(type_b).unwrap());
}

#[derive(URIDCache)]
struct MyURIDCache {
    type_a: URID<MyTypeA>,
    type_b: URID<MyTypeB>,
}

#[test]
fn test_cache() {
    let mut test_bench = test_bench::TestMapper::new();

    let cache = {
        let map = test_bench.get_map();
        MyURIDCache::from_map(&map).unwrap()
    };

    assert_eq!(1, cache.type_a);
    assert_eq!(2, cache.type_b);
}
