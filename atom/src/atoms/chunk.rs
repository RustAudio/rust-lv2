//! An atom containing memory of undefined type.
//!
//! This contents of this atom is considered as a simple blob of data. It used, for example, by the host to transmit the size of a writable atom port. Since it is so simple, it does not need a reading or writing parameter.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//!
//! use lv2_atom::space::{AtomSpace, AtomSpaceWriter, SpaceWriter};
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
//!     let in_chunk: &AtomSpace = ports.input.read(urids.chunk).unwrap();
//!     let mut out_chunk: AtomSpaceWriter = ports.output.write(urids.chunk).unwrap();
//!
//!     let bytes = in_chunk.as_bytes();
//!     out_chunk.write_bytes(bytes).unwrap();
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Chunk](http://lv2plug.in/ns/ext/atom/atom.html#Chunk)
use crate::space::error::{AtomReadError, AtomWriteError};
use crate::space::*;
use crate::AtomSpaceWriter;
use crate::{Atom, AtomHandle};
use urid::UriBound;

/// An atom containing an arbitrary byte buffer.
///
/// [See also the module documentation.](index.html)
pub struct Chunk;

unsafe impl UriBound for Chunk {
    const URI: &'static [u8] = sys::LV2_ATOM__Chunk;
}

pub struct ChunkReaderHandle;
impl<'a> AtomHandle<'a> for ChunkReaderHandle {
    type Handle = &'a AtomSpace;
}

pub struct ChunkWriterHandle;
impl<'a> AtomHandle<'a> for ChunkWriterHandle {
    type Handle = AtomSpaceWriter<'a>;
}

impl Atom for Chunk {
    type ReadHandle = ChunkReaderHandle;
    type WriteHandle = ChunkWriterHandle;

    #[inline]
    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        Ok(body)
    }

    #[inline]
    fn write(
        frame: AtomSpaceWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use crate::atoms::chunk::*;
    use crate::*;

    #[test]
    fn test_chunk_and_slice_writer() {
        const SLICE_LENGTH: usize = 42;

        let map = HashURIDMapper::new();
        let urids = crate::atoms::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = space.init_atom(urids.chunk).unwrap();
            let data = writer.allocate(SLICE_LENGTH).unwrap();

            for (i, value) in data.into_iter().enumerate() {
                *value = i as u8;
            }

            space.write_value(41u8).unwrap();
        }

        // verifying
        {
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();
            assert_eq!(atom.header().size_of_body(), SLICE_LENGTH);
            assert_eq!(atom.header().urid(), urids.chunk.get());

            let data = atom.body().as_bytes();
            for (i, value) in data.iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }

        // reading
        {
            let data =
                unsafe { Chunk::read(raw_space.read().next_atom().unwrap().body()) }.unwrap();
            assert_eq!(data.bytes_len(), SLICE_LENGTH);

            for (i, value) in data.as_bytes().iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }
    }
}
