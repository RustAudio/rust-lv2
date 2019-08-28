use std::alloc::Layout;
use std::mem::size_of;
use std::cell::Cell;
use urid::URID;

/// Specialized smart pointer to retrieve atoms.
#[derive(Clone, Copy)]
pub struct Space<'a> {
    data: Option<&'a [u8]>,
}

impl<'a> Space<'a> {
    /// Create a new atom space from a pointer.
    ///
    /// This method is used by ports to determine the available space for atoms: First, it reads the size of the atom and then creates an atom space of the read size.
    ///
    /// Since it is assumed that the pointer as well as the space behind the atom is valid, this method is unsafe.
    pub unsafe fn from_atom_ptr(atom: *const sys::LV2_Atom) -> Self {
        let size = (*atom).size as usize + size_of::<sys::LV2_Atom>();
        let data = std::slice::from_raw_parts(atom as *const u8, size);
        Self::new(data)
    }

    /// Create a new atom space from raw data.
    ///
    /// The data has to be 64-bit-aligned, which means that `data.as_ptr() as usize % 8` must always be 0. The soundness of this module depends on this invariant and the method will panic if it's not upheld.
    pub fn new(data: &'a [u8]) -> Self {
        if data.as_ptr() as usize % 8 != 0 {
            panic!("Trying to create an unaligned atom space");
        }
        Space { data: Some(data) }
    }

    /// Try to retrieve a slice of bytes.
    ///
    /// This method basically splits off the lower part of the internal bytes slice and creates a new atom space pointer of the upper part, if it's big enough. Since atom space has to be 64-bit-aligned, there might be a padding space that's neither in the lower nor in the upper slice.
    pub fn retrieve_raw(&mut self, size: usize) -> Option<&'a [u8]> {
        let data = self.data?;

        if size > data.len() {
            return None;
        }
        let (lower_space, upper_space) = data.split_at(size);

        let padding = if size % 8 == 0 { 0 } else { 8 - size % 8 };
        self.data = if padding <= upper_space.len() {
            let upper_space = upper_space.split_at(padding).1;
            Some(upper_space)
        } else {
            None
        };

        Some(lower_space)
    }

    /// Try to retrieve atom space.
    ///
    /// This method calls [`retrieve_raw`](#method.retrieve_raw) and wraps the returned slice in an atom space.
    pub fn retrieve_space(&mut self, size: usize) -> Option<Self> {
        let space = self.retrieve_raw(size)?;
        assert_eq!(space.len(), size);

        let space = Self::new(space);
        Some(space)
    }

    /// Try to retrieve a reference to a sized type.
    ///
    /// This method retrieves a slice of memory using the [`retrieve_raw`](#method.retrieve_raw) method and interprets it as an instance of `T`. Since there is no way to check that the memory is actually a valid instance of `T`, this method is unsafe.
    pub unsafe fn retrieve_type<T: Sized>(&mut self) -> Option<&'a T> {
        assert_eq!(8 % Layout::new::<T>().align(), 0);

        let size = size_of::<T>();
        let space = self.retrieve_raw(size)?;
        assert_eq!(space.len(), size);

        let instance = &*(space.as_ptr() as *const T);
        Some(instance)
    }
}

pub trait MutSpace<'a> {
    fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut [u8]>;
}

pub struct RootMutSpace<'a> {
    space: Cell<Option<&'a mut [u8]>>,
}

impl<'a> RootMutSpace<'a> {
    pub fn new(space: &'a mut [u8]) -> Self {
        if space.as_ptr() as usize % 8 != 0 {
            panic!("Trying to create an unaligned atom space");
        }
        RootMutSpace {
            space: Cell::new(Some(space)),
        }
    }
}

impl<'a> MutSpace<'a> for RootMutSpace<'a> {
    fn write_raw(&mut self, raw_data: &[u8]) -> Option<&'a mut [u8]> {
        if self.space.get_mut().is_none() {
            return None;
        }
        let space = self.space.replace(None).unwrap();

        if raw_data.len() > space.len() {
            return None;
        }
        let (lower_slice, upper_slice) = space.split_at_mut(raw_data.len());
        lower_slice.copy_from_slice(raw_data);

        let padding = if raw_data.len() % 8 == 0 {
            0
        } else {
            8 - raw_data.len() % 8
        };

        if padding < upper_slice.len() {
            let (_, upper_slice) = upper_slice.split_at_mut(padding);
            self.space.set(Some(upper_slice))
        }
        Some(lower_slice)
    }
}

pub struct FramedMutSpace<'a, 'b> {
    atom: &'a mut sys::LV2_Atom,
    parent: &'b mut dyn MutSpace<'a>,
}

impl<'a, 'b> MutSpace<'a> for FramedMutSpace<'a, 'b> {
    fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut [u8]> {
        let data = self.parent.write_raw(data)?;
        self.atom.size += data.len() as u32;
        Some(data)
    }
}

impl<'a, 'b> dyn MutSpace<'a> + 'b {
    pub fn write<T: Sized>(&mut self, instance: &T) -> Option<&'a mut T> {
        let size = std::mem::size_of::<T>();
        let input_data =
            unsafe { std::slice::from_raw_parts(instance as *const T as *const u8, size) };

        let output_data = self.write_raw(input_data)?;

        assert_eq!(size, output_data.len());
        unsafe { Some(&mut *(output_data.as_mut_ptr() as *mut T)) }
    }

    pub fn create_atom_frame<'c>(&'c mut self, urid: URID) -> Option<FramedMutSpace<'a, 'c>> {
        let atom = sys::LV2_Atom {
            size: 0,
            type_: urid.get(),
        };
        let atom: &'a mut sys::LV2_Atom = self.write(&atom)?;
        Some(FramedMutSpace { atom, parent: self })
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use crate::AtomURIDCache;
    use std::mem::{size_of, size_of_val};
    use urid::mapper::URIDMap;
    use urid::URIDCache;

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

        let mut space = Space::new(vector.as_slice());
        let lower_space = space.retrieve_raw(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let integer = unsafe { space.retrieve_type::<u32>() }.unwrap();
        assert_eq!(*integer, 0x42424242);
    }
    

    #[test]
    fn test_root_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let mut frame: RootMutSpace = RootMutSpace::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        match frame.write_raw(test_data.as_slice()) {
            Some(written_data) => assert_eq!(test_data.as_slice(), written_data),
            None => panic!("Writing failed!"),
        }

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = (&mut frame as &mut dyn MutSpace)
            .write(&test_atom)
            .unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
    }

    #[test]
    fn test_framed_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let mut root: RootMutSpace = RootMutSpace::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        let mut urid_map = URIDMap::new().make_map_interface();
        let urids = AtomURIDCache::from_map(&urid_map.map()).unwrap();

        let mut atom_frame: FramedMutSpace = (&mut root as &mut dyn MutSpace)
            .create_atom_frame(urids.int.into_general())
            .unwrap();

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        let written_data = atom_frame.write_raw(test_data.as_slice()).unwrap();
        assert_eq!(test_data.as_slice(), written_data);
        assert_eq!(atom_frame.atom.size, test_data.len() as u32);

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let borrowed_frame = &mut atom_frame as &mut dyn MutSpace;
        let written_atom = borrowed_frame.write(&test_atom).unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
        assert_eq!(
            atom_frame.atom.size as usize,
            test_data.len() + size_of_val(&test_atom)
        );
    }
}
