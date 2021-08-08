//! An atom containing memory of undefined type.
//!
//! This contents of this atom is considered as a simple blob of data. It used, for example, by the host to transmit the size of a writable atom port. Since it is so simple, it does not need a reading or writing parameter.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
//!     let in_chunk: &[u8] = ports.input.read(urids.chunk, ()).unwrap();
//!     let mut out_chunk: AtomSpaceWriter = ports.output.init(urids.chunk, ()).unwrap();
//!
//!     lv2_atom::space::write_bytes(&mut out_chunk, in_chunk).unwrap();
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Chunk](http://lv2plug.in/ns/ext/atom/atom.html#Chunk)
use crate::space::*;
use crate::Atom;
use urid::UriBound;

/// An atom containing memory of undefined type.
///
/// [See also the module documentation.](index.html)
pub struct Chunk;

unsafe impl UriBound for Chunk {
    const URI: &'static [u8] = sys::LV2_ATOM__Chunk;
}

impl<'a, 'b> Atom<'a, 'b> for Chunk
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = &'a [u8];
    type WriteParameter = ();
    type WriteHandle = AtomSpaceWriter<'b>;

    unsafe fn read(space: &'a Space, _: ()) -> Option<&'a [u8]> {
        Some(space.as_bytes())
    }

    fn init(frame: AtomSpaceWriter<'b>, _: ()) -> Option<AtomSpaceWriter<'b>> {
        Some(frame)
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::*;
    use crate::*;
    use std::mem::size_of;
    use urid::*;

    #[test]
    fn test_chunk_and_slice_writer() {
        const SLICE_LENGTH: usize = 42;

        let map = HashURIDMapper::new();
        let urids = crate::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = AtomSpace::boxed(256);

        // writing
        {
            let mut space = raw_space.as_bytes_mut();
            let mut writer = space::init_atom(&mut space, urids.chunk, ()).unwrap();
            let data = writer.allocate_unaligned(SLICE_LENGTH - 1).unwrap();

            for (i, value) in data.into_iter().enumerate() {
                *value = i as u8;
            }

            space::write_value(&mut space, 41u8).unwrap();
        }

        // verifying
        {
            let (atom, data) = raw_space.split_at(size_of::<sys::LV2_Atom>()).unwrap();

            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.size as usize, SLICE_LENGTH);
            assert_eq!(atom.type_, urids.chunk.get());

            let data = data.split_at(SLICE_LENGTH).unwrap().0.as_bytes();
            for i in 0..SLICE_LENGTH {
                assert_eq!(data[i] as usize, i);
            }
        }

        // reading
        {
            let data = unsafe { Chunk::read(raw_space.split_atom_body(urids.chunk).unwrap().0, ()) }.unwrap();
            assert_eq!(data.len(), SLICE_LENGTH);

            for (i, value) in data.iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }
    }
}
