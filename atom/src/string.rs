use crate::chunk::ByteWriter;
use crate::prelude::*;
use crate::space::*;
use core::prelude::*;
use urid::prelude::*;

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

impl<'a, 'b> Atom<'a, 'b> for StringLiteral
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = (URID, &'a str);
    type WriteParameter = URID;
    type WriteHandle = StringWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<(URID, &'a str)> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        if header.lang != 0 && header.datatype == 0 {
            let data = body.data()?;
            let lang = URID::new(header.lang)?;
            std::str::from_utf8(&data[0..data.len() - 1])
                .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
                .ok()
                .map(|string| (lang, string))
        } else {
            None
        }
    }

    fn write(mut frame: FramedMutSpace<'a, 'b>, lang: URID) -> Option<StringWriter<'a, 'b>> {
        (&mut frame as &mut dyn MutSpace).write(
            &sys::LV2_Atom_Literal_Body {
                lang: lang.get(),
                datatype: 0,
            },
            true,
        )?;
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

impl<'a, 'b> Atom<'a, 'b> for DataLiteral
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = (URID, &'a [u8]);
    type WriteParameter = URID;
    type WriteHandle = ByteWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<(URID, &'a [u8])> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        if header.lang == 0 && header.datatype != 0 {
            let data = body.data()?;
            let urid = unsafe { URID::new_unchecked(header.datatype) };
            Some((urid, data))
        } else {
            None
        }
    }

    fn write(mut frame: FramedMutSpace<'a, 'b>, datatype: URID) -> Option<ByteWriter<'a, 'b>> {
        (&mut frame as &mut dyn MutSpace).write(
            &sys::LV2_Atom_Literal_Body {
                lang: 0,
                datatype: datatype.get(),
            },
            true,
        )?;
        Some(ByteWriter::new(frame))
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

impl<'a, 'b> Atom<'a, 'b> for String
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = &'a str;
    type WriteParameter = ();
    type WriteHandle = StringWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<&'a str> {
        body.data()
            .and_then(|data| std::str::from_utf8(data).ok())
            .map(|string| &string[..string.len() - 1]) // removing the null-terminator
    }

    fn write(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<StringWriter<'a, 'b>> {
        Some(StringWriter { frame })
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
    use crate::prelude::*;
    use crate::space::*;
    use core::prelude::*;
    use std::ffi::CStr;
    use std::mem::{size_of, size_of_val};
    use urid::mapper::HashURIDMapper;
    use urid::prelude::*;

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
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.atom.string_literal)
                .unwrap();
            let mut writer = StringLiteral::write(frame, urids.german.into_general()).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Literal>());

            let literal = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom_Literal) };
            assert_eq!(literal.atom.type_, urids.atom.string_literal.get());
            assert_eq!(
                literal.atom.size as usize,
                size_of::<sys::LV2_Atom_Literal_Body>()
                    + size_of_val(SAMPLE0)
                    + size_of_val(SAMPLE1)
                    + 1
            );
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
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.atom.string_literal).unwrap();
            let (lang, text) = StringLiteral::read(body, ()).unwrap();

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
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.atom.data_literal)
                .unwrap();
            let mut writer = DataLiteral::write(frame, urids.atom.int.into_general()).unwrap();
            writer.write::<i32>(&42).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Literal>());

            let literal = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom_Literal) };
            assert_eq!(literal.atom.type_, urids.atom.string_literal.get());
            assert_eq!(
                literal.atom.size as usize,
                size_of::<sys::LV2_Atom_Literal_Body>() + size_of::<i32>()
            );
            assert_eq!(literal.body.lang, 0);
            assert_eq!(literal.body.datatype, urids.atom.int);

            let int = unsafe { *(space.as_ptr() as *const i32) };
            assert_eq!(int, 42);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.atom.data_literal).unwrap();
            let (datatype, value) = DataLiteral::read(body, ()).unwrap();

            assert_eq!(datatype, urids.atom.int);
            assert_eq!(unsafe { *(value.as_ptr() as *const i32) }, 42);
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
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.string)
                .unwrap();
            let mut writer = String::write(frame, ()).unwrap();
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
            let (body, _) = space.split_atom_body(urids.string).unwrap();
            let string = String::read(body, ()).unwrap();
            assert_eq!(string, SAMPLE0.to_owned() + SAMPLE1);
        }
    }
}
