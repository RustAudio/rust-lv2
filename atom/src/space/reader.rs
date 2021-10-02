use crate::atom_prelude::*;
use std::mem::MaybeUninit;

/// A cursor-like struct to help read contiguous memory regions for atoms.
#[derive(Clone)]
pub struct SpaceReader<'a> {
    space: &'a [u8],
}

#[inline]
fn split_space<T: 'static>(
    space: &AlignedSpace<T>,
    bytes: usize,
) -> Result<(&AlignedSpace<T>, &[u8]), AtomReadError> {
    space
        .split_at(bytes)
        .ok_or(AtomReadError::ReadingOutOfBounds {
            requested: bytes,
            available: space.bytes_len(),
        })
}

impl<'a> SpaceReader<'a> {
    #[inline]
    pub fn new(space: &'a [u8]) -> Self {
        SpaceReader { space }
    }

    #[inline]
    fn next_uninit_value<T: 'static>(&mut self) -> Result<&'a MaybeUninit<T>, AtomReadError> {
        let space = AlignedSpace::align_from_bytes(self.space)?;
        let value_size = ::core::mem::size_of::<T>();
        let (value, remaining) = split_space(space, value_size)?;

        self.space = remaining;

        // PANIC: We just split_at the right amount of bytes for a value of T, there should be enough space
        Ok(value
            .as_uninit_slice()
            .get(0)
            .expect("Not enough space for an uninit value"))
    }

    #[inline]
    fn next_uninit_value_slice<T: 'static>(
        &mut self,
        length: usize,
    ) -> Result<&'a [MaybeUninit<T>], AtomReadError> {
        let space = AlignedSpace::align_from_bytes(self.space)?;

        let split_point = crate::util::value_index_to_byte_index::<T>(length);
        let (data, remaining) = split_space(space, split_point)?;

        self.space = remaining;

        Ok(data.as_uninit_slice())
    }

    #[inline]
    fn as_uninit_slice<T: 'static>(&self) -> Result<&'a [MaybeUninit<T>], AlignmentError> {
        let space = AlignedSpace::align_from_bytes(self.space)?;
        Ok(space.as_uninit_slice())
    }

    /// Returns the remaining bytes as a slice of type a given type `T`.
    ///
    /// # Errors
    ///
    /// This methods returns an error if the slice couldn't get correctly aligned for the type `T`.
    ///
    /// # Safety
    ///
    /// The caller of this method has to ensure the buffer is filled with properly initialized
    /// values of type `T`.
    #[inline]
    pub unsafe fn as_slice<T: 'static>(&self) -> Result<&'a [T], AlignmentError> {
        self.as_uninit_slice()
            .map(|s| crate::util::assume_init_slice(s))
    }

    /// Returns the next remaining bytes as a slice of type a given type `T` and length.
    ///
    /// # Errors
    ///
    /// This methods returns an error if the slice couldn't get correctly aligned for the type `T`,
    /// or if `length` is out of bounds.
    ///
    /// # Safety
    ///
    /// The caller of this method has to ensure the requested slice is filled with properly
    /// initialized values of type `T`.
    #[inline]
    pub unsafe fn next_values<T: 'static>(
        &mut self,
        length: usize,
    ) -> Result<&'a [T], AtomReadError> {
        self.next_uninit_value_slice(length)
            .map(|s| crate::util::assume_init_slice(s))
    }

    /// Returns the next `length` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if `length` is out of bounds.
    #[inline]
    pub fn next_bytes(&mut self, length: usize) -> Result<&'a [u8], AtomReadError> {
        let bytes = self
            .space
            .get(..length)
            .ok_or_else(|| AtomReadError::ReadingOutOfBounds {
                requested: length,
                available: self.space.len(),
            })?;

        self.space = self.space.get(length..).unwrap_or(&[]);

        Ok(bytes)
    }

    /// Returns the next value as a given type `T`.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is too big for the remaining buffer, or if the buffer cannot
    /// be aligned to match the value's alignment requirements.
    ///
    /// # Safety
    ///
    /// The caller is responsible to ensure that a properly initialized value of type `T` is present.
    #[inline]
    pub unsafe fn next_value<T: 'static>(&mut self) -> Result<&'a T, AtomReadError> {
        self.next_uninit_value()
            .map(|v| crate::util::assume_init_ref(v))
    }

    /// Returns the next atom.
    ///
    /// This method reads the next atom header, and then returns both the header and the associated
    /// body as an [`UnidentifiedAtom`].
    ///
    /// # Errors
    ///
    /// Returns an error if the value is too big for the remaining buffer, if the buffer cannot
    /// be aligned to match the value's alignment requirements, or if the atom's body side is out
    /// of bounds.
    ///
    /// # Safety
    ///
    /// The caller is responsible to ensure that a properly initialized atom is present.
    #[inline]
    pub unsafe fn next_atom(&mut self) -> Result<&'a UnidentifiedAtom, AtomReadError> {
        let space = AlignedSpace::<AtomHeader>::align_from_bytes(self.space)?;
        let header = space
            .assume_init_slice()
            .get(0)
            .ok_or(AtomReadError::ReadingOutOfBounds {
                available: space.bytes_len(),
                requested: core::mem::size_of::<AtomHeader>(),
            })?;
        let (_, rest) = split_space(space, header.size_of_atom())?;

        let atom = UnidentifiedAtom::from_header(header);
        self.space = rest;

        Ok(atom)
    }

    #[inline]
    pub fn remaining_bytes(&self) -> &'a [u8] {
        self.space
    }

    /// Performs a given reading operation, but only advance the cursor if the operation is successful.
    ///
    /// # Errors
    ///
    /// Returns whichever errors the given operation handler returned.
    #[inline]
    pub fn try_read<F, U>(&mut self, read_handler: F) -> Result<U, AtomReadError>
    where
        F: FnOnce(&mut Self) -> Result<U, AtomReadError>,
    {
        let mut reader = self.clone();
        let value = read_handler(&mut reader)?;
        self.space = reader.remaining_bytes();

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::space::VecSpace;
    use std::mem::{size_of, size_of_val};
    use urid::URID;

    #[test]
    fn test_read_atom() {
        let mut space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let space = space.as_space_mut();
        let urid: URID = unsafe { URID::new_unchecked(17) };

        // Writing an integer atom.
        unsafe {
            *(space.as_bytes_mut().as_mut_ptr() as *mut sys::LV2_Atom_Int) = sys::LV2_Atom_Int {
                atom: sys::LV2_Atom {
                    size: size_of::<i32>() as u32,
                    type_: urid.get(),
                },
                body: 42,
            };

            let atom = space.read().next_atom().unwrap();
            let body = atom.body().as_bytes();

            assert_eq!(size_of::<i32>(), size_of_val(body));
            assert_eq!(42, *(body.as_ptr() as *const i32));
        }
    }
}
