use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use std::convert::TryFrom;
use urid::{URIDBound, URID};

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

/// An atom containing a key-value pair.
///
/// A property as a pair of a URID key and an atom as it's value. Additionally and optionally, you may also define a context in which the property is valid.
///
/// Most of the time, properties are a part of an [`Object`](struct.Object.html) atom and therefore, you don't to read or write them directly. However, they could also appear on their own in theory, which is why reading and writing methods are still provided.
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
    fn write_header<'a, 'b>(space: &'b mut dyn MutSpace<'a>, header: PropertyHeader) -> Option<()> {
        space.write(&header.key.get(), true)?;
        match header.context {
            Some(context) => space.write(&context.get(), false)?,
            None => space.write::<u32>(&0, false)?,
        };
        Some(())
    }

    /// Initialize a property atom.
    ///
    /// This method initializes a property atom by creating an atom frame and writing out the property header. However, it does not not initialize the value atom. You have to do that yourselves and if you don't, the propertry is invalid.
    pub fn write<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        header: PropertyHeader,
        urids: &AtomURIDCache,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        let mut frame = space.create_atom_frame(urids.property)?;
        if Self::write_header(&mut frame, header).is_some() {
            Some(frame)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::object::*;
    use crate::scalar::*;
    use crate::AtomURIDCache;
    use core::Uri;
    use std::mem::size_of;
    use std::os::raw::c_int;
    use urid::URIDCache;

    #[test]
    fn test_property() {
        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = AtomURIDCache::from_map(&map).unwrap();

        let key = map
            .map_uri(Uri::from_bytes_with_nul(b"urn:myvalue\0").unwrap())
            .unwrap();
        let value: c_int = 42;

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let header = PropertyHeader { key, context: None };

            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut frame = Property::write(&mut space, header, &urids).unwrap();
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

            let read_value = unsafe {*(space.as_ptr() as *const c_int)};
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
