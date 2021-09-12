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
//! use lv2_atom::atoms::string::StringWriter;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
//!     let input: &str = ports.input.read(urids.string).unwrap();
//!     let mut writer: StringWriter = ports.output.init(urids.string).unwrap();
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
use crate::AtomHandle;
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

impl LiteralInfo {
    fn try_from_raw(header: &sys::LV2_Atom_Literal_Body) -> Option<Self> {
        if header.lang != 0 && header.datatype == 0 {
            Some(LiteralInfo::Language(URID::new(header.lang)?))
        } else if header.lang == 0 && header.datatype != 0 {
            Some(LiteralInfo::Datatype(URID::new(header.datatype)?))
        } else {
            None
        }
    }

    fn into_raw(self) -> sys::LV2_Atom_Literal_Body {
        match self {
            LiteralInfo::Language(lang) => sys::LV2_Atom_Literal_Body {
                lang: lang.get(),
                datatype: 0,
            },
            LiteralInfo::Datatype(datatype) => sys::LV2_Atom_Literal_Body {
                lang: 0,
                datatype: datatype.get(),
            },
        }
    }
}

pub struct LiteralInfoWriter<'a> {
    writer: AtomSpaceWriter<'a>,
}

impl<'a> LiteralInfoWriter<'a> {
    pub fn write_info(mut self, info: LiteralInfo) -> Result<StringWriter<'a>, AtomError> {
        self.writer.write_value(info.into_raw())?;

        Ok(StringWriter {
            writer: self.writer.terminated(0),
        })
    }
}

pub struct LiteralReadHandle;

impl<'a> AtomHandle<'a> for LiteralReadHandle {
    type Handle = (LiteralInfo, &'a str);
}

pub struct LiteralWriteHandle;

impl<'a> AtomHandle<'a> for LiteralWriteHandle {
    type Handle = LiteralInfoWriter<'a>;
}

impl Atom for Literal {
    type ReadHandle = LiteralReadHandle;
    type WriteHandle = LiteralWriteHandle;

    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomError> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Literal_Body = reader.next_value()?;

        let info =
            LiteralInfo::try_from_raw(header).ok_or_else(|| AtomError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
            })?;

        let data = reader.remaining_bytes();

        std::str::from_utf8(&data[0..data.len() - 1])
            .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
            .map_err(|_| AtomError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
            })
            .map(|string| (info, string))
    }

    #[inline]
    fn init(
        frame: AtomSpaceWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomError> {
        Ok(LiteralInfoWriter { writer: frame })
    }
}

pub struct StringReadHandle;

impl<'a> AtomHandle<'a> for StringReadHandle {
    type Handle = &'a str;
}

pub struct StringWriteHandle;

impl<'a> AtomHandle<'a> for StringWriteHandle {
    type Handle = StringWriter<'a>;
}

/// An atom containing a UTF-8 encoded string.
///
/// [See also the module documentation.](index.html)
pub struct String;

unsafe impl UriBound for String {
    const URI: &'static [u8] = sys::LV2_ATOM__String;
}

impl Atom for String {
    type ReadHandle = StringReadHandle;
    type WriteHandle = StringWriteHandle;

    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomError> {
        let data = body.as_bytes();
        let rust_str_bytes =
            data.get(..data.len() - 1)
                .ok_or_else(|| AtomError::InvalidAtomValue {
                    reading_type_uri: Self::uri(),
                })?; // removing the null-terminator
        Ok(
            core::str::from_utf8(rust_str_bytes).map_err(|_| AtomError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
            })?,
        )
    }

    fn init(
        frame: AtomSpaceWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomError> {
        Ok(StringWriter {
            writer: frame.terminated(0),
        })
    }
}

/// Handle to append strings to a string or literal.
pub struct StringWriter<'a> {
    writer: Terminated<AtomSpaceWriter<'a>>,
}

impl<'a> StringWriter<'a> {
    /// Append a string.
    ///
    /// This method copies the given string to the end of the string atom/literal and then returns a mutable reference to the copy.
    ///
    /// If the internal space for the atom is not big enough, this method returns `None`.
    pub fn append(&mut self, string: &str) -> Result<&mut str, AtomError> {
        let bytes = self.writer.write_bytes(string.as_bytes())?;

        // SAFETY: We just wrote that string, therefore it is guaranteed to be valid UTF-8
        unsafe { Ok(std::str::from_utf8_unchecked_mut(bytes)) }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use crate::AtomHeader;
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
        let urids: TestURIDs = TestURIDs::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());

            let mut writer = space
                .init_atom(urids.atom.literal)
                .unwrap()
                .write_info(LiteralInfo::Language(urids.german.into_general()))
                .unwrap();

            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let mut reader = raw_space.read();
            let literal: &sys::LV2_Atom_Literal = unsafe { reader.next_value() }.unwrap();

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
            let string = CStr::from_bytes_with_nul(reader.next_bytes(size).unwrap())
                .unwrap()
                .to_str()
                .unwrap();
            assert_eq!(SAMPLE0.to_owned() + SAMPLE1, string);
        }

        // reading
        {
            let (info, text) = unsafe {
                raw_space
                    .read()
                    .next_atom()
                    .unwrap()
                    .read(urids.atom.literal)
            }
            .unwrap();

            assert_eq!(info, LiteralInfo::Language(urids.german.into_general()));
            assert_eq!(text, SAMPLE0.to_owned() + SAMPLE1);
        }
    }

    #[test]
    fn test_string() {
        let map = HashURIDMapper::new();
        let urids = crate::atoms::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());

            let mut writer = space.init_atom(urids.string).unwrap();
            writer.append(SAMPLE0).unwrap();
            writer.append(SAMPLE1).unwrap();
        }

        // verifying
        {
            let mut reader = raw_space.read();
            let string: &sys::LV2_Atom_String = unsafe { reader.next_value() }.unwrap();
            assert_eq!(string.atom.type_, urids.string);
            assert_eq!(string.atom.size as usize, SAMPLE0.len() + SAMPLE1.len() + 1);

            let string =
                std::str::from_utf8(reader.next_bytes(string.atom.size as usize).unwrap()).unwrap();
            assert_eq!(string[..string.len() - 1], SAMPLE0.to_owned() + SAMPLE1);
        }

        // reading
        {
            let string = unsafe { raw_space.read().next_atom() }
                .unwrap()
                .read(urids.string)
                .unwrap();
            assert_eq!(string, SAMPLE0.to_owned() + SAMPLE1);
        }
    }
}
