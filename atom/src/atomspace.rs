use std::alloc::Layout;
use std::mem::size_of;

/// Specialized smart pointer to retrieve atoms.
#[derive(Clone, Copy)]
pub struct AtomSpace<'a> {
    data: Option<&'a [u8]>,
}

impl<'a> AtomSpace<'a> {
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
        Self { data: Some(data) }
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

#[cfg(test)]
mod tests {
    use crate::atomspace::*;

    #[test]
    fn test_atom_space() {
        let mut vector: Vec<u8> = vec![0; 256];
        for i in 0..128 {
            vector[i] = i as u8;
        }
        unsafe {
            let ptr = vector.as_mut_slice().as_mut_ptr().add(128) as *mut u32;
            *(ptr) = 0x42424242;
        }

        let mut space = AtomSpace::new(vector.as_slice());
        let lower_space = space.retrieve_raw(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let integer = unsafe { space.retrieve_type::<u32>() }.unwrap();
        assert_eq!(*integer, 0x42424242);
    }
}
