//! Smart pointers with safe atom reading and writing methods.

mod list;
mod space;
mod allocatable;
mod atom;

pub use space::Space;
pub use list::{SpaceList, SpaceHead};
pub use allocatable::*;
pub use atom::AtomSpace;

#[cfg(test)]
mod tests {
    use crate::space::*;
    use std::mem::{size_of, size_of_val};
    use urid::*;

    #[test]
    fn test_space() {
        let mut vector: Vec<u8> = vec![0; 256];
        for i in 0..128 {
            vector[i] = i as u8;
        }
        unsafe {
            let ptr = vector.as_mut_slice().as_mut_ptr().add(128) as *mut u32;
            *(ptr) = 0x42424242;
        }

        let space = Space::from_bytes(vector.as_slice());
        let (lower_space, space) = space.split_bytes_at(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let (integer, _) = unsafe { space.split_for_type::<u32>() }.unwrap();
        assert_eq!(*integer, 0x42424242);
    }

    #[test]
    fn test_split_atom() {
        let mut data: Box<[u64]> = Box::new([0; 256]);
        let urid: URID = unsafe { URID::new_unchecked(17) };

        // Writing an integer atom.
        unsafe {
            *(data.as_mut_ptr() as *mut sys::LV2_Atom_Int) = sys::LV2_Atom_Int {
                atom: sys::LV2_Atom {
                    size: size_of::<i32>() as u32,
                    type_: urid.get(),
                },
                body: 42,
            };

            let space = Space::from_ref(data.as_ref());
            let (atom, _) = space.split_atom().unwrap();
            let (body, _) = atom.split_atom_body(urid).unwrap();
            let body = body.as_bytes();

            assert_eq!(size_of::<i32>(), size_of_val(body));
            assert_eq!(42, unsafe { *(body.as_ptr() as *const i32) });
        }
    }

    #[test]
    fn test_from_reference() {
        let value: u64 = 0x42424242;
        let space = Space::from_ref(&value);
        assert_eq!(value, *unsafe { space.split_for_type::<u64>() }.unwrap().0);
    }

    fn test_mut_space<'a, S: AllocateSpace<'a>>(mut space: S) {
        let map = HashURIDMapper::new();
        let urids = crate::AtomURIDCollection::from_map(&map).unwrap();

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        match space.write_raw(test_data.as_slice(), true) {
            Some(written_data) => assert_eq!(test_data.as_slice(), written_data),
            None => panic!("Writing failed!"),
        }

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = crate::space::write_value(&mut space, test_atom).unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);

        let created_space = unsafe { Space::from_atom_mut(written_atom) }
            .space
            .take()
            .unwrap();
        assert_eq!(
            created_space.as_ptr() as usize,
            written_atom as *mut _ as usize
        );
        assert_eq!(created_space.len(), size_of::<sys::LV2_Atom>() + 42);

        let mut atom_frame =
            AtomSpace::write_new(&mut space as &mut dyn AllocateSpace, urids.chunk).unwrap();

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        let written_data = atom_frame.write_raw(test_data.as_slice(), true).unwrap();
        assert_eq!(test_data.as_slice(), written_data);
        assert_eq!(atom_frame.atom().size, test_data.len() as u32);

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = crate::space::write_value(&mut atom_frame, test_atom).unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
        assert_eq!(
            atom_frame.atom().size as usize,
            test_data.len() + size_of_val(&test_atom)
        );
    }

    #[test]
    fn test_root_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let frame = unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        };

        test_mut_space(frame);
    }

    #[test]
    fn test_space_head() {
        let mut space = SpaceList::default();
        let head = SpaceHead::new(&mut space);
        test_mut_space(head);
    }

    #[test]
    fn test_padding_inside_frame() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let raw_space: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        };

        // writing
        {
            let mut root = raw_space;
            let mut frame =
                AtomSpace::write_new(&mut root, URID::<()>::new(1).unwrap())
                    .unwrap();
            crate::space::write_value(&mut frame, 42u32).unwrap();
            crate::space::write_value(&mut frame, 17u32).unwrap();
        }

        // checking
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, 1);
            assert_eq!(atom.size as usize, 12);

            let (value, space) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 42);
            let (_, space) = space.split_at(4);

            let (value, _) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 17);
        }
    }

    #[test]
    fn unaligned_root_write() {
        let mut raw_space = Box::new([0u8; 8]);

        {
            let mut root_space = &mut raw_space[3..];
            crate::space::write_value(&mut root_space, 42u8).unwrap();
        }

        assert_eq!(&[0, 0, 0, 42, 0, 0, 0, 0], raw_space.as_ref());
    }
}
