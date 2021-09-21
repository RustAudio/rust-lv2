use crate::prelude::AlignedSpace;
use crate::space::error::AtomReadError;
use crate::{AtomHeader, UnidentifiedAtom};
use std::mem::MaybeUninit;

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
        let space = AlignedSpace::try_align_from_bytes(self.space)?;
        let value_size = ::core::mem::size_of::<T>();
        let (value, remaining) = space.try_split_at(value_size)?;

        self.space = remaining.as_bytes();

        // This shouldn't be possible, but it doesn't hurt to check
        value.as_uninit().ok_or(AtomReadError::Unknown)
    }

    #[inline]
    fn next_uninit_value_slice<T: 'static>(
        &mut self,
        length: usize,
    ) -> Result<&'a [MaybeUninit<T>], AtomReadError> {
        let space = AlignedSpace::try_align_from_bytes(self.space)?;

        let split_point = crate::util::value_index_to_byte_index::<T>(length);
        let (data, remaining) = space.try_split_at(split_point)?;

        self.space = remaining.as_bytes();

        Ok(data.as_uninit_slice())
    }

    #[inline]
    fn as_uninit_slice<T: 'static>(&self) -> Result<&'a [MaybeUninit<T>], AtomReadError> {
        let space = AlignedSpace::try_align_from_bytes(self.space)?;
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
        let space = AlignedSpace::<AtomHeader>::try_align_from_bytes(&self.space)?;
        let header = space
            .assume_init_value()
            .ok_or(AtomReadError::ReadingOutOfBounds {
                available: space.len(),
                requested: core::mem::size_of::<AtomHeader>(),
            })?;
        let (_, rest) = space.try_split_at(header.size_of_atom())?;

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
