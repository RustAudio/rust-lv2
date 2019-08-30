use crate::space::*;
use crate::AtomURIDCache;
use crate::ScalarAtom;
use core::UriBound;
use std::convert::TryFrom;
use std::mem::size_of;
use urid::{URIDBound, URID};

pub struct Vector;

unsafe impl UriBound for Vector {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

impl URIDBound for Vector {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.vector
    }
}

impl Vector {
    pub fn get_children<'a, C: ScalarAtom>(
        space: Space<'a, Self>,
        urids: &C::CacheType,
    ) -> Option<&'a [C::InternalType]> {
        let (body, space) = unsafe { space.split_type::<sys::LV2_Atom_Vector_Body>() }?;

        if body.child_type != C::urid(urids)
            || body.child_size as usize != size_of::<C::InternalType>()
        {
            return None;
        }

        let data = space.data()?;

        assert_eq!(data.len() % size_of::<C::InternalType>(), 0);
        let children_count = data.len() / size_of::<C::InternalType>();

        let children = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const C::InternalType, children_count)
        };
        Some(children)
    }

    pub fn new_vector<'a, 'b, C: ScalarAtom>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        child_urids: &C::CacheType,
    ) -> Option<SliceWriter<'a, 'b, C::InternalType, Self>> {
        let mut frame: FramedMutSpace<Self> = unsafe { space.create_atom_frame(urids)? };

        let body = sys::LV2_Atom_Vector_Body {
            child_type: C::urid(child_urids).get(),
            child_size: size_of::<C::InternalType>() as u32,
        };
        unsafe { (&mut frame as &mut dyn MutSpace).write(&body, true)? };

        let children = unsafe { frame.allocate(0, false) }?;
        Some(SliceWriter {
            data: unsafe {
                std::slice::from_raw_parts_mut(
                    children.as_mut_ptr() as *mut C::InternalType,
                    children.len(),
                )
            },
            frame,
        })
    }
}

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

impl Chunk {
    pub fn get(space: Space<Self>) -> Option<&[u8]> {
        space.data()
    }

    pub fn new_chunk<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
    ) -> Option<SliceWriter<'a, 'b, u8, Self>> {
        let mut frame = unsafe { space.create_atom_frame::<Self>(urids) }?;
        Some(SliceWriter {
            data: unsafe { frame.allocate(0, false) }?,
            frame,
        })
    }
}

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
    pub fn get_str(space: Space<Self>) -> Option<(URID, &str)> {
        let (header, space) = unsafe { space.split_type::<sys::LV2_Atom_Literal_Body>()? };
        if let Ok(LiteralType::Language(urid)) = LiteralType::try_from(header) {
            let data = space.data()?;
            std::str::from_utf8(&data[0..data.len() - 1])
                .or_else(|error| std::str::from_utf8(&data[0..error.valid_up_to()]))
                .ok()
                .map(|string| (urid, string))
        } else {
            None
        }
    }

    pub fn get(space: Space<Self>) -> Option<(LiteralType, &[u8])> {
        let (header, space) = unsafe { space.split_type::<sys::LV2_Atom_Literal_Body>()? };
        let literal_type = LiteralType::try_from(header).ok()?;
        let space = space.data()?;
        Some((literal_type, space))
    }

    unsafe fn write_body(frame: &mut dyn MutSpace, literal_type: LiteralType) -> Option<()> {
        frame
            .write(&sys::LV2_Atom_Literal_Body::from(literal_type), true)
            .map(|_| ())
    }

    pub fn new_str_literal<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        lang: URID,
    ) -> Option<LiteralWriter<'a, 'b>> {
        let mut frame: FramedMutSpace<Self> = unsafe { space.create_atom_frame(urids)? };
        unsafe { Self::write_body(&mut frame, LiteralType::Language(lang)) };
        Some(LiteralWriter {
            writer: SliceWriter {
                data: unsafe { frame.allocate(0, false)? },
                frame,
            },
        })
    }

    pub fn new_data_literal<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        datatype: URID,
    ) -> Option<SliceWriter<'a, 'b, u8, Self>> {
        let mut frame: FramedMutSpace<Self> = unsafe { space.create_atom_frame(urids)? };
        unsafe { Self::write_body(&mut frame, LiteralType::Datatype(datatype)) };
        Some(SliceWriter {
            data: unsafe { frame.allocate(0, false)? },
            frame,
        })
    }
}

pub struct LiteralWriter<'a, 'b> {
    writer: SliceWriter<'a, 'b, u8, Literal>,
}

impl<'a, 'b> LiteralWriter<'a, 'b> {
    pub fn append(&mut self, string: &str) -> Option<&mut str> {
        let data = string.as_bytes();
        let space = self.writer.allocate(data.len())?;
        space.copy_from_slice(data);
        unsafe { Some(std::str::from_utf8_unchecked_mut(space)) }
    }
}

impl<'a, 'b> Drop for LiteralWriter<'a, 'b> {
    fn drop(&mut self) {
        self.writer.push(0);
    }
}

pub struct SliceWriter<'a, 'b, T: Sized, A: URIDBound + ?Sized> {
    data: &'a mut [T],
    frame: FramedMutSpace<'a, 'b, A>,
}

impl<'a, 'b, T: Sized, A: URIDBound + ?Sized> SliceWriter<'a, 'b, T, A> {
    pub fn push(&mut self, child: T) -> Option<&mut T> {
        self.allocate(1).map(|slice| {
            slice[0] = child;
            &mut slice[0]
        })
    }

    pub fn allocate(&mut self, count: usize) -> Option<&mut [T]> {
        let new_children = unsafe { self.frame.allocate(size_of::<T>() * count, false)? };
        let new_children =
            unsafe { std::slice::from_raw_parts_mut(new_children.as_mut_ptr() as *mut T, count) };
        self.data = unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut T,
                self.data.len() + new_children.len(),
            )
        };
        Some(new_children)
    }

    pub fn get(self) -> &'a mut [T] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use crate::*;
    use core::UriBound;
    use std::ffi::CStr;
    use std::mem::size_of;
    use urid::URIDCache;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());

            let mut writer = Vector::new_vector::<Int>(&mut space, &urids, &urids).unwrap();
            {
                let children = writer.allocate(CHILD_COUNT).unwrap();
                for i in 0..children.len() {
                    children[i] = i as i32;
                }
            }
            // Checking that the written slice and the general slice are the same.
            for (i, child) in writer.get().into_iter().enumerate() {
                assert_eq!(i, *child as usize);
            }
        }

        // verifying
        {
            let (vector, children) = raw_space.split_at(size_of::<sys::LV2_Atom_Vector>());

            let vector = unsafe { &*(vector.as_ptr() as *const sys::LV2_Atom_Vector) };
            assert_eq!(vector.atom.type_, urids.vector.get());
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * CHILD_COUNT
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int.get());

            let children =
                unsafe { std::slice::from_raw_parts(children.as_ptr() as *const i32, CHILD_COUNT) };
            for i in 0..CHILD_COUNT {
                assert_eq!(children[i], i as i32);
            }
        }

        // reading
        {
            let space: Space<Vector> =
                unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom), &urids) }
                    .unwrap();
            let children: &[i32] = Vector::get_children::<Int>(space, &urids).unwrap();

            assert_eq!(children.len(), CHILD_COUNT);
            for i in 0..children.len() {
                assert_eq!(children[i], i as i32);
            }
        }
    }

    #[test]
    fn test_chunk_and_slice_writer() {
        const SLICE_LENGTH: usize = 42;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = Chunk::new_chunk(&mut space, &urids).unwrap();

            for (i, value) in writer
                .allocate(SLICE_LENGTH - 1)
                .unwrap()
                .into_iter()
                .enumerate()
            {
                *value = i as u8;
            }
            writer.push(41).unwrap();
        }

        // verifying
        {
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
            let space: Space<Chunk> =
                unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom), &urids) }
                    .unwrap();

            let data = Chunk::get(space).unwrap();
            assert_eq!(data.len(), SLICE_LENGTH);

            for (i, value) in data.iter().enumerate() {
                assert_eq!(*value as usize, i);
            }
        }
    }

    #[test]
    fn test_literal() {
        struct German;
        unsafe impl UriBound for German {
            const URI: &'static [u8] = b"http://lexvo.org/id/iso639-1/de\0";
        }

        #[derive(URIDCache)]
        pub struct TestURIDs {
            atom: AtomURIDCache,
            german: URID<German>,
        }

        const SAMPLE: &str = "Da steh ich nun, ich armer Tor! Und bin so klug als wie zuvor;";

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = TestURIDs::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer =
                Literal::new_str_literal(&mut space, &urids.atom, urids.german.into_general())
                    .unwrap();
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
            let space: Space<Literal> = unsafe {
                Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom), &urids.atom)
                    .unwrap()
            };
            let (lang, text) = Literal::get_str(space).unwrap();

            assert_eq!(lang, urids.german);
            assert_eq!(text, SAMPLE);
        }
    }
}
