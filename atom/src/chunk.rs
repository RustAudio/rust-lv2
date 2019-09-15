use crate::space::*;
use crate::{Atom, AtomURIDCache};
use core::UriBound;
use urid::{URIDBound, URID};

/// An atom containing a chunk of memory with undefined contents.
///
/// This atom is specified [here](http://lv2plug.in/ns/ext/atom/atom.html#Chunk).
pub struct Chunk;

unsafe impl UriBound for Chunk {
    const URI: &'static [u8] = sys::LV2_ATOM__Chunk;
}

impl URIDBound for Chunk {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.chunk
    }
}

impl<'a, 'b> Atom<'a, 'b> for Chunk
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = &'a [u8];
    type WriteParameter = ();
    type WriteHandle = FramedMutSpace<'a, 'b>;

    fn read(space: Space<'a>, _: (), urids: &AtomURIDCache) -> Option<(&'a [u8], Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.chunk)?;
        Some((body.data()?, space))
    }

    fn write(
        space: &'b mut dyn MutSpace<'a>,
        _: (),
        urids: &AtomURIDCache,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        space.create_atom_frame(urids.chunk)
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::chunk::*;
    use crate::*;
    use std::mem::{size_of, size_of_val};
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    #[test]
    fn test_chunk_and_slice_writer() {
        const SLICE_LENGTH: usize = 42;

        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = Chunk::write(&mut space, (), &urids).unwrap();

            for (i, value) in (&mut writer as &mut dyn MutSpace)
                .allocate(SLICE_LENGTH - 1, false)
                .unwrap()
                .1
                .into_iter()
                .enumerate()
            {
                *value = i as u8;
            }
            (&mut writer as &mut dyn MutSpace)
                .write(&41u8, false)
                .unwrap();
        }

        // verifying
        {
            let raw_space = unsafe {
                std::slice::from_raw_parts(
                    raw_space.as_ptr() as *const u8,
                    size_of_val(raw_space.as_ref()),
                )
            };
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

            let data = Chunk::read(space, (), &urids).unwrap().0;
            assert_eq!(data.len(), SLICE_LENGTH);

            for (i, value) in data.iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }
    }
}
