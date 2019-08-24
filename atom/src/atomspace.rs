use crate::{AtomBody, AtomURIDCache};
use std::alloc::Layout;
use std::mem::size_of;

/// Specialized smart pointer to retrieve atoms.
#[derive(Clone, Copy)]
pub struct AtomSpace<'a> {
    data: &'a [u8],
}

impl<'a> AtomSpace<'a> {
    /// Create a new atom space from raw data.
    ///
    /// The data has to be 64-bit-aligned, which means that `data.as_ptr() as usize % 8` must always be 0. The soundness of this module depends on this invariant and the method will panic if it's not upheld.
    pub fn new(data: &'a [u8]) -> Self {
        if data.as_ptr() as usize % 8 != 0 {
            panic!("Trying to create an unaligned atom space");
        }
        Self { data }
    }

    /// Try to retrieve a slice of bytes.
    ///
    /// This method basically splits off the lower part of the internal bytes slice and creates a new atom space pointer of the upper part, if it's big enough. Since atom space has to be 64-bit-aligned, there might be a padding space that's neither in the lower nor in the upper slice.
    pub fn retrieve_raw(self, size: usize) -> Option<(&'a [u8], Option<Self>)> {
        if size > self.data.len() {
            return None;
        }
        let (lower_space, upper_space) = self.data.split_at(size);

        let padding = if size % 8 == 0 { 0 } else { 8 - size % 8 };
        let upper_space = if padding <= upper_space.len() {
            let upper_space = Self::new(upper_space.split_at(padding).1);
            Some(upper_space)
        } else {
            None
        };
        Some((lower_space, upper_space))
    }

    /// Try to retrieve atom space.
    ///
    /// This method calls [`retrieve_raw`](#method.retrieve_raw) and wraps the returned slice in an atom space.
    pub fn retrieve_space(self, size: usize) -> Option<(Self, Option<Self>)> {
        let (lower_space, upper_space) = self.retrieve_raw(size)?;
        let lower_space = Self::new(lower_space);
        Some((lower_space, upper_space))
    }

    /// Try to retrieve a reference to a sized type.
    ///
    /// This method retrieves a slice of memory using the [`retrieve_raw`](#method.retrieve_raw) method and interprets it as an instance of `T`. Since there is no way to check that the memory is actually a valid instance of `T`, this method is unsafe.
    pub unsafe fn retrieve_type<T: Sized>(self) -> Option<(&'a T, Option<Self>)> {
        assert_eq!(8 % Layout::new::<T>().align(), 0);

        let size = size_of::<T>();
        let (lower_space, upper_space) = self.retrieve_raw(size)?;
        assert_eq!(lower_space.len(), size);

        let instance = &*(lower_space.as_ptr() as *const T);
        Some((instance, upper_space))
    }

    /// Try to retrieve the body of an atom.
    ///
    /// This method retrieves an atom header first and checks if the URID is valid. Then, it retrieves the space, as noted in the header, and creates a body reference from it.
    pub fn retrieve_atom<T: AtomBody>(self, urids: &AtomURIDCache) -> Option<(T, Option<Self>)> {
        let (header, upper_space) = unsafe { self.retrieve_type::<sys::LV2_Atom>() }?;
        let upper_space = upper_space?;
        if header.type_ != T::urid(urids) {
            return None;
        }

        let (raw_data, upper_space) = upper_space.retrieve_space(header.size as usize)?;

        let atom_body = T::create_ref(raw_data)?;

        Some((atom_body, upper_space))
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

        let space = AtomSpace::new(vector.as_slice());
        let (lower_space, upper_space) = space.retrieve_raw(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let upper_space = upper_space.unwrap();
        let (integer, _) = unsafe { upper_space.retrieve_type::<u32>() }.unwrap();
        assert_eq!(*integer, 0x42424242);
    }
}
