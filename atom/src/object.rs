//! An atom containing multiple key-value pairs.
//!
//! This module is centered on the [`Object`](struct.Object.html) atom type. An object is the atomized form of an RDF instance: It has an (optional) id, a type and multiple properties declared as URID/Atom pairs. Both the id and the type are URIDs too.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use urid::*;
//!
//! #[uri("urn:object-class")]
//! struct ObjectClass;
//!
//! #[uri("urn:property-a")]
//! struct PropertyA;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! #[derive(URIDCollection)]
//! struct MyURIDs {
//!     atom: AtomURIDCollection,
//!     object_class: URID<ObjectClass>,
//!     property_a: URID<PropertyA>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &MyURIDs) {
//!     // Create the reading handle.
//!     // We don't need the header now.
//!     let (_header, object_reader) = ports.input.read(urids.atom.object, ()).unwrap();
//!
//!     /// Iterate through all properties of the object.
//!     for (property_header, atom) in object_reader {
//!         // If the property is an integer...
//!         if let Some(integer) = atom.read(urids.atom.int, ()) {
//!             // Print it!
//!             println!(
//!                 "Property No. {} has integer value {}",
//!                 property_header.key.get(),
//!                 integer
//!             );
//!         } else {
//!             // Print that is not an integer.
//!             println!(
//!                 "Property No. {} is not an integer",
//!                 property_header.key.get()
//!             );
//!         }
//!     }
//!
//!     // Initialize the object.
//!     let mut object_writer = ports.output.init(
//!         urids.atom.object,
//!         ObjectHeader {
//!             id: None,
//!             otype: urids.object_class.into_general(),
//!         }
//!     ).unwrap();
//!
//!     // Write a property to the object.
//!     object_writer.init(urids.property_a, urids.atom.int, 42).unwrap();
//! }
//! ```
//!
//! # Specification
//! [http://lv2plug.in/ns/ext/atom/atom.html#Object](http://lv2plug.in/ns/ext/atom/atom.html#Object).
use crate::space::*;
use crate::*;
use std::convert::TryFrom;
use std::iter::Iterator;
use urid::UriBound;
use urid::URID;

/// An atom containing multiple key-value pairs.
///
/// [See also the module documentation.](index.html)
pub struct Object;

unsafe impl UriBound for Object {
    const URI: &'static [u8] = sys::LV2_ATOM__Object;
}

/// Information about an object atom.
pub struct ObjectHeader {
    /// The id of the object to distinguish different objects of the same type.
    ///
    /// If you don't need it, you should set it to `None`.
    pub id: Option<URID>,
    /// The type of the object (same as `rdf:type`).
    pub otype: URID,
}

impl<'handle, 'space: 'handle> Atom<'handle, 'space> for Object {
    type ReadParameter = ();
    type ReadHandle = (ObjectHeader, ObjectReader<'handle>);
    type WriteParameter = ObjectHeader;
    type WriteHandle = ObjectWriter<'handle, 'space>;

    unsafe fn read(body: &'handle Space, _: ()) -> Option<(ObjectHeader, ObjectReader<'handle>)> {
        let (header, body) = body.split_for_value_as_unchecked::<sys::LV2_Atom_Object_Body>()?;
        let header = ObjectHeader {
            id: URID::try_from(header.id).ok(),
            otype: URID::try_from(header.otype).ok()?,
        };

        let reader = ObjectReader { space: body };

        Some((header, reader))
    }

    fn init(
        mut frame: AtomSpaceWriter<'handle, 'space>,
        header: ObjectHeader,
    ) -> Option<ObjectWriter<'handle, 'space>> {
        {
            let x = space::write_value(&mut frame, sys::LV2_Atom_Object_Body {
                id: header.id.map(URID::get).unwrap_or(0),
                otype: header.otype.get(),
            })?;
        }
        Some(ObjectWriter { frame })
    }
}

/// Alias of `Object`, used by older hosts.
///
/// A blank object is an object that isn't an instance of a class. The [specification recommends](https://lv2plug.in/ns/ext/atom/atom.html#Blank) to use an [`Object`](struct.Object.html) with an id of `None`, but some hosts still use it and therefore, it's included in this library.
///
/// If you want to read an object, you should also support `Blank`s, but if you want to write an object, you should always use `Object`.
pub struct Blank;

unsafe impl UriBound for Blank {
    const URI: &'static [u8] = sys::LV2_ATOM__Blank;
}

impl<'handle, 'space: 'handle> Atom<'handle, 'space> for Blank {
    type ReadParameter = <Object as Atom<'handle, 'space>>::ReadParameter;
    type ReadHandle = <Object as Atom<'handle, 'space>>::ReadHandle;
    type WriteParameter = <Object as Atom<'handle, 'space>>::WriteParameter;
    type WriteHandle = <Object as Atom<'handle, 'space>>::WriteHandle;

    #[allow(clippy::unit_arg)]
    #[inline]
    unsafe fn read(body: &'handle Space, parameter: Self::ReadParameter) -> Option<Self::ReadHandle> {
        Object::read(body, parameter)
    }

    fn init(
        frame: AtomSpaceWriter<'handle, 'space>,
        parameter: Self::WriteParameter,
    ) -> Option<Self::WriteHandle> {
        Object::init(frame, parameter)
    }
}

/// An iterator over all properties in an object.
///
/// Each iteration item is the header of the property, as well as the space occupied by the value atom. You can use normal `read` methods on the returned space.
pub struct ObjectReader<'a> {
    space: &'a Space,
}

impl<'a> Iterator for ObjectReader<'a> {
    type Item = (PropertyHeader, UnidentifiedAtom<'a>);

    fn next(&mut self) -> Option<(PropertyHeader, UnidentifiedAtom<'a>)> {
        // SAFETY: The fact that this contains a valid property is guaranteed by this type.
        let (header, atom, space) = unsafe { Property::read_body(self.space) }?;
        self.space = space;
        // SAFETY: The fact that this contains a valid atom header is guaranteed by this type.
        Some((header, atom))
    }
}

/// Writing handle for object properties.
///
/// This handle is a safeguard to assure that a object is always a series of properties.
pub struct ObjectWriter<'handle, 'space: 'handle> {
    frame: AtomSpaceWriter<'handle, 'space>,
}

impl<'handle, 'space: 'handle> ObjectWriter<'handle, 'space> {
    /// Initialize a new property with a context.
    ///
    /// This method does the same as [`init`](#method.init), but also sets the context URID.
    pub fn init_with_context<'read, K: ?Sized, T: ?Sized, A: Atom<'read, 'space>>(
        &'space mut self,
        key: URID<K>,
        context: URID<T>,
        child_urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        Property::write_header(&mut self.frame, key.into_general(), Some(context))?;
        space::init_atom(&mut self.frame, child_urid, parameter)
    }

    /// Initialize a new property.
    ///
    /// This method writes out the header of a property and returns a reference to the space, so the property values can be written.
    ///
    /// Properties also have a context URID internally, which is rarely used. If you want to add one, use [`init_with_context`](#method.init_with_context).
    pub fn init<'read, 'write, K: ?Sized, A: Atom<'read, 'space>>(
        &'space mut self,
        key: URID<K>,
        child_urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        Property::write_header(&mut self.frame, key, None::<URID<()>>)?;
        space::init_atom(&mut self.frame, child_urid, parameter)
    }
}

/// An atom containing a key-value pair.
///
/// A property represents a single URID -> atom mapping. Additionally and optionally, you may also define a context in which the property is valid. For more information, visit the [specification](http://lv2plug.in/ns/ext/atom/atom.html#Property).
///
/// Most of the time, properties are a part of an [`Object`](struct.Object.html) atom and therefore, you don't need to read or write them directly. However, they could in theory appear on their own too, which is why reading and writing methods are still provided.
pub struct Property;

unsafe impl UriBound for Property {
    const URI: &'static [u8] = sys::LV2_ATOM__Property;
}

/// Information about a property atom.
#[derive(Clone, Copy)]
pub struct PropertyHeader {
    /// The key of the property.
    pub key: URID,
    /// URID of the context (generally `None`).
    pub context: Option<URID>,
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
/// A custom version of the property body that does not include the value atom header.
///
/// We will retrieve/store it separately.
struct StrippedPropertyHeader {
    key: u32,
    context: u32,
}

impl Property {
    /// Read the body of a property atom from a space.
    ///
    /// It returns the property header, containing the key and optional context of the property, the body of the actual atom, and the remaining space behind the atom.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given Space actually contains a valid property.
    unsafe fn read_body(space: &Space) -> Option<(PropertyHeader, UnidentifiedAtom, &Space)> {
        let (header, space) = space.split_for_value_as_unchecked::<StrippedPropertyHeader>()?;

        let header = PropertyHeader {
            key: URID::try_from(header.key).ok()?,
            context: URID::try_from(header.context).ok(),
        };

        let (atom, space) = space.split_atom()?;
        Some((header, atom, space))
    }

    /// Write out the header of a property atom.
    ///
    /// This method simply writes out the content of the header to the space and returns `Some(())` if it's successful.
    #[inline]
    fn write_header<'a, 'space, K: ?Sized, C: ?Sized>(
        space: &'a mut impl AllocateSpace<'space>,
        key: URID<K>,
        context: Option<URID<C>>,
    ) -> Option<()> {
        let header = StrippedPropertyHeader {
            key: key.get(),
            context: context.map(URID::get).unwrap_or(0)
        };

        space::write_value(space, header)?;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use std::mem::size_of;
    use urid::*;
    use std::ops::Deref;

    #[test]
    fn test_object() {
        let map = HashURIDMapper::new();
        let urids = AtomURIDCollection::from_map(&map).unwrap();

        let object_type = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:my-type\0").unwrap())
            .unwrap();

        let first_key = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:value-a\0").unwrap())
            .unwrap();
        let first_value: i32 = 17;

        let second_key = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:value-b\0").unwrap())
            .unwrap();
        let second_value: f32 = 42.0;

        let mut raw_space = AtomSpace::boxed(256);

        // writing
        {
            let mut cursor = raw_space.as_bytes_mut();
            let frame = AtomSpaceWriter::write_new(&mut cursor, urids.object).unwrap();
            let mut writer = Object::init(
                frame,
                ObjectHeader {
                    id: None,
                    otype: object_type,
                },
            )
            .unwrap();
            {
                writer.init(first_key, urids.int, first_value).unwrap();
            }
            {
                writer.init(second_key, urids.float, second_value).unwrap();
            }
        }

        // Atom header: size: u32, type: u32
        // Object header: id: u32 = None, otype: u32 = object_type
            // Object prop header1: key: u32 = first_key, context: u32 = 0
            // Object prop body atom: size: u32 = 4 type: u32 = int
                // Int atom value: i32 = 17, padding(4)
            // Object prop header12 key: u32 = first_key, context: u32 = 0
            // Object prop body atom: size: u32 = 4 type: u32 = int
                // Float atom value: i32 = 69, padding(4)

        // verifying
        {
            // Header
            let s = raw_space.deref();
            let (atom, space) = unsafe { raw_space.split_atom() }.unwrap();
            let header = atom.header().unwrap();
            let x = atom.body().unwrap().len();
            assert_eq!(header.urid(), urids.object);
            assert_eq!(
                header.size_of_body(),
                size_of::<sys::LV2_Atom_Object_Body>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + 2 * size_of::<i32>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + size_of::<f32>()
            );

            // Object.
            let (object, space) = unsafe { atom.body().unwrap().split_for_value_as_unchecked::<sys::LV2_Atom_Object_Body>() }.unwrap();
            assert_eq!(object.id, 0);
            assert_eq!(object.otype, object_type);

            // First property.
            let (property, space) = unsafe { space.split_for_value_as_unchecked::<sys::LV2_Atom_Property_Body>() }.unwrap();
            assert_eq!(property.key, first_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.int);
            assert_eq!(property.value.size as usize, size_of::<i32>());

            let (value, space) = unsafe { space.split_for_value_as_unchecked::<i32>() }.unwrap();
            assert_eq!(*value, first_value);

            // Second property.
            let (property, space) = unsafe { space.split_for_value_as_unchecked::<sys::LV2_Atom_Property_Body>() }.unwrap();
            assert_eq!(property.key, second_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.float);
            assert_eq!(property.value.size as usize, size_of::<f32>());

            let (value, space) = unsafe { space.split_for_value_as_unchecked::<f32>() }.unwrap();
            assert_eq!(*value, second_value);
            assert_eq!(space.len(), 0);
        }

        // reading
        {
            let (body, _) = unsafe { raw_space.split_atom_body(urids.object) }.unwrap();

            let (header, iter) = unsafe { Object::read(body, ()) }.unwrap();
            assert_eq!(header.otype, object_type);
            assert_eq!(header.id, None);

            let properties: Vec<(PropertyHeader, UnidentifiedAtom)> = iter.collect();
            let (header, atom) = properties[0];
            assert_eq!(header.key, first_key);
            assert_eq!(atom.read::<Int>(urids.int, ()).unwrap(), first_value);
            let (header, atom) = properties[1];
            assert_eq!(header.key, second_key);
            assert_eq!(atom.read::<Float>(urids.float, ()).unwrap(), second_value);
        }
    }
}
