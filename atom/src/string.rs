//! String handling atoms.
//!
//! This module contains two different atoms: The [`String`](struct.String.html) and the [`Literal`](struct.Literal.html). The former is for simple, non-localized UTF-8 strings, like URIs or paths, and the later is either for localized text, e.g. descriptions in the user interface, or RDF literals.
//!
//! Reading and writing these atoms is pretty simple: They don't require a parameter and return a either a `&str` or the literal info and a `&str`. Writing is done with a writing handle which can append strings to the string/literal. When dropped, the handle will append the null character, you therefore don't have to handle it on your own.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::string::StringWriter;
//! use lv2_urid::prelude::*;
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCache) {
//!     let input: &str = ports.input.read(urids.string, ()).unwrap();
//!     let mut writer: StringWriter = ports.output.init(urids.string, ()).unwrap();
//!     writer.append(input).unwrap();
//! }
//! ```
//!
//! # Specifications
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#String](http://lv2plug.in/ns/ext/atom/atom.html#String)
//! [http://lv2plug.in/ns/ext/atom/atom.html#Literal](http://lv2plug.in/ns/ext/atom/atom.html#Literal)
use crate::prelude::*;
use crate::space::*;
use core::prelude::*;
use urid::prelude::*;

/// An atom containing either a localized string or an RDF literal.
///
/// [See also the module documentation.](index.html)
pub struct Literal;

unsafe impl UriBound for Literal {
    const URI: &'static [u8] = sys::LV2_ATOM__Literal;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// The type or language URID of a literal.
pub enum LiteralInfo {
    Language(URID),
    Datatype(URID),
}

impl<'a, 'b> Atom<'a, 'b> for Literal
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = (LiteralInfo, &'a str);
    type WriteParameter = LiteralInfo;
    type WriteHandle = StringWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<(LiteralInfo, &'a str)> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Literal_Body>()?;
        let info = if header.lang != 0 && header.datatype == 0 {
            LiteralInfo::Language(URID::new(header.lang)?)
        } else if header.lang == 0 && header.datatype != 0 {
            LiteralInfo::Datatype(URID::new(header.datatype)?)
        } else {
            return None;
        };
        let data = body.data()?;
        std::str::from_utf8(&data[0..data.len() - 1])
            .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
            .ok()
            .map(|string| (info, string))
    }

    fn init(mut frame: FramedMutSpace<'a, 'b>, info: LiteralInfo) -> Option<StringWriter<'a, 'b>> {
        (&mut frame as &mut dyn MutSpace).write(
            &match info {
                LiteralInfo::Language(lang) => sys::LV2_Atom_Literal_Body {
                    lang: lang.get(),
                    datatype: 0,
                },
                LiteralInfo::Datatype(datatype) => sys::LV2_Atom_Literal_Body {
                    lang: 0,
                    datatype: datatype.get(),
                },
            },
            true,
        )?;
        Some(StringWriter { frame })
    }
}

/// An atom containing a UTF-8 encoded string.
///
/// [See also the module documentation.](index.html)
pub struct String;

unsafe impl UriBound for String {
    const URI: &'static [u8] = sys::LV2_ATOM__String;
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

    fn init(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<StringWriter<'a, 'b>> {
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
    use urid::mapper::*;
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
    fn test_literal() {
        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = TestURIDs::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.atom.literal)
                .unwrap();
            let mut writer =
                Literal::init(frame, LiteralInfo::Language(urids.german.into_general())).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Literal>());

            let literal = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom_Literal) };
            assert_eq!(literal.atom.type_, urids.atom.literal.get());
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
            let (body, _) = space.split_atom_body(urids.atom.literal).unwrap();
            let (info, text) = Literal::read(body, ()).unwrap();

            assert_eq!(info, LiteralInfo::Language(urids.german.into_general()));
            assert_eq!(text, SAMPLE0.to_owned() + SAMPLE1);
        }
    }

    #[test]
    fn test_string() {
        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.string)
                .unwrap();
            let mut writer = String::init(frame, ()).unwrap();
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
