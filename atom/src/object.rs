use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use std::convert::TryFrom;
use std::iter::Iterator;
use urid::{URIDBound, URID};

/// An atom containing multiple key-value pairs.
///
/// As [specified](http://lv2plug.in/ns/ext/atom/atom.html#Object), the object is the atom representation of an RDF resource, which you can think of as a URID -> atom map.
pub struct Object;

unsafe impl UriBound for Object {
    const URI: &'static [u8] = sys::LV2_ATOM__Object;
}

impl URIDBound for Object {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.object
    }
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

impl Object {
    /// Read an object from a space.
    ///
    /// This method assumes that the space contains an object atom and returns the header of the atom as well as an iterator over all properties of the object. Lastly, it also returns the space behind the atom.
    pub fn read<'a>(
        space: Space<'a>,
        urids: &AtomURIDCache,
    ) -> Option<(ObjectHeader, ObjectReader<'a>, Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.object)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Object_Body>()?;
        let header = ObjectHeader {
            id: URID::try_from(header.id).ok(),
            otype: URID::try_from(header.otype).ok()?,
        };

        let reader = ObjectReader { space: body };

        Some((header, reader, space))
    }

    /// Initialize an object atom.
    ///
    /// This method creates an atom frame, writes out the header and returns a writer. With the writer, you can add properties.
    pub fn write<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        id: Option<URID>,
        otype: URID,
        urids: &AtomURIDCache,
    ) -> Option<ObjectWriter<'a, 'b>> {
        let mut frame = space.create_atom_frame(urids.object)?;
        {
            let frame = &mut frame as &mut dyn MutSpace;
            frame.write(
                &sys::LV2_Atom_Object_Body {
                    id: id.map(|urid| urid.get()).unwrap_or(0),
                    otype: otype.get(),
                },
                true,
            );
        }
        Some(ObjectWriter { frame })
    }
}

/// An iterator over all properties in an object.
///
/// Each iteration item is the header of the property, as well as the space occupied by the value atom. You can use normal `read` methods on the returned space.
pub struct ObjectReader<'a> {
    space: Space<'a>,
}

impl<'a> Iterator for ObjectReader<'a> {
    type Item = (PropertyHeader, Space<'a>);

    fn next(&mut self) -> Option<(PropertyHeader, Space<'a>)> {
        let (header, value, space) = Property::read_body(self.space)?;
        self.space = space;
        Some((header, value))
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
    pub fn write_property<'c>(
        &'c mut self,
        key: URID,
        context: Option<URID>,
    ) -> Option<&'c mut dyn MutSpace<'a>> {
        Property::write_header(&mut self.frame, key, context)?;
        Some(&mut self.frame)
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

impl URIDBound for Property {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.property
    }
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

    /// Read a property atom from a space.
    ///
    /// This method assumes that the space contains a property atom, including the header, and returns the information header of the property, the space with the contained atom and the space behind the property.
    pub fn read<'a>(
        space: Space<'a>,
        urids: &AtomURIDCache,
    ) -> Option<(PropertyHeader, Space<'a>, Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.property)?;
        let (header, atom, _) = Self::read_body(body)?;
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

    /// Initialize a property atom.
    ///
    /// This method initializes a property atom by creating an atom frame and writing out the property header. However, it does not not initialize the value atom. You have to do that yourselves and if you don't, the propertry is invalid.
    pub fn write<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        key: URID,
        context: Option<URID>,
        urids: &AtomURIDCache,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        let mut frame = space.create_atom_frame(urids.property)?;
        if Self::write_header(&mut frame, key, context).is_some() {
            Some(frame)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::object::*;
    use crate::scalar::*;
    use crate::AtomURIDCache;
    use core::Uri;
    use std::mem::size_of;
    use std::os::raw::*;
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    #[test]
    fn test_object() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = AtomURIDCache::from_map(&map).unwrap();

        let object_type = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:my-type\0").unwrap())
            .unwrap();

        let first_key = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:value-a\0").unwrap())
            .unwrap();
        let first_value: c_int = 17;

        let second_key = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:value-b\0").unwrap())
            .unwrap();
        let second_value: c_float = 42.0;

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = Object::write(&mut space, None, object_type, &urids).unwrap();
            {
                let space = writer.write_property(first_key, None).unwrap();
                Int::write(space, first_value, &urids).unwrap();
            }
            {
                let space = writer.write_property(second_key, None).unwrap();
                Float::write(space, second_value, &urids).unwrap();
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
                    + 2 * size_of::<c_int>()
                    + size_of::<sys::LV2_Atom_Property_Body>()
                    + size_of::<c_float>()
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
            assert_eq!(property.value.size as usize, size_of::<c_int>());

            let (value, space) = space.split_at(size_of::<c_int>());
            let value = unsafe { *(value.as_ptr() as *const c_int) };
            assert_eq!(value, first_value);
            let (_, space) = space.split_at(size_of::<c_int>());

            // Second property.
            let (property, space) = space.split_at(size_of::<sys::LV2_Atom_Property_Body>());
            let property = unsafe { &*(property.as_ptr() as *const sys::LV2_Atom_Property_Body) };
            assert_eq!(property.key, second_key);
            assert_eq!(property.context, 0);
            assert_eq!(property.value.type_, urids.float);
            assert_eq!(property.value.size as usize, size_of::<c_float>());

            let (value, _) = space.split_at(size_of::<c_float>());
            let value = unsafe { *(value.as_ptr() as *const c_float) };
            assert_eq!(value, second_value);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());

            let (header, iter, _) = Object::read(space, &urids).unwrap();
            assert_eq!(header.otype, object_type);
            assert_eq!(header.id, None);

            let properties: Vec<(PropertyHeader, Space)> = iter.collect();
            assert_eq!(properties[0].0.key, first_key);
            assert_eq!(Int::read(properties[0].1, &urids).unwrap().0, first_value);
            assert_eq!(properties[1].0.key, second_key);
            assert_eq!(
                Float::read(properties[1].1, &urids).unwrap().0,
                second_value
            );
        }
    }

    #[test]
    fn test_property() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = AtomURIDCache::from_map(&map).unwrap();

        let key = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:myvalue\0").unwrap())
            .unwrap();
        let value: c_int = 42;

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut frame = Property::write(&mut space, key, None, &urids).unwrap();
            Int::write(&mut frame, value, &urids).unwrap();
        }

        // verifying
        {
            let (property, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Property>());

            let property = unsafe { &*(property.as_ptr() as *const sys::LV2_Atom_Property) };
            assert_eq!(
                property.atom.size as usize,
                size_of::<sys::LV2_Atom_Property_Body>() + size_of::<c_int>()
            );
            assert_eq!(property.atom.type_, urids.property);
            assert_eq!(property.body.key, key);
            assert_eq!(property.body.context, 0);
            assert_eq!(property.body.value.size as usize, size_of::<c_int>());
            assert_eq!(property.body.value.type_, urids.int);

            let read_value = unsafe { *(space.as_ptr() as *const c_int) };
            assert_eq!(read_value, value);
        }

        // reading
        {
            let space = Space::from_reference(raw_space.as_ref());
            let (header, atom, _) = Property::read(space, &urids).unwrap();
            assert_eq!(header.key, key);
            assert_eq!(header.context, None);
            let (read_value, _) = Int::read(atom, &urids).unwrap();
            assert_eq!(read_value, value);
        }
    }
}
