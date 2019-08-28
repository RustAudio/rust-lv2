use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::size_of;
use urid::URIDBound;

/// Specialized smart pointer to retrieve atoms.
pub struct Space<'a, A: URIDBound + ?Sized> {
    data: Option<&'a [u8]>,
    atom_type: PhantomData<A>,
}

impl<'a, A: URIDBound + ?Sized> Space<'a, A> {
    /// Create a new atom space from a pointer.
    ///
    /// This method is used by ports to determine the available space for atoms: First, it reads the size of the atom and then creates an atom space of the read size.
    ///
    /// Since it is assumed that the pointer as well as the space behind the atom is valid, this method is unsafe.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn from_atom(atom: &sys::LV2_Atom, urids: &A::CacheType) -> Option<Self> {
        if atom.type_ == A::urid(urids) {
            let size = atom.size as usize;
            let data = std::slice::from_raw_parts(
                (atom as *const sys::LV2_Atom).add(1) as *const u8,
                size,
            );
            Some(Self::from_slice(data))
        } else {
            None
        }
    }

    pub fn from_slice(data: &'a [u8]) -> Self {
        if data.as_ptr() as usize % 8 != 0 {
            panic!("Trying to create an unaligned atom space");
        }
        Space {
            data: Some(data),
            atom_type: PhantomData,
        }
    }

    pub unsafe fn from_space<A2: URIDBound>(other: Space<'a, A2>) -> Self {
        Self {
            data: other.data,
            atom_type: PhantomData,
        }
    }

    /// Try to retrieve a slice of bytes.
    ///
    /// This method basically splits off the lower part of the internal bytes slice and creates a new atom space pointer of the upper part, if it's big enough. Since atom space has to be 64-bit-aligned, there might be a padding space that's neither in the lower nor in the upper slice.
    pub fn split_raw(self, size: usize) -> Option<(&'a [u8], Self)> {
        let data = self.data?;

        if size > data.len() {
            return None;
        }
        let (lower_space, upper_space) = data.split_at(size);

        let padding = if size % 8 == 0 { 0 } else { 8 - size % 8 };
        let upper_space = if padding <= upper_space.len() {
            let upper_space = upper_space.split_at(padding).1;
            Some(upper_space)
        } else {
            None
        };
        let upper_space = Self {
            data: upper_space,
            atom_type: PhantomData,
        };

        Some((lower_space, upper_space))
    }

    /// Try to retrieve atom space.
    ///
    /// This method calls [`retrieve_raw`](#method.retrieve_raw) and wraps the returned slice in an atom space.
    pub fn split_space(self, size: usize) -> Option<(Self, Self)> {
        self.split_raw(size)
            .map(|(data, rhs)| (Self::from_slice(data), rhs))
    }

    /// Try to retrieve a reference to a sized type.
    ///
    /// This method retrieves a slice of memory using the [`retrieve_raw`](#method.retrieve_raw) method and interprets it as an instance of `T`. Since there is no way to check that the memory is actually a valid instance of `T`, this method is unsafe.
    pub unsafe fn split_type<T: Sized>(self) -> Option<(&'a T, Self)> {
        self.split_raw(size_of::<T>())
            .map(|(data, rhs)| (&*(data.as_ptr() as *const T), rhs))
    }

    pub fn data(&self) -> Option<&'a [u8]> {
        self.data
    }

    pub fn mut_data(&mut self) -> &mut Option<&'a [u8]> {
        &mut self.data
    }
}

impl<'a, A: URIDBound + ?Sized> Clone for Space<'a, A> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            atom_type: PhantomData,
        }
    }
}

impl<'a, A: URIDBound + ?Sized> Copy for Space<'a, A> {}

pub trait MutSpace<'a> {
    fn allocate(&mut self, size: usize) -> Option<&'a mut [u8]>;

    fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut [u8]> {
        self.allocate(data.len()).map(|space| {
            space.copy_from_slice(data);
            space
        })
    }
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
    fn allocate(&mut self, size: usize) -> Option<&'a mut [u8]> {
        if self.space.get_mut().is_none() {
            return None;
        }
        let space = self.space.replace(None).unwrap();

        if size > space.len() {
            return None;
        }
        let (lower_slice, upper_slice) = space.split_at_mut(size);

        let padding = if size % 8 == 0 { 0 } else { 8 - size % 8 };

        if padding < upper_slice.len() {
            let (_, upper_slice) = upper_slice.split_at_mut(padding);
            self.space.set(Some(upper_slice))
        }
        Some(lower_slice)
    }
}

pub struct FramedMutSpace<'a, 'b, A: URIDBound + ?Sized> {
    atom: &'a mut sys::LV2_Atom,
    parent: &'b mut dyn MutSpace<'a>,
    atom_type: PhantomData<A>,
}

impl<'a, 'b, A: URIDBound + ?Sized> MutSpace<'a> for FramedMutSpace<'a, 'b, A> {
    fn allocate(&mut self, size: usize) -> Option<&'a mut [u8]> {
        self.parent.allocate(size).map(|data| {
            self.atom.size += size as u32;
            data
        })
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

    pub fn create_atom_frame<'c, A: URIDBound + ?Sized>(
        &'c mut self,
        urids: &A::CacheType,
    ) -> Option<FramedMutSpace<'a, 'c, A>> {
        let atom = sys::LV2_Atom {
            size: 0,
            type_: A::urid(urids).get(),
        };
        let atom: &'a mut sys::LV2_Atom = self.write(&atom)?;
        Some(FramedMutSpace {
            atom,
            parent: self,
            atom_type: PhantomData,
        })
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

        let space: Space<crate::Int> = Space::from_slice(vector.as_slice());
        let (lower_space, space) = space.split_raw(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let (integer, _) = unsafe { space.split_type::<u32>() }.unwrap();
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
        let written_atom = (&mut frame as &mut dyn MutSpace).write(&test_atom).unwrap();
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

        let mut atom_frame: FramedMutSpace<crate::Int> = (&mut root as &mut dyn MutSpace)
            .create_atom_frame(&urids)
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
