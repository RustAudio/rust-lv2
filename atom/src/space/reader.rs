use crate::prelude::AlignedSpace;
use crate::space::error::AtomReadError;
use crate::{AtomHeader, UnidentifiedAtom};
use std::mem::MaybeUninit;

#[derive(Clone)]
pub struct SpaceReader<'a> {
    space: &'a [u8],
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
        let (value, remaining) = space.split_at(value_size)?;

        self.space = remaining.as_bytes();

        // PANIC: We just split_at the right amount of bytes for a value of T, there should be enough space
        Ok(value
            .as_uninit()
            .expect("Not enough space for an uninit value"))
    }

    #[inline]
    fn next_uninit_value_slice<T: 'static>(
        &mut self,
        length: usize,
    ) -> Result<&'a [MaybeUninit<T>], AtomReadError> {
        let space = AlignedSpace::align_from_bytes(self.space)?;

        let split_point = crate::util::value_index_to_byte_index::<T>(length);
        let (data, remaining) = space.split_at(split_point)?;

        self.space = remaining.as_bytes();

        Ok(data.as_uninit_slice())
    }

    #[inline]
    fn as_uninit_slice<T: 'static>(&self) -> Result<&'a [MaybeUninit<T>], AtomReadError> {
        let space = AlignedSpace::align_from_bytes(self.space)?;
        Ok(space.as_uninit_slice())
    }

    #[inline]
    pub unsafe fn as_slice<T: 'static>(&self) -> Result<&'a [T], AtomReadError> {
        self.as_uninit_slice()
            .map(|s| crate::util::assume_init_slice(s))
    }

    #[inline]
    pub unsafe fn next_slice<U: 'static>(
        &mut self,
        length: usize,
    ) -> Result<&'a [U], AtomReadError> {
        self.next_uninit_value_slice(length)
            .map(|s| crate::util::assume_init_slice(s))
    }

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

    #[inline]
    pub unsafe fn next_value<U: 'static>(&mut self) -> Result<&'a U, AtomReadError> {
        self.next_uninit_value()
            .map(|v| crate::util::assume_init_ref(v))
    }

    #[inline]
    pub unsafe fn next_atom(&mut self) -> Result<&'a UnidentifiedAtom, AtomReadError> {
        let space = AlignedSpace::<AtomHeader>::align_from_bytes(&self.space)?;
        let header = space
            .assume_init_value()
            .ok_or(AtomReadError::ReadingOutOfBounds {
                available: space.len(),
                requested: core::mem::size_of::<AtomHeader>(),
            })?;
        let (_, rest) = space.split_at(header.size_of_atom())?;

        let atom = UnidentifiedAtom::from_header(header);
        self.space = rest.as_bytes();

        Ok(atom)
    }

    #[inline]
    pub fn remaining_bytes(&self) -> &'a [u8] {
        self.space
    }

    #[inline]
    pub fn try_read<F, U>(&mut self, read_handler: F) -> Result<U, AtomReadError>
    where
        F: FnOnce(&mut Self) -> Result<U, AtomReadError>,
    {
        let mut reader = Self { space: self.space };
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
