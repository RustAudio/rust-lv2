//! An atom containing multiple key-value pairs.
//!
//! This module is centered on the [`Object`](struct.Object.html) atom type. An object is the atomized form of an RDF instance: It has an (optional) id, a type and multiple properties declared as URID/Atom pairs. Both the id and the type are URIDs too.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_urid::prelude::*;
//!
//! struct ObjectClass;
//! unsafe impl UriBound for ObjectClass {
//!     const URI: &'static [u8] = b"urn:object-class\0";
//! }
//!
//! struct PropertyA;
//! unsafe impl UriBound for PropertyA {
//!     const URI: &'static [u8] = b"urn:property-a\0";
//! }
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! #[derive(URIDCache)]
//! struct MyURIDs {
//!     atom: AtomURIDCache,
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
//!     object_writer.write(urids.property_a, None, urids.atom.int, 42).unwrap();
//! }
//! ```
//!
//! # Specification
//! [http://lv2plug.in/ns/ext/atom/atom.html#Object](http://lv2plug.in/ns/ext/atom/atom.html#Object).
use crate::space::*;
use crate::*;
use core::UriBound;
use std::convert::TryFrom;
use std::iter::Iterator;
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

impl<'a, 'b> Atom<'a, 'b> for Object
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = (ObjectHeader, ObjectReader<'a>);
    type WriteParameter = ObjectHeader;
    type WriteHandle = ObjectWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<(ObjectHeader, ObjectReader<'a>)> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Object_Body>()?;
        let header = ObjectHeader {
            id: URID::try_from(header.id).ok(),
            otype: URID::try_from(header.otype).ok()?,
        };

        let reader = ObjectReader { space: body };

        Some((header, reader))
    }

    fn init(
        mut frame: FramedMutSpace<'a, 'b>,
        header: ObjectHeader,
    ) -> Option<ObjectWriter<'a, 'b>> {
        {
            let frame = &mut frame as &mut dyn MutSpace;
            frame.write(
                &sys::LV2_Atom_Object_Body {
                    id: header.id.map(|urid| urid.get()).unwrap_or(0),
                    otype: header.otype.get(),
                },
                true,
            );
        }
        Some(ObjectWriter { frame })
    }
}

/// Deprecated alias of `Object`
/// 
/// A blank object is an object that isn't an instance of a class. The [specification recommends](https://lv2plug.in/ns/ext/atom/atom.html#Blank) to use an [`Object`](struct.Object.html) with an id of `None`, but some hosts still use it and therefore, it's included in this library.
#[deprecated]
pub struct Blank;

#[allow(deprecated)]
unsafe impl UriBound for Blank {
    const URI: &'static [u8] = sys::LV2_ATOM__Blank;
}

#[allow(deprecated)]
impl<'a, 'b> Atom<'a, 'b> for Blank
where
    'a: 'b,
{
    type ReadParameter = <Object as Atom<'a, 'b>>::ReadParameter;
    type ReadHandle = <Object as Atom<'a, 'b>>::ReadHandle;
    type WriteParameter = <Object as Atom<'a, 'b>>::WriteParameter;
    type WriteHandle = <Object as Atom<'a, 'b>>::WriteHandle;

    fn read(body: Space<'a>, parameter: Self::ReadParameter) -> Option<Self::ReadHandle> {
        Object::read(body, parameter)
    }

    fn init(
        frame: FramedMutSpace<'a, 'b>,
        parameter: Self::WriteParameter,
    ) -> Option<Self::WriteHandle> {
        Object::init(frame, parameter)
    }
}

/// An iterator over all properties in an object.
///
/// Each iteration item is the header of the property, as well as the space occupied by the value atom. You can use normal `read` methods on the returned space.
pub struct ObjectReader<'a> {
    space: Space<'a>,
}

impl<'a> Iterator for ObjectReader<'a> {
    type Item = (PropertyHeader, UnidentifiedAtom<'a>);

    fn next(&mut self) -> Option<(PropertyHeader, UnidentifiedAtom<'a>)> {
        let (header, value, space) = Property::read_body(self.space)?;
        self.space = space;
        Some((header, UnidentifiedAtom::new(value)))
    }
}

/// Writing handle for object properties.
///
/// This handle is a safeguard to assure that a object is always a series of properties.
pub struct ObjectWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
}

impl<'a, 'b> ObjectWriter<'a, 'b> {
    /// Initialize a new property.
    ///
    /// This method writes out the header of a property and returns a reference to the space, so the property values can be written.
    pub fn write<'c, G: ?Sized, A: Atom<'a, 'c>>(
        &'c mut self,
        key: URID<G>,
        context: Option<URID>,
        child_urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        Property::write_header(&mut self.frame, key.into_general(), context)?;
        let child_frame = (&mut self.frame as &mut dyn MutSpace).create_atom_frame(child_urid)?;
        A::init(child_frame, parameter)
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

impl Property {
    /// Read the body of a property atom from a space.
    ///
    /// This method assumes that the space actually contains the body of a property atom, without the header. It returns the property header, containing the key and optional context of the property, the body of the actual atom, and the space behind the atom.
    fn read_body(space: Space) -> Option<(PropertyHeader, Space, Space)> {
        #[repr(C)]
        #[derive(Clone, Copy)]
        /// A custom version of the property body that does not include the value atom header.
        ///
        /// We will retrieve it separately.
        struct StrippedPropertyBody {
            key: u32,
            context: u32,
        }

        let (header, space) = space.split_type::<StrippedPropertyBody>()?;

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
    fn write_header(space: &mut dyn MutSpace, key: URID, context: Option<URID>) -> Option<()> {
        space.write(&key.get(), true)?;
        space.write(&context.map(|urid| urid.get()).unwrap_or(0), false)?;
        Some(())
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use core::prelude::*;
    use std::mem::size_of;
    use urid::mapper::*;
    use urid::prelude::*;

    #[test]
    fn test_object() {
        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = AtomURIDCache::from_map(&map).unwrap();

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

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.object)
                .unwrap();
            let mut writer = Object::init(
                frame,
                ObjectHeader {
                    id: None,
                    otype: object_type,
                },
            )
            .unwrap();
            {
                writer
                    .write(first_key, None, urids.int, first_value)
                    .unwrap();
            }
            {
                writer
                    .write(second_key, None, urids.float, second_value)
                    .unwrap();
            }
        }

        // verifying
        {
            // Header
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, urids.object);
            assert_eq!(
                atom.size as usize,
                size_of::<sys::LV2_Atom_Object_Body>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + 2 * size_of::<i32>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + size_of::<f32>()
            );

            // Object.
            let (object, space) = space.split_at(size_of::<sys::LV2_Atom_Object_Body>());
            let object = unsafe { &*(object.as_ptr() as *const sys::LV2_Atom_Object_Body) };
            assert_eq!(object.id, 0);
            assert_eq!(object.otype, object_type);

            // First property.
            let (property, space) = space.split_at(size_of::<sys::LV2_Atom_Property_Body>());
            let property = unsafe { &*(property.as_ptr() as *const sys::LV2_Atom_Property_Body) };
            assert_eq!(property.key, first_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.int);
            assert_eq!(property.value.size as usize, size_of::<i32>());

            let (value, space) = space.split_at(size_of::<i32>());
            let value = unsafe { *(value.as_ptr() as *const i32) };
            assert_eq!(value, first_value);
            let (_, space) = space.split_at(size_of::<i32>());

            // Second property.
            let (property, space) = space.split_at(size_of::<sys::LV2_Atom_Property_Body>());
            let property = unsafe { &*(property.as_ptr() as *const sys::LV2_Atom_Property_Body) };
            assert_eq!(property.key, second_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.float);
            assert_eq!(property.value.size as usize, size_of::<f32>());

            let (value, _) = space.split_at(size_of::<f32>());
            let value = unsafe { *(value.as_ptr() as *const f32) };
            assert_eq!(value, second_value);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.object).unwrap();

            let (header, iter) = Object::read(body, ()).unwrap();
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
