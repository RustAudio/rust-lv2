# Library for idiomatic URID support.

In the world of [RDF](https://en.wikipedia.org/wiki/Resource_Description_Framework), resources are described using [URIs](https://en.wikipedia.org/wiki/Uniform_Resource_Identifier). In Rust, this concept can be adapted to describe types with URIs using the [`UriBound`](trait.UriBound.html) trait. Then, other crates might use these URIs to describe relationships between types or values using URIs.

However, comparing URIs isn't necessarily fast. Therefore, another concept was introduced: The [URID](struct.URID.html). A URID is basically a `u32` which represents a URI. These URIDs are assigned by a [`Map`](trait.Map.html) and can be "dereferenced" by an [`Unmap`](trait.Unmap.html).

This library also supports connecting URIDs to their `UriBound` via a generics argument. This can be used, for example, to request the URID of a certain bound as a parameter of a function. If someone would try to call this function with the wrong URID, the compiler will raise an error before the code is even compiled.

This may seem a bit minor to you now, but the audio plugin framework [rust-lv2](https://github.com/RustAudio/rust-lv2) heavily relies on this crate for fast, portable and dynamic data identification and exchange.

# Example

``` Rust
use urid::*;

// Some types with URIs. The attribute implements `UriBound` with the given URI.
#[uri("urn:urid-example:my-struct-a")]
struct MyStructA;

#[uri("urn:urid-example:my-struct-b")]
struct MyStructB;

// A collection of URIDs that can be created by a mapper with one method call.
#[derive(URIDCollection)]
struct MyURIDCollection {
    my_struct_a: URID<MyStructA>,
    my_struct_b: URID<MyStructB>,
}

// A function that checks whether the unmapper behaves correctly.
// Due to the type argument, it can not be misused.
fn test_unmapper<M: Unmap, T: UriBound>(unmap: &M, urid: URID<T>) {
    assert_eq!(T::uri(), unmap.unmap(urid).unwrap());
}

// Create a simple mapper. The `HashURIDMapper` is thread-safe and can map and unmap all URIs.
let map = HashURIDMapper::new();

// Get the URIDs of the structs. You can use the collection or retrieve individual URIDs.
let urids: MyURIDCollection = map.populate_collection().unwrap();
let urid_a: URID<MyStructA> = map.map_type().unwrap();

// You can also retrieve the URID of a single URI without a binding to a type.
let urid_b: URID = map.map_str("https://rustup.rs").unwrap();

test_unmapper(&map, urids.my_struct_a);
test_unmapper(&map, urids.my_struct_b);
test_unmapper(&map, urid_a);
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.