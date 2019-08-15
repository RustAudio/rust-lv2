extern crate lv2_core as core;
extern crate lv2_urid as urid;

mod test_bench {
    use core::feature::Feature;
    use std::collections::HashMap;
    use std::ffi::{c_void, CStr};
    use std::sync::Mutex;
    use urid::feature::*;

    type InternalMap = Mutex<HashMap<&'static CStr, u32>>;

    unsafe extern "C" fn internal_mapping_fn(handle: *mut c_void, uri: *const i8) -> u32 {
        let mut handle = (*(handle as *mut InternalMap)).lock().unwrap();
        let uri = CStr::from_ptr(uri);
        if !handle.contains_key(uri) {
            let new_urid = handle.len() as u32 + 1;
            handle.insert(uri, new_urid);
        }
        handle[uri]
    }

    unsafe extern "C" fn internal_unmapping_fn(handle: *mut c_void, urid: u32) -> *const i8 {
        let handle = (*(handle as *mut InternalMap)).lock().unwrap();
        for key in handle.keys() {
            if handle[key] == urid {
                return key.as_ptr();
            }
        }
        std::ptr::null()
    }

    pub struct TestBench {
        pub internal_map: Box<InternalMap>,
        pub sys_map: urid::sys::LV2_URID_Map,
        pub sys_unmap: urid::sys::LV2_URID_Unmap,
    }

    impl TestBench {
        pub fn new() -> Self {
            let mut internal_map = Box::new(Mutex::new(HashMap::new()));
            let sys_map = urid::sys::LV2_URID_Map {
                handle: internal_map.as_mut() as *mut InternalMap as *mut c_void,
                map: Some(internal_mapping_fn),
            };
            let sys_unmap = urid::sys::LV2_URID_Unmap {
                handle: internal_map.as_mut() as *mut InternalMap as *mut c_void,
                unmap: Some(internal_unmapping_fn),
            };
            Self {
                internal_map,
                sys_map,
                sys_unmap,
            }
        }

        pub fn make_map<'a>(&'a mut self) -> Map<'a> {
            Map::from_raw_data(Some(&mut self.sys_map)).unwrap()
        }

        pub fn make_unmap<'a>(&'a mut self) -> Unmap<'a> {
            Unmap::from_raw_data(Some(&mut self.sys_unmap)).unwrap()
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
    let mut test_bench = test_bench::TestBench::new();

    let map = test_bench.make_map();

    assert_eq!(1, map.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());

    assert_eq!(2, map.map_type::<MyTypeB>().unwrap());
    assert_eq!(2, map.map_uri(MyTypeB::uri()).unwrap());

    assert_eq!(1, map.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());
}

#[test]
fn test_unmap() {
    let mut test_bench = test_bench::TestBench::new();

    let (type_a, type_b) = {
        let map = test_bench.make_map();

        (
            map.map_type::<MyTypeA>().unwrap(),
            map.map_type::<MyTypeB>().unwrap(),
        )
    };

    let unmap = test_bench.make_unmap();
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
    let mut test_bench = test_bench::TestBench::new();

    let cache = {
        let map = test_bench.make_map();
        MyURIDCache::from_map(&map).unwrap()
    };

    assert_eq!(1, cache.type_a);
    assert_eq!(2, cache.type_b);
}
