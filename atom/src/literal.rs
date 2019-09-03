use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use std::convert::TryFrom;
use urid::{URIDBound, URID};

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

#[cfg(test)]
mod tests {
    use crate::literal::*;
    use core::UriBound;
    use std::ffi::CStr;
    use std::mem::size_of;
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

    #[test]
    fn test_literal() {
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
