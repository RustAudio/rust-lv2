use urid::*;

#[uri("urn:my-struct")]
pub struct MyStruct {
    _a: i32,
}

#[uri("urn:my-enum")]
pub enum MyEnum {
    _A,
    _B,
}

#[uri("urn:my-union")]
pub union MyUnion {
    _a: i32,
    _b: f32,
}

pub struct MyGeneric<T> {
    _t: T,
}

#[uri("urn:my-type")]
pub type MyType = MyGeneric<i32>;

fn test_type<T: UriBound>(expected_uri: &str) {
    // Test for the null terminator.
    assert_eq!(0, T::URI[T::URI.len() - 1]);

    // Test for string equality.
    assert_eq!(expected_uri, T::uri().to_str().unwrap());
}

#[test]
fn test_bounds() {
    test_type::<MyStruct>("urn:my-struct");
    test_type::<MyEnum>("urn:my-enum");
    test_type::<MyUnion>("urn:my-union");
    test_type::<MyType>("urn:my-type");
}
