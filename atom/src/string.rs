use crate::space::*;
use crate::*;
use core::UriBound;
use std::convert::TryFrom;
use urid::{URIDBound, URID};

pub struct StringLiteral;

unsafe impl UriBound for StringLiteral {
    const URI: &'static [u8] = sys::LV2_ATOM__Literal;
}

impl URIDBound for StringLiteral {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.string_literal
    }
}

impl<'a, 'b> Atom<'a, 'b> for StringLiteral where 'a: 'b {
    type ReadParameter = ();
    type ReadHandle = (URID, &'a str);
    type WriteParameter = URID;
    type WriteHandle = StringWriter<'a, 'b>;

    fn read(
        space: Space<'a>,
        _: (),
        urids: &AtomURIDCache,
    ) -> Option<((URID, &'a str), Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.string_literal)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        if let Ok(LiteralType::Language(urid)) = LiteralType::try_from(header) {
            let data = body.data()?;
            std::str::from_utf8(&data[0..data.len() - 1])
                .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
                .ok()
                .map(|string| ((urid, string), space))
        } else {
            None
        }
    }

    fn write(
        space: &'b mut dyn MutSpace<'a>,
        lang: URID,
        urids: &AtomURIDCache,
    ) -> Option<StringWriter<'a, 'b>> {
        let mut frame = space.create_atom_frame(urids.string_literal)?;
        (&mut frame as &mut dyn MutSpace).write(&sys::LV2_Atom_Literal_Body {
            lang: lang.get(),
            datatype: 0
        }, true)?;
        Some(StringWriter { frame })
    }
}

pub struct DataLiteral;

unsafe impl UriBound for DataLiteral {
    const URI: &'static [u8] = sys::LV2_ATOM__Literal;
}

impl URIDBound for DataLiteral {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.data_literal
    }
}

impl<'a, 'b> Atom<'a, 'b> for DataLiteral where 'a: 'b {
    type ReadParameter = ();
    type ReadHandle = (URID, &'a [u8]);
    type WriteParameter = URID;
    type WriteHandle = FramedMutSpace<'a, 'b>;

    fn read(
        space: Space<'a>,
        _: (),
        urids: &AtomURIDCache,
    ) -> Option<((URID, &'a [u8]), Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.data_literal)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        if let Ok(LiteralType::Datatype(urid)) = LiteralType::try_from(header) {
            let data = body.data()?;
            Some(((urid, data), space))
        } else {
            None
        }
    }

    fn write(
        space: &'b mut dyn MutSpace<'a>,
        datatype: URID,
        urids: &AtomURIDCache,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        let mut frame = space.create_atom_frame(urids.string_literal)?;
        (&mut frame as &mut dyn MutSpace).write(&sys::LV2_Atom_Literal_Body {
            lang: 0,
            datatype: datatype.get(),
        }, true)?;
        Some(frame)
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

/// An atom containing a UTF-8 encoded string.
///
/// This string type is for technical, free-form strings, for example URIs. If you want to transfer localizable display text, you should use the [`Literal`](struct.Literal.html) type, as described in the [specification](http://lv2plug.in/ns/ext/atom/atom.html#String).
pub struct String;

unsafe impl UriBound for String {
    const URI: &'static [u8] = sys::LV2_ATOM__String;
}

impl URIDBound for String {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.string
    }
}

impl<'a, 'b> Atom<'a, 'b> for String where 'a: 'b {
    type ReadParameter = ();
    type ReadHandle = &'a str;
    type WriteParameter = ();
    type WriteHandle = StringWriter<'a, 'b>;
    
    fn read(space: Space<'a>, _: (), urids: &AtomURIDCache) -> Option<(&'a str, Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.string)?;
        body.data()
            .and_then(|data| std::str::from_utf8(data).ok())
            .map(|string| &string[..string.len() - 1]) // removing the null-terminator
            .map(|string| (string, space))
    }

    fn write(
        space: &'b mut dyn MutSpace<'a>,
        _: (),
        urids: &AtomURIDCache,
    ) -> Option<StringWriter<'a, 'b>> {
        space
            .create_atom_frame(urids.string)
            .map(|frame| StringWriter { frame })
    }
}

/// Handle to append strings to a string or literal.
pub struct StringWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
}

impl<'a, 'b> StringWriter<'a, 'b> {
    /// Append a string.
    ///
    /// This method copies the given string to the end of the string atom/literal and then returns a mutable reference to the copy.
    ///
    /// If the internal space for the atom is not big enough, this method returns `None`.
    pub fn append(&mut self, string: &str) -> Option<&mut str> {
        let data = string.as_bytes();
        let space = self.frame.write_raw(data, false)?;
        unsafe { Some(std::str::from_utf8_unchecked_mut(space)) }
    }
}

impl<'a, 'b> Drop for StringWriter<'a, 'b> {
    fn drop(&mut self) {
        // Null terminator.
        (&mut self.frame as &mut dyn MutSpace).write(&0u8, false);
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::string::*;
    use core::UriBound;
    use std::ffi::CStr;
    use std::mem::{size_of, size_of_val};
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    struct German;
    unsafe impl UriBound for German {
        const URI: &'static [u8] = b"http://lexvo.org/id/iso639-1/de\0";
    }

    #[derive(URIDCache)]
    pub struct TestURIDs {
        atom: AtomURIDCache,
        german: URID<German>,
    }

    const SAMPLE0: &str = "Da steh ich nun, ich armer Tor! ";
    const SAMPLE1: &str = "Und bin so klug als wie zuvor;";

    #[test]
    fn test_string_literal() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = TestURIDs::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer =
                StringLiteral::write(&mut space, urids.german.into_general(), &urids.atom).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Literal>());

            let literal = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom_Literal) };
            assert_eq!(literal.atom.type_, urids.atom.string_literal.get());
            assert_eq!(literal.atom.size as usize, size_of::<sys::LV2_Atom_Literal_Body>() + size_of_val(SAMPLE0) + size_of_val(SAMPLE1) + 1);
            assert_eq!(literal.body.lang, urids.german.get());
            assert_eq!(literal.body.datatype, 0);

            let size = literal.atom.size as usize - size_of::<sys::LV2_Atom_Literal_Body>();
            let string = CStr::from_bytes_with_nul(space.split_at(size).0)
                .unwrap()
                .to_str()
                .unwrap();
            assert_eq!(SAMPLE0.to_owned() + SAMPLE1, string);
        }

        // reading
        {
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };
            let ((lang, text), _) = StringLiteral::read(space, (), &urids.atom).unwrap();

            assert_eq!(lang, urids.german);
            assert_eq!(text, SAMPLE0.to_owned() + SAMPLE1);
        }
    }

    #[test]
    fn test_data_literal() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = TestURIDs::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut frame =
                DataLiteral::write(&mut space, urids.atom.int.into_general(), &urids.atom).unwrap();
            (&mut frame as &mut dyn MutSpace).write::<i32>(&42, true).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Literal>());

            let literal = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom_Literal) };
            assert_eq!(literal.atom.type_, urids.atom.string_literal.get());
            assert_eq!(literal.atom.size as usize, size_of::<sys::LV2_Atom_Literal_Body>() + size_of::<i32>());
            assert_eq!(literal.body.lang, 0);
            assert_eq!(literal.body.datatype, urids.atom.int);

            let int = unsafe {*(space.as_ptr() as *const i32)};
            assert_eq!(int, 42);
        }

        // reading
        {
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };
            let ((datatype, value), _) = DataLiteral::read(space, (), &urids.atom).unwrap();

            assert_eq!(datatype, urids.atom.int);
            assert_eq!(unsafe {*(value.as_ptr() as *const i32)}, 42);
        }
    }

    #[test]
    fn test_string() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = String::write(&mut space, (), &urids).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let (string, space) = raw_space.split_at(size_of::<sys::LV2_Atom_String>());

            let string = unsafe { &*(string.as_ptr() as *const sys::LV2_Atom_String) };
            assert_eq!(string.atom.type_, urids.string);
            assert_eq!(string.atom.size as usize, SAMPLE0.len() + SAMPLE1.len() + 1);

            let string = std::str::from_utf8(space.split_at(string.atom.size as usize).0).unwrap();
            assert_eq!(string[..string.len() - 1], SAMPLE0.to_owned() + SAMPLE1);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let string = String::read(space, (), &urids).unwrap().0;
            assert_eq!(string, SAMPLE0.to_owned() + SAMPLE1);
        }
    }
}
