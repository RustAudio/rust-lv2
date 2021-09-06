use crate::prelude::Space;
use crate::{AtomHeader, UnidentifiedAtom};
use std::mem::MaybeUninit;

pub struct SpaceReader<'a, T> {
    space: &'a Space<T>,
}

impl<'a, T: 'static> SpaceReader<'a, T> {
    #[inline]
    pub fn new(space: &'a Space<T>) -> Self {
        SpaceReader { space }
    }

    #[inline]
    fn next_uninit_value<U: 'static>(&mut self) -> Option<&'a MaybeUninit<U>> {
        let space = self.space.realign()?;
        let value_size = ::core::mem::size_of::<U>();
        let (value, remaining) = space.split_at(value_size)?;

        self.space = remaining.realign().unwrap_or_else(Space::empty);

        value.as_uninit()
    }

    #[inline]
    fn next_uninit_value_slice<U: 'static>(
        &mut self,
        length: usize,
    ) -> Option<&'a [MaybeUninit<U>]> {
        let space = self.space.realign()?;
        let split_point = crate::util::value_index_to_byte_index::<U>(length);
        let (data, remaining) = space.split_at(split_point)?;

        self.space = remaining.realign().unwrap_or_else(Space::empty);

        Some(data.as_uninit_slice())
    }

    #[inline]
    fn as_uninit_slice<U: 'static>(&self) -> Option<&'a [MaybeUninit<U>]> {
        Some(self.space.realign()?.as_uninit_slice())
    }

    #[inline]
    pub unsafe fn as_slice<U: 'static>(&self) -> Option<&'a [U]> {
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
        let split_point = crate::util::value_index_to_byte_index::<u8>(length);

        let (data, remaining) = self.space.split_at(split_point)?;
        self.space = remaining.realign().unwrap_or_else(Space::empty);

        Some(data.as_bytes())
    }

    #[inline]
    pub unsafe fn next_value<U: 'static>(&mut self) -> Option<&'a U> {
        self.next_uninit_value()
            .map(|v| crate::util::assume_init_ref(v))
    }

    #[inline]
    pub fn into_remaining(self) -> &'a Space<T> {
        self.space
    }

    #[inline]
    pub fn try_read<F, U>(&mut self, read_handler: F) -> Option<U>
    where
        F: FnOnce(&mut Self) -> Option<U>,
    {
        let mut reader = self.space.read();
        let value = read_handler(&mut reader)?;
        self.space = reader.into_remaining();

        Some(value)
    }
}

pub type AtomSpaceReader<'a> = SpaceReader<'a, AtomHeader>;

impl<'a> AtomSpaceReader<'a> {
    #[inline]
    pub unsafe fn next_atom(&mut self) -> Option<&'a UnidentifiedAtom> {
        let header = self.space.assume_init_value()?;
        let (_, rest) = self.space.split_at(header.size_of_atom())?;

        let atom = UnidentifiedAtom::from_header(header);
        self.space = rest;

        Some(atom)
    }
}
