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
    pub fn retrieve_atom<T: AtomBody>(
        self,
        urids: &AtomURIDCache,
    ) -> Option<(&'a T, Option<Self>)> {
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

/// Specialized smart pointer to write atoms.
pub struct MutAtomSpace<'a> {
    data: &'a mut [u8],
}

impl<'a> MutAtomSpace<'a> {
    /// Create a new mutable atom space.
    pub fn new(data: &'a mut [u8]) -> Self {
        if data.as_ptr() as usize % 8 != 0 {
            panic!("Trying to create an unaligned atom space");
        }
        Self { data }
    }

    /// Try to write raw data to the atom space.
    ///
    /// First, the space is split into a lower slice which will contain the written data and a upper slice which can be used later. Then, the raw data will be written into the lower slice and the upper slice will be padded and wrapped in a mutable atom space. If there is no space left after the data has been written, the method will return `None` instead of some mutable atom space.
    ///
    /// If there is not enough space for the raw data, `Err(self)` will be returned.
    pub fn write_raw(self, raw_data: &[u8]) -> Result<(&'a mut [u8], Option<Self>), Self> {
        if raw_data.len() > self.data.len() {
            return Err(self);
        }
        let (lower_slice, upper_slice) = self.data.split_at_mut(raw_data.len());
        lower_slice.copy_from_slice(raw_data);

        let padding = if raw_data.len() % 8 == 0 {
            0
        } else {
            8 - raw_data.len() % 8
        };

        if padding < upper_slice.len() {
            let (_, upper_slice) = upper_slice.split_at_mut(padding);
            Ok((lower_slice, Some(Self::new(upper_slice))))
        } else {
            Ok((lower_slice, None))
        }
    }

    /// Write an instance of a sized type to the atom space.
    ///
    /// This method uses the [`write_raw`](#method.write_raw) method to write the data. Therefore, it has similar behaviour.
    ///
    /// The type `T` may only be plain-old-data (must not contain references or pointers of any kind). Since this can not properly be checked by Rust's type system, this method is unsafe.
    pub unsafe fn write<T: Sized>(self, reference: &T) -> Result<(&'a mut T, Option<Self>), Self> {
        let size = size_of::<T>();
        let raw_data = std::slice::from_raw_parts(reference as *const T as *const u8, size);
        let (written_data, space) = self.write_raw(raw_data)?;
        let written_data = &mut *(written_data.as_mut_ptr() as *mut T);
        Ok((written_data, space))
    }
}

#[cfg(test)]
mod tests {
    use crate::atomspace::*;
    use std::alloc::*;

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

    #[test]
    fn test_mut_atom_space() {
        const MEMORY_SIZE: usize = 256;
        let layout = Layout::from_size_align(MEMORY_SIZE, 8).unwrap();
        let memory = unsafe { alloc(layout) };

        {
            let space =
                MutAtomSpace::new(unsafe { std::slice::from_raw_parts_mut(memory, MEMORY_SIZE) });

            let mut test_data: Vec<u8> = vec![0; 24];
            for i in 0..test_data.len() {
                test_data[i] = i as u8;
            }

            let space = match space.write_raw(test_data.as_slice()) {
                Ok((written_data, upper_space)) => {
                    assert_eq!(test_data.as_slice(), written_data);
                    upper_space.unwrap()
                }
                Err(_) => panic!("Writing failed!"),
            };

            let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
            match unsafe { space.write(&test_atom) } {
                Ok((written_atom, _)) => {
                    assert_eq!(written_atom.size, test_atom.size);
                    assert_eq!(written_atom.type_, test_atom.type_);
                }
                Err(_) => panic!("Writing failed!"),
            }
        }

        unsafe { dealloc(memory, layout) };
    }
}
