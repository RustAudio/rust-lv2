use lv2_urid::*;
use std::pin::Pin;
use urid::*;

#[uri_bound("urn:my-type-a")]
struct MyTypeA;

#[uri_bound("urn:my-type-b")]
struct MyTypeB;

#[test]
fn test_map() {
    let mut host_map: Pin<Box<HostMap<HashURIDMapper>>> = Box::pin(HashURIDMapper::new().into());
    let map_interface = host_map.as_mut().make_map_interface();
    let map = LV2Map::new(&map_interface);

    assert_eq!(1, map.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());

    assert_eq!(2, map.map_uri(MyTypeB::uri()).unwrap());
    assert_eq!(2, map.map_type::<MyTypeB>().unwrap());

    assert_eq!(1, map.map_uri(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());
}

#[test]
fn test_unmap() {
    let mut host_map: Pin<Box<HostMap<HashURIDMapper>>> = Box::pin(HashURIDMapper::new().into());
    let map_interface = host_map.as_mut().make_map_interface();
    let map = LV2Map::new(&map_interface);
    let unmap_interface = host_map.as_mut().make_unmap_interface();
    let unmap = LV2Unmap::new(&unmap_interface);

    let (type_a, type_b) = {
        (
            map.map_type::<MyTypeA>().unwrap(),
            map.map_type::<MyTypeB>().unwrap(),
        )
    };

    assert_eq!(MyTypeA::uri(), unmap.unmap(type_a).unwrap());
    assert_eq!(MyTypeB::uri(), unmap.unmap(type_b).unwrap());
}

#[derive(URIDCollection)]
struct MyURIDCollection {
    type_a: URID<MyTypeA>,
    type_b: URID<MyTypeB>,
}

#[test]
fn test_collection() {
    let mut host_map: Pin<Box<HostMap<HashURIDMapper>>> = Box::pin(HashURIDMapper::new().into());
    let map_interface = host_map.as_mut().make_map_interface();
    let map = LV2Map::new(&map_interface);
    let collection = MyURIDCollection::from_map(&map).unwrap();

    assert_eq!(1, collection.type_a);
    assert_eq!(2, collection.type_b);
}
