use crate::space::*;
use crate::AtomURIDCache;
use crate::ScalarAtom;
use core::UriBound;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::size_of;
use urid::{URIDBound, URID};

/// An atom containing a slice of scalar atoms.
///
/// This atom is specified [here](http://lv2plug.in/ns/ext/atom/atom.html#Vector).
pub struct Vector;

unsafe impl UriBound for Vector {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

impl URIDBound for Vector {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.vector
    }
}

impl Vector {
    /// Try to read the content of a vector containing `C` atoms.
    ///
    /// If successful, the method returns the content of the atom, a slice of `C::InternalType`, and the space behind the atom.
    ///
    /// If the space is not big enough or does not contain a vector with the given content type, the method returns `None`.
    pub fn read<'a, C: ScalarAtom>(
        space: Space<'a>,
        urids: &AtomURIDCache,
        child_urids: &C::CacheType,
    ) -> Option<(&'a [C::InternalType], Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.vector)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Vector_Body>()?;

        if header.child_type != C::urid(child_urids)
            || header.child_size as usize != size_of::<C::InternalType>()
        {
            return None;
        }

        let data = body.data()?;

        assert_eq!(data.len() % size_of::<C::InternalType>(), 0);
        let children_count = data.len() / size_of::<C::InternalType>();

        let children = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const C::InternalType, children_count)
        };
        Some((children, space))
    }

    /// Initialize a vector containing `C` atoms.
    ///
    /// This method initializes an empty vector and returns a writer to add `C` to the vector. If the space is not big enough, the method returns `None`.
    pub fn write<'a, 'b, C: ScalarAtom>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        child_urids: &C::CacheType,
    ) -> Option<VectorWriter<'a, 'b, C::InternalType>> {
        let mut frame = space.create_atom_frame(urids.vector)?;

        let body = sys::LV2_Atom_Vector_Body {
            child_type: C::urid(child_urids).get(),
            child_size: size_of::<C::InternalType>() as u32,
        };
        (&mut frame as &mut dyn MutSpace).write(&body, false)?;

        Some(VectorWriter {
            frame,
            type_: PhantomData,
        })
    }
}

/// An atom containing a chunk of memory with undefined contents.
///
/// This atom is specified [here](http://lv2plug.in/ns/ext/atom/atom.html#Chunk).
pub struct Chunk;

unsafe impl UriBound for Chunk {
    const URI: &'static [u8] = sys::LV2_ATOM__Chunk;
}

impl URIDBound for Chunk {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.chunk
    }
}

impl Chunk {
    /// Read a chunk of bytes.
    ///
    /// The returned slice is the body of the atom and the space is the space behind the atom.
    pub fn read<'a>(space: Space<'a>, urids: &AtomURIDCache) -> Option<(&'a [u8], Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.chunk)?;
        Some((body.data()?, space))
    }

    /// Initialize a chunk atom.
    ///
    /// This method creates an empty chunk and returns a writer to add more data to the chunk.
    pub fn write<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        space.create_atom_frame(urids.chunk)
    }
}

/// Either the language or the datatype of a literal.
///
/// A literal is either a UTF8 string or another type. If it's a string, it has a language, and if it's a value of another type, the exact type has to be known. This enum covers these options.
#[derive(Clone, Copy)]
pub enum LiteralType {
    Language(URID),
    Datatype(URID),
}

impl TryFrom<&sys::LV2_Atom_Literal_Body> for LiteralType {
    type Error = ();

    fn try_from(body: &sys::LV2_Atom_Literal_Body) -> Result<Self, ()> {
        let lang = URID::try_from(body.lang).map(Self::Language);
        let datatype = URID::try_from(body.datatype).map(Self::Datatype);
        lang.or(datatype)
    }
}

impl From<LiteralType> for sys::LV2_Atom_Literal_Body {
    fn from(literal_type: LiteralType) -> Self {
        match literal_type {
            LiteralType::Language(urid) => sys::LV2_Atom_Literal_Body {
                lang: urid.get(),
                datatype: 0,
            },
            LiteralType::Datatype(urid) => sys::LV2_Atom_Literal_Body {
                lang: 0,
                datatype: urid.get(),
            },
        }
    }
}

/// An atom that holds either a UTF8 string or a value of another type.
///
/// Usually, a literal is just a localized UTF8 string. However, the [specification](http://lv2plug.in/ns/ext/atom/atom.html#Literal) also leaves space for the literal to hold a value of an arbitrary type. Therefore, this implementation provides convenient methods to read/write strings and not-so-convenient methods to read/write arbitrary data.
pub struct Literal;

unsafe impl UriBound for Literal {
    const URI: &'static [u8] = sys::LV2_ATOM__Literal;
}

impl URIDBound for Literal {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.literal
    }
}

impl Literal {
    /// Read a literal containing a localised UTF8 string.
    ///
    /// If the space is big enough and contains a string literal, the method returns the URID of the language, the string itself and the space behind the atom.
    pub fn read_str<'a>(
        space: Space<'a>,
        urids: &AtomURIDCache,
    ) -> Option<(URID, &'a str, Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.literal)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        if let Ok(LiteralType::Language(urid)) = LiteralType::try_from(header) {
            let data = body.data()?;
            std::str::from_utf8(&data[0..data.len() - 1])
                .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
                .ok()
                .map(|string| (urid, string, space))
        } else {
            None
        }
    }

    /// Read a literal.
    ///
    /// If the space is big enough and contains a literal, the method returns the datatype/language of the literal, the body data of the literal and the space behind the atom.
    pub fn read<'a>(
        space: Space<'a>,
        urids: &AtomURIDCache,
    ) -> Option<(LiteralType, Space<'a>, Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.literal)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        let literal_type = LiteralType::try_from(header).ok()?;
        Some((literal_type, body, space))
    }

    /// Create an atom frame of a literal atom, write the literal body and return the frame.
    fn write_body<'a, 'b>(
        space: &'a mut dyn MutSpace<'b>,
        literal_type: LiteralType,
        urid: URID<Literal>,
    ) -> Option<FramedMutSpace<'b, 'a>> {
        let mut frame = space.create_atom_frame(urid)?;
        (&mut frame as &mut dyn MutSpace)
            .write(&sys::LV2_Atom_Literal_Body::from(literal_type), false)?;
        Some(frame)
    }

    /// Initialize a string literal atom.
    ///
    /// This method creates an empty string literal with the given language and returns a writer to append slices to it.
    ///
    /// If the space is not big enough, this method returns `None`.
    pub fn write_str<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        lang: URID,
    ) -> Option<LiteralWriter<'a, 'b>> {
        let frame = Self::write_body(space, LiteralType::Language(lang), urids.literal)?;
        Some(LiteralWriter { frame })
    }

    /// Initialize a literal.
    ///
    /// This method creates an empty literal with the given datatype or language and returns a framed space to append data to it.
    ///
    /// If the space is not big enough, this method returns `None`.
    pub fn write<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        datatype: URID,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        Self::write_body(space, LiteralType::Datatype(datatype), urids.literal)
    }
}

/// Handle to append strings to a string literal.
pub struct LiteralWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
}

impl<'a, 'b> LiteralWriter<'a, 'b> {
    /// Append a string to the literal.
    ///
    /// This method copies the given string to the end of the literal and then returns a mutable reference to the copy.
    ///
    /// If the internal space for the literal is not big enough, this method returns `None`.
    pub fn append(&mut self, string: &str) -> Option<&mut str> {
        let data = string.as_bytes();
        let space = self.frame.write_raw(data, false)?;
        unsafe { Some(std::str::from_utf8_unchecked_mut(space)) }
    }
}

impl<'a, 'b> Drop for LiteralWriter<'a, 'b> {
    fn drop(&mut self) {
        // Null terminator.
        (&mut self.frame as &mut dyn MutSpace).write(&0u8, false);
    }
}

/// Handle to append elements to a vector.
///
/// This works by allocating a slice of memory behind the vector and then writing your data to it.
pub struct VectorWriter<'a, 'b, T>
where
    T: Unpin + Copy + Send + Sync + Sized + 'static,
{
    frame: FramedMutSpace<'a, 'b>,
    type_: PhantomData<T>,
}

impl<'a, 'b, T> VectorWriter<'a, 'b, T>
where
    T: Unpin + Copy + Send + Sync + Sized + 'static,
{
    /// Push a single value to the vector.
    pub fn push(&mut self, child: T) -> Option<&mut T> {
        (&mut self.frame as &mut dyn MutSpace).write(&child, false)
    }

    /// Append a slice of undefined memory to the vector.
    ///
    /// Using this method, you don't need to have the elements in memory before you can write them.
    pub fn allocate(&mut self, size: usize) -> Option<&mut [T]> {
        self.frame
            .allocate(size_of::<T>() * size, false)
            .map(|data| unsafe {
                std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut T, size)
            })
    }

    /// Append multiple elements to the vector.
    pub fn append(&mut self, data: &[T]) -> Option<&mut [T]> {
        let raw_data = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };
        self.frame
            .allocate(raw_data.len(), false)
            .map(|space| unsafe {
                space.copy_from_slice(raw_data);
                std::slice::from_raw_parts_mut(space.as_mut_ptr() as *mut T, data.len())
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use crate::*;
    use core::UriBound;
    use std::ffi::CStr;
    use std::mem::size_of;
    use urid::URIDCache;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = Vector::write::<Int>(&mut space, &urids, &urids).unwrap();
            writer.append(&[42; CHILD_COUNT - 1]);
            writer.push(1);
        }

        // verifying
        {
            let (vector, children) = raw_space.split_at(size_of::<sys::LV2_Atom_Vector>());

            let vector = unsafe { &*(vector.as_ptr() as *const sys::LV2_Atom_Vector) };
            assert_eq!(vector.atom.type_, urids.vector.get());
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * CHILD_COUNT
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int.get());

            let children =
                unsafe { std::slice::from_raw_parts(children.as_ptr() as *const i32, CHILD_COUNT) };
            for value in &children[0..children.len() - 1] {
                assert_eq!(*value, 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }

        // reading
        {
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };
            let children: &[i32] = Vector::read::<Int>(space, &urids, &urids).unwrap().0;

            assert_eq!(children.len(), CHILD_COUNT);
            for i in 0..children.len() - 1 {
                assert_eq!(children[i], 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }
    }

    #[test]
    fn test_chunk_and_slice_writer() {
        const SLICE_LENGTH: usize = 42;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = Chunk::write(&mut space, &urids).unwrap();

            for (i, value) in (&mut writer as &mut dyn MutSpace)
                .allocate(SLICE_LENGTH - 1, false)
                .unwrap()
                .into_iter()
                .enumerate()
            {
                *value = i as u8;
            }
            (&mut writer as &mut dyn MutSpace)
                .write(&41u8, false)
                .unwrap();
        }

        // verifying
        {
            let (atom, data) = raw_space.split_at(size_of::<sys::LV2_Atom>());

            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.size as usize, SLICE_LENGTH);
            assert_eq!(atom.type_, urids.chunk.get());

            let data = data.split_at(SLICE_LENGTH).0;
            for i in 0..SLICE_LENGTH {
                assert_eq!(data[i] as usize, i);
            }
        }

        // reading
        {
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };

            let data = Chunk::read(space, &urids).unwrap().0;
            assert_eq!(data.len(), SLICE_LENGTH);

            for (i, value) in data.iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }
    }

    #[test]
    fn test_literal() {
        struct German;
        unsafe impl UriBound for German {
            const URI: &'static [u8] = b"http://lexvo.org/id/iso639-1/de\0";
        }

        #[derive(URIDCache)]
        pub struct TestURIDs {
            atom: AtomURIDCache,
            german: URID<German>,
        }

        const SAMPLE: &str = "Da steh ich nun, ich armer Tor! Und bin so klug als wie zuvor;";

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = TestURIDs::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer =
                Literal::write_str(&mut space, &urids.atom, urids.german.into_general()).unwrap();
            writer.append(SAMPLE).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Literal>());

            let literal = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom_Literal) };
            assert_eq!(literal.atom.type_, urids.atom.literal.get());
            assert_eq!(literal.body.lang, urids.german.get());
            assert_eq!(literal.body.datatype, 0);

            let size = literal.atom.size as usize - size_of::<sys::LV2_Atom_Literal_Body>();
            let string = CStr::from_bytes_with_nul(space.split_at(size).0)
                .unwrap()
                .to_str()
                .unwrap();
            assert_eq!(SAMPLE, string);
        }

        // reading
        {
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };
            let (lang, text, _) = Literal::read_str(space, &urids.atom).unwrap();

            assert_eq!(lang, urids.german);
            assert_eq!(text, SAMPLE);
        }
    }
}
