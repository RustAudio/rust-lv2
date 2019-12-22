#![cfg(feature = "host")]

extern crate lv2_core as core;
extern crate lv2_urid as urid;

use core::prelude::*;
use urid::mapper::{HashURIDMapper, URIDMapper};
use urid::prelude::*;

struct MyTypeA;

unsafe impl UriBound for MyTypeA {
    const URI: &'static [u8] = b"urn:my-type-a\0";
}

struct MyTypeB;

unsafe impl UriBound for MyTypeB {
    const URI: &'static [u8] = b"urn:my-type-b\0";
}

#[test]
fn test_map() {
    let mut mapper = Box::pin(HashURIDMapper::new());
    let host_map = mapper.as_mut().make_map_interface();
    let map_feature = Map::new(&host_map);

    assert_eq!(1, map_feature.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map_feature.map_type::<MyTypeA>().unwrap());

    assert_eq!(2, map_feature.map_type::<MyTypeB>().unwrap());
    assert_eq!(2, map_feature.map_uri(MyTypeB::uri()).unwrap());

    assert_eq!(1, map_feature.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map_feature.map_type::<MyTypeA>().unwrap());
}

#[test]
fn test_unmap() {
    let mut mapper = Box::pin(HashURIDMapper::new());
    let host_map = mapper.as_mut().make_map_interface();
    let host_unmap = mapper.as_mut().make_unmap_interface();
    let map_feature = Map::new(&host_map);
    let unmap_feature = Unmap::new(&host_unmap);

    let (type_a, type_b) = {
        (
            map_feature.map_type::<MyTypeA>().unwrap(),
            map_feature.map_type::<MyTypeB>().unwrap(),
        )
    };

    assert_eq!(MyTypeA::uri(), unmap_feature.unmap(type_a).unwrap());
    assert_eq!(MyTypeB::uri(), unmap_feature.unmap(type_b).unwrap());
}

#[derive(URIDCache)]
struct MyURIDCache {
    type_a: URID<MyTypeA>,
    type_b: URID<MyTypeB>,
}

#[test]
fn test_cache() {
    let mut mapper = Box::pin(HashURIDMapper::new());
    let host_map = mapper.as_mut().make_map_interface();
    let map_feature = Map::new(&host_map);
    let cache = MyURIDCache::from_map(&map_feature).unwrap();

    assert_eq!(1, cache.type_a);
    assert_eq!(2, cache.type_b);
}
