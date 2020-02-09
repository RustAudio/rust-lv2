//! An atom containing memory of undefined type.
//!
//! This contents of this atom is considered as a simple blob of data. It used, for example, by the host to transmit the size of a writable atom port. Since it is so simple, it does not need a reading or writing parameter.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::chunk::ByteWriter;
//! use lv2_urid::prelude::*;
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCache) {
//!     let in_chunk: &[u8] = ports.input.read(urids.chunk, ()).unwrap();
//!     let mut out_chunk: ByteWriter = ports.output.init(urids.chunk, ()).unwrap();
//!
//!     out_chunk.write_raw(in_chunk).unwrap();
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Chunk](http://lv2plug.in/ns/ext/atom/atom.html#Chunk)
use crate::space::*;
use crate::Atom;
use core::UriBound;

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
    type WriteHandle = ByteWriter<'a, 'b>;

    fn read(space: Space<'a>, _: ()) -> Option<&'a [u8]> {
        space.data()
    }

    fn init(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<ByteWriter<'a, 'b>> {
        Some(ByteWriter::new(frame))
    }
}

/// A blob writer.
///
/// This struct is able to copy data into an atom. It basically wraps a `FramedMutSpace`.
pub struct ByteWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
}

impl<'a, 'b> ByteWriter<'a, 'b> {
    /// Create a new byte writer.
    pub fn new(frame: FramedMutSpace<'a, 'b>) -> Self {
        Self { frame }
    }

    /// Allocate memory in the atom, but don't write anything to it.
    pub fn allocate(&mut self, size: usize) -> Option<&'a mut [u8]> {
        self.frame.allocate(size, false).map(|(_, bytes)| bytes)
    }

    /// Copy data from the given slice to the atom.
    pub fn write_raw(&mut self, bytes: &[u8]) -> Option<&'a mut [u8]> {
        self.frame.write_raw(bytes, false)
    }

    /// Copy a struct instance to the atom.
    pub fn write<T>(&mut self, instance: &T) -> Option<&'a mut T>
    where
        T: Unpin + Copy + Send + Sync + Sized + 'static,
    {
        (&mut self.frame as &mut dyn MutSpace).write(instance, false)
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::chunk::*;
    use crate::*;
    use std::mem::size_of;
    use urid::mapper::*;
    use urid::prelude::*;

    #[test]
    fn test_chunk_and_slice_writer() {
        const SLICE_LENGTH: usize = 42;

        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.chunk)
                .unwrap();
            let mut writer = Chunk::init(frame, ()).unwrap();

            for (i, value) in writer
                .allocate(SLICE_LENGTH - 1)
                .unwrap()
                .into_iter()
                .enumerate()
            {
                *value = i as u8;
            }
            writer.write(&41u8).unwrap();
        }

        // verifying
        {
            let raw_space = raw_space.as_ref();
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
            let space = Space::from_reference(raw_space.as_ref());

            let data = Chunk::read(space.split_atom_body(urids.chunk).unwrap().0, ()).unwrap();
            assert_eq!(data.len(), SLICE_LENGTH);

            for (i, value) in data.iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }
    }
}
