use urid::*;

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
    let map = HashURIDMapper::new();

    assert_eq!(1, map.map(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());

    assert_eq!(2, map.map_type::<MyTypeB>().unwrap());
    assert_eq!(2, map.map(MyTypeB::uri()).unwrap());

    assert_eq!(1, map.map(MyTypeA::uri()).unwrap());
    assert_eq!(1, map.map_type::<MyTypeA>().unwrap());
}

#[test]
fn test_unmap() {
    let map = HashURIDMapper::new();

    let (type_a, type_b) = {
        (
            map.map_type::<MyTypeA>().unwrap(),
            map.map_type::<MyTypeB>().unwrap(),
        )
    };

    assert_eq!(MyTypeA::uri(), map.unmap(type_a).unwrap());
    assert_eq!(MyTypeB::uri(), map.unmap(type_b).unwrap());
}

#[derive(URIDCollection)]
struct MyURIDCollection {
    type_a: URID<MyTypeA>,
    type_b: URID<MyTypeB>,
}

#[test]
fn test_collection() {
    let map = HashURIDMapper::new();
    let collection = MyURIDCollection::from_map(&map).unwrap();

    assert_eq!(1, collection.type_a);
    assert_eq!(2, collection.type_b);
}
