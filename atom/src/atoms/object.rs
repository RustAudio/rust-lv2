//! An atom containing multiple key-value pairs.
//!
//! This module is centered on the [`Object`](struct.Object.html) atom type. An object is the atomized form of an RDF instance: It has an (optional) id, a type and multiple properties declared as URID/Atom pairs. Both the id and the type are URIDs too.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use urid::*;
//! use lv2_atom::atoms::object::ObjectHeader;
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
//!     let (_header, object_reader) = ports.input.read(urids.atom.object).unwrap();
//!
//!     /// Iterate through all properties of the object.
//!     for (property_header, atom) in object_reader {
//!         // If the property is an integer...
//!         if let Ok(integer) = atom.read(urids.atom.int) {
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
//!     let mut object_writer = ports.output.write(urids.atom.object)
//!         .unwrap()
//!         .write_header(
//!             ObjectHeader {
//!                 id: None,
//!                 otype: urids.object_class.into_general(),
//!             }
//!     ).unwrap();
//!
//!     // Write a property to the object.
//!     object_writer.new_property(urids.property_a, urids.atom.int).unwrap();
//! }
//! ```
//!
//! # Specification
//! [http://lv2plug.in/ns/ext/atom/atom.html#Object](http://lv2plug.in/ns/ext/atom/atom.html#Object).
use crate::space::SpaceReader;
use crate::*;
use core::convert::TryFrom;
use core::iter::Iterator;
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
#[derive(Copy, Clone)]
pub struct ObjectHeader {
    /// The id of the object to distinguish different objects of the same type.
    ///
    /// If you don't need it, you should set it to `None`.
    pub id: Option<URID>,
    /// The type of the object (same as `rdf:type`).
    pub otype: URID,
}

pub struct ObjectReaderHandle;
impl<'a> AtomHandle<'a> for ObjectReaderHandle {
    type Handle = (ObjectHeader, ObjectReader<'a>);
}

pub struct ObjectWriterHandle;
impl<'a> AtomHandle<'a> for ObjectWriterHandle {
    type Handle = ObjectHeaderWriter<'a>;
}

/// A type-state for the Object Writer, that writes the header of an object.
pub struct ObjectHeaderWriter<'a> {
    frame: AtomWriter<'a>,
}

impl<'a> ObjectHeaderWriter<'a> {
    /// Initializes the object with the given header.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    pub fn write_header(
        mut self,
        header: ObjectHeader,
    ) -> Result<ObjectWriter<'a>, AtomWriteError> {
        self.frame.write_value(sys::LV2_Atom_Object_Body {
            id: header.id.map(URID::get).unwrap_or(0),
            otype: header.otype.get(),
        })?;

        Ok(ObjectWriter { frame: self.frame })
    }
}

impl Atom for Object {
    type ReadHandle = ObjectReaderHandle;
    type WriteHandle = ObjectWriterHandle;

    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Object_Body = reader.next_value()?;

        let header = ObjectHeader {
            id: URID::try_from(header.id).ok(),
            otype: URID::try_from(header.otype).map_err(|_| AtomReadError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
                error_message: "Invalid object type URID: 0",
            })?,
        };

        let reader = ObjectReader { reader };

        Ok((header, reader))
    }

    fn write(
        frame: AtomWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Ok(ObjectHeaderWriter { frame })
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

impl Atom for Blank {
    type ReadHandle = <Object as Atom>::ReadHandle;
    type WriteHandle = <Object as Atom>::WriteHandle;

    #[inline]
    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        Object::read(body)
    }

    #[inline]
    fn write(
        frame: AtomWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Object::write(frame)
    }
}

/// An iterator over all properties in an object.
///
/// Each iteration item is the header of the property, as well as the space occupied by the value atom. You can use normal `read` methods on the returned space.
pub struct ObjectReader<'a> {
    reader: SpaceReader<'a>,
}

impl<'a> Iterator for ObjectReader<'a> {
    type Item = (PropertyHeader, &'a UnidentifiedAtom);

    fn next(&mut self) -> Option<(PropertyHeader, &'a UnidentifiedAtom)> {
        // SAFETY: The fact that this contains a valid property is guaranteed by this type.
        self.reader
            .try_read(|reader| unsafe { Property::read_body(reader) })
            .ok()
    }
}

/// Writing handle for object properties.
///
/// This handle is a safeguard to assure that a object is always a series of properties.
pub struct ObjectWriter<'a> {
    frame: AtomWriter<'a>,
}

impl<'a> ObjectWriter<'a> {
    /// Initializes a new property, with a given context.
    ///
    /// This method does the same as [`init`](#method.init), but also sets the context URID.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    pub fn new_property_with_context<K: ?Sized, T: ?Sized, A: Atom>(
        &mut self,
        key: URID<K>,
        context: URID<T>,
        atom_type: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Property::write_header(&mut self.frame, key.into_general(), Some(context))?;
        self.frame.write_atom(atom_type)
    }

    /// Initializes a new property.
    ///
    /// This method writes out the header of a property and returns a reference to the space, so the property values can be written.
    ///
    /// Properties also have a context URID internally, which is rarely used. If you want to add one, use [`init_with_context`](#method.init_with_context).
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    pub fn new_property<K: ?Sized, A: Atom>(
        &mut self,
        key: URID<K>,
        atom_type: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Property::write_header(&mut self.frame, key, None::<URID<()>>)?;
        self.frame.write_atom(atom_type)
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
    unsafe fn read_body<'a>(
        reader: &mut SpaceReader<'a>,
    ) -> Result<(PropertyHeader, &'a UnidentifiedAtom), AtomReadError> {
        let header: &StrippedPropertyHeader = reader.next_value()?;

        let header = PropertyHeader {
            key: URID::try_from(header.key).map_err(|_| AtomReadError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
                error_message: "Invalid object property key URID: 0",
            })?,
            context: URID::try_from(header.context).ok(),
        };

        let atom = reader.next_atom()?;

        Ok((header, atom))
    }

    /// Write out the header of a property atom.
    ///
    /// This method simply writes out the content of the header to the space and returns `Some(())` if it's successful.
    #[inline]
    fn write_header<K: ?Sized, C: ?Sized>(
        space: &mut impl SpaceWriter,
        key: URID<K>,
        context: Option<URID<C>>,
    ) -> Result<(), AtomWriteError> {
        let header = StrippedPropertyHeader {
            key: key.get(),
            context: context.map(URID::get).unwrap_or(0),
        };

        space.write_value(header)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::atoms::object::{ObjectHeader, PropertyHeader};
    use crate::prelude::*;
    use crate::space::*;
    use crate::AtomHeader;
    use std::mem::size_of;
    use urid::*;

    #[test]
    #[allow(clippy::float_cmp)]
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

        let mut raw_space = AlignedVec::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut cursor = SpaceCursor::new(raw_space.as_bytes_mut());
            let frame = AtomWriter::write_new(&mut cursor, urids.object).unwrap();
            let mut writer = Object::write(frame)
                .unwrap()
                .write_header(ObjectHeader {
                    id: None,
                    otype: object_type,
                })
                .unwrap();
            {
                writer
                    .new_property(first_key, urids.int)
                    .unwrap()
                    .set(first_value)
                    .unwrap();
            }
            {
                writer
                    .new_property(second_key, urids.float)
                    .unwrap()
                    .set(second_value)
                    .unwrap();
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
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();

            assert_eq!(atom.header().urid(), urids.object);
            assert_eq!(
                atom.header().size_of_body(),
                size_of::<sys::LV2_Atom_Object_Body>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + 2 * size_of::<i32>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + 2 * size_of::<f32>()
            );

            // Object.
            let mut object_reader = atom.body().read();
            let object: &sys::LV2_Atom_Object_Body = unsafe { object_reader.next_value() }.unwrap();
            assert_eq!(object.id, 0);
            assert_eq!(object.otype, object_type);

            // First property.
            let property: &sys::LV2_Atom_Property_Body =
                unsafe { object_reader.next_value() }.unwrap();
            assert_eq!(property.key, first_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.int);
            assert_eq!(property.value.size as usize, 2 * size_of::<i32>());

            let value: &i32 = unsafe { object_reader.next_value() }.unwrap();
            assert_eq!(*value, first_value);

            // Second property.
            let property: &sys::LV2_Atom_Property_Body =
                unsafe { object_reader.next_value() }.unwrap();
            assert_eq!(property.key, second_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.float);
            assert_eq!(property.value.size as usize, 2 * size_of::<f32>());

            let value: &f32 = unsafe { object_reader.next_value() }.unwrap();
            assert_eq!(*value, second_value);
            assert_eq!(object_reader.remaining_bytes().len(), 4);
        }

        // reading
        {
            let (header, iter) = unsafe { raw_space.read().next_atom() }
                .unwrap()
                .read(urids.object)
                .unwrap();

            assert_eq!(header.otype, object_type);
            assert_eq!(header.id, None);

            let properties: Vec<(PropertyHeader, &UnidentifiedAtom)> = iter.collect();

            let (header, atom) = properties[0];
            assert_eq!(header.key, first_key);
            assert_eq!(*atom.read(urids.int).unwrap(), first_value);

            let (header, atom) = properties[1];
            assert_eq!(header.key, second_key);
            assert_eq!(*atom.read(urids.float).unwrap(), second_value);
        }
    }
}
