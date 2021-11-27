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
//!     let mut writer: StringWriter = ports.output.write(urids.string).unwrap();
//!     writer.append(input).unwrap();
//! }
//! ```
//!
//! # Specifications
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#String](http://lv2plug.in/ns/ext/atom/atom.html#String)
//! [http://lv2plug.in/ns/ext/atom/atom.html#Literal](http://lv2plug.in/ns/ext/atom/atom.html#Literal)
use crate::prelude::*;
use crate::space::error::{AtomReadError, AtomWriteError};
use crate::space::*;
use crate::AtomHandle;
use std::ffi::CStr;
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
    fn try_from_raw(header: &sys::LV2_Atom_Literal_Body) -> Result<Self, &'static str> {
        match (URID::new(header.lang), URID::new(header.datatype)) {
            (Some(urid), _) => Ok(LiteralInfo::Language(urid)),
            (None, Some(urid)) => Ok(LiteralInfo::Datatype(urid)),
            (None, None) => Err("Invalid Literal header: neither lang or datatype URIDs are set"),
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

/// A type-state for the Literal Writer, that writes the info header of a literal.
pub struct LiteralInfoWriter<'a> {
    writer: AtomWriter<'a>,
}

impl<'a> LiteralInfoWriter<'a> {
    /// Initializes the literal with the given info.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    pub fn write_info(mut self, info: LiteralInfo) -> Result<StringWriter<'a>, AtomWriteError> {
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
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Literal_Body = reader.next_value()?;

        let info =
            LiteralInfo::try_from_raw(header).map_err(|err| AtomReadError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
                error_message: err,
            })?;

        let data = reader.remaining_bytes();

        std::str::from_utf8(&data[0..data.len() - 1])
            .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
            .map_err(|_| AtomReadError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
                error_message: "Literal contents are invalid UTF-8",
            })
            .map(|string| (info, string))
    }

    #[inline]
    fn write(
        frame: AtomWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
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
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        let c_str = CStr::from_bytes_with_nul(body.as_bytes()).map_err(|_| {
            AtomReadError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
                error_message: "String value is not null-terminated",
            }
        })?;

        let str = c_str
            .to_str()
            .map_err(|_| AtomReadError::InvalidAtomValue {
                reading_type_uri: Self::uri(),
                error_message: "String contents are invalid UTF-8",
            })?;

        Ok(str)
    }

    fn write(
        frame: AtomWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Ok(StringWriter {
            writer: frame.terminated(0),
        })
    }
}

/// Handle to append strings to a string or literal.
pub struct StringWriter<'a> {
    writer: Terminated<AtomWriter<'a>>,
}

impl<'a> StringWriter<'a> {
    /// Appends a string to the atom's buffer.
    ///
    /// This method copies the given string to the end of the string atom, and then returns a
    /// mutable reference to the copy.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer for,
    /// the given additional string, or if any other write error occurs.
    pub fn append(&mut self, string: &str) -> Result<&mut str, AtomWriteError> {
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

        let mut raw_space = AlignedVec::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());

            let mut writer = space
                .write_atom(urids.atom.literal)
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

        let mut raw_space = AlignedVec::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());

            let mut writer = space.write_atom(urids.string).unwrap();
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
