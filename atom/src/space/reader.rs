use crate::prelude::Space;
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
    fn next_uninit_value<T: 'static>(&mut self) -> Option<&'a MaybeUninit<T>> {
        let space = Space::try_align_from_bytes(self.space)?;
        let value_size = ::core::mem::size_of::<T>();
        let (value, remaining) = space.split_at(value_size)?;

        self.space = remaining.as_bytes();

        value.as_uninit()
    }

    #[inline]
    fn next_uninit_value_slice<T: 'static>(
        &mut self,
        length: usize,
    ) -> Option<&'a [MaybeUninit<T>]> {
        let space = Space::try_align_from_bytes(self.space)?;

        let split_point = crate::util::value_index_to_byte_index::<T>(length);
        let (data, remaining) = space.split_at(split_point)?;

        self.space = remaining.as_bytes();

        Some(data.as_uninit_slice())
    }

    #[inline]
    fn as_uninit_slice<T: 'static>(&self) -> Option<&'a [MaybeUninit<T>]> {
        let space = Space::try_align_from_bytes(self.space)?;
        Some(space.as_uninit_slice())
    }

    #[inline]
    pub unsafe fn as_slice<T: 'static>(&self) -> Option<&'a [T]> {
        self.as_uninit_slice()
            .map(|s| crate::util::assume_init_slice(s))
    }

    #[inline]
    pub unsafe fn next_slice<U: 'static>(&mut self, length: usize) -> Option<&'a [U]> {
        self.next_uninit_value_slice(length)
            .map(|s| crate::util::assume_init_slice(s))
    }

    #[inline]
    pub fn next_bytes(&mut self, length: usize) -> Option<&'a [u8]> {
        let bytes = self.space.get(..length)?;
        self.space = self.space.get(length..).unwrap_or(&[]);

        Some(bytes)
    }

    #[inline]
    pub unsafe fn next_value<U: 'static>(&mut self) -> Option<&'a U> {
        self.next_uninit_value()
            .map(|v| crate::util::assume_init_ref(v))
    }

    #[inline]
    pub unsafe fn next_atom(&mut self) -> Option<&'a UnidentifiedAtom> {
        let space = Space::<AtomHeader>::try_align_from_bytes(&self.space)?;
        let header = space.assume_init_value()?;
        let (_, rest) = space.split_at(header.size_of_atom())?;

        let atom = UnidentifiedAtom::from_header(header);
        self.space = rest.as_bytes();

        Some(atom)
    }

    #[inline]
    pub fn remaining_bytes(&self) -> &'a [u8] {
        self.space
    }

    #[inline]
    pub fn try_read<F, U>(&mut self, read_handler: F) -> Option<U>
    where
        F: FnOnce(&mut Self) -> Option<U>,
    {
        let mut reader = Self { space: self.space };
        let value = read_handler(&mut reader)?;
        self.space = reader.remaining_bytes();

        Some(value)
    }
}
