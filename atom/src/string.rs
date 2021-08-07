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
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
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
use urid::*;

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

impl<'a, 'b> Atom<'a, 'b> for Literal {
    type ReadParameter = ();
    type ReadHandle = (LiteralInfo, &'a str);
    type WriteParameter = LiteralInfo;
    type WriteHandle = StringWriter<'b>;

    unsafe fn read(body: &'a Space, _: ()) -> Option<(LiteralInfo, &'a str)> {
        let (header, body) = body.split_for_value_as_unchecked::<sys::LV2_Atom_Literal_Body>()?;
        let info = if header.lang != 0 && header.datatype == 0 {
            LiteralInfo::Language(URID::new(header.lang)?)
        } else if header.lang == 0 && header.datatype != 0 {
            LiteralInfo::Datatype(URID::new(header.datatype)?)
        } else {
            return None;
        };
        let data = body.as_bytes();
        std::str::from_utf8(&data[0..data.len() - 1])
            .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
            .ok()
            .map(|string| (info, string))
    }

    fn init(mut frame: AtomSpaceWriter<'b>, info: LiteralInfo) -> Option<StringWriter<'b>> {
        crate::space::write_value(&mut frame,
            match info {
                LiteralInfo::Language(lang) => sys::LV2_Atom_Literal_Body {
                    lang: lang.get(),
                    datatype: 0,
                },
                LiteralInfo::Datatype(datatype) => sys::LV2_Atom_Literal_Body {
                    lang: 0,
                    datatype: datatype.get(),
                },
            }
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

impl<'a, 'b> Atom<'a, 'b> for String {
    type ReadParameter = ();
    type ReadHandle = &'a str;
    type WriteParameter = ();
    type WriteHandle = StringWriter<'b>;

    unsafe fn read(body: &'a Space, _: ()) -> Option<&'a str> {
        let data = body.as_bytes();
        let rust_str_bytes = data.get(..data.len() - 1)?; // removing the null-terminator
        Some(core::str::from_utf8(rust_str_bytes).ok()?)
    }

    fn init(frame: AtomSpaceWriter<'b>, _: ()) -> Option<StringWriter<'b>> {
        Some(StringWriter { frame })
    }
}

/// Handle to append strings to a string or literal.
pub struct StringWriter<'a> {
    frame: AtomSpaceWriter<'a>,
}

impl<'a, 'b> StringWriter<'a> {
    /// Append a string.
    ///
    /// This method copies the given string to the end of the string atom/literal and then returns a mutable reference to the copy.
    ///
    /// If the internal space for the atom is not big enough, this method returns `None`.
    pub fn append(&'a mut self, string: &str) -> Option<&'a mut str> {
        let data = string.as_bytes();
        let space = crate::space::write_bytes(&mut self.frame, data)?;
        unsafe { Some(std::str::from_utf8_unchecked_mut(space)) }
    }
}

impl<'a> Drop for StringWriter<'a> {
    fn drop(&mut self) {
        // Null terminator.
        // FIXME: this seems unsafe if the value could not be written for some reason.
        let _ = crate::space::write_value(&mut self.frame, 0u8);
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use std::ffi::CStr;
    use std::mem::{size_of, size_of_val};
    use urid::*;

    struct German;
    unsafe impl UriBound for German {
        const URI: &'static [u8] = b"http://lexvo.org/id/iso639-1/de\0";
    }

    #[derive(URIDCollection)]
    pub struct TestURIDs {
        atom: AtomURIDCollection,
        german: URID<German>,
    }

    const SAMPLE0: &str = "Da steh ich nun, ich armer Tor! ";
    const SAMPLE1: &str = "Und bin so klug als wie zuvor;";

    #[test]
    fn test_literal() {
        let map = HashURIDMapper::new();
        let urids = TestURIDs::from_map(&map).unwrap();

        let mut raw_space = Space::boxed(256);

        // writing
        {
            let mut space = raw_space.as_bytes_mut();
            let mut writer = crate::space::init_atom(&mut space, urids.atom.literal, LiteralInfo::Language(urids.german.into_general())).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let (literal, space) = unsafe { raw_space.split_for_value_as_unchecked::<sys::LV2_Atom_Literal>() }.unwrap();

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
            let string = CStr::from_bytes_with_nul(space.split_at(size).unwrap().0.as_bytes())
                .unwrap()
                .to_str()
                .unwrap();
            assert_eq!(SAMPLE0.to_owned() + SAMPLE1, string);
        }

        // reading
        {
            let (body, _) = unsafe { raw_space.split_atom_body(urids.atom.literal) }.unwrap();
            let (info, text) = unsafe { Literal::read(body, ()) }.unwrap();

            assert_eq!(info, LiteralInfo::Language(urids.german.into_general()));
            assert_eq!(text, SAMPLE0.to_owned() + SAMPLE1);
        }
    }

    #[test]
    fn test_string() {
        let map = HashURIDMapper::new();
        let urids = crate::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = Space::boxed(256);

        // writing
        {
            let mut space = raw_space.as_bytes_mut();

            let mut writer = crate::space::init_atom(&mut space, urids.string, ()).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let (string, space) = unsafe { raw_space.split_for_value_as_unchecked::<sys::LV2_Atom_String>() }.unwrap();
            assert_eq!(string.atom.type_, urids.string);
            assert_eq!(string.atom.size as usize, SAMPLE0.len() + SAMPLE1.len() + 1);

            let string = std::str::from_utf8(space.split_at(string.atom.size as usize).unwrap().0.as_bytes()).unwrap();
            assert_eq!(string[..string.len() - 1], SAMPLE0.to_owned() + SAMPLE1);
        }

        // reading
        {
            let (body, _) = unsafe { raw_space.split_atom_body(urids.string) }.unwrap();
            let string = unsafe { String::read(body, ()) }.unwrap();
            assert_eq!(string, SAMPLE0.to_owned() + SAMPLE1);
        }
    }
}
