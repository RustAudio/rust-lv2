#![deny(unsafe_code)]

use crate::space::{Space, SpaceAllocator};
use std::mem::MaybeUninit;
use std::ops::Range;

pub struct VecSpace<T> {
    inner: Vec<MaybeUninit<T>>,
}

impl<T: Copy + 'static> VecSpace<T> {
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    #[inline]
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            inner: vec![MaybeUninit::zeroed(); capacity],
        }
    }

    #[inline]
    pub fn as_space(&self) -> &Space<T> {
        Space::from_uninit_slice(&self.inner)
    }

    #[inline]
    pub fn as_space_mut(&mut self) -> &mut Space<T> {
        Space::from_uninit_slice_mut(&mut self.inner)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_space().as_bytes()
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.as_space_mut().as_bytes_mut()
    }

    #[inline]
    fn get_or_allocate_bytes_mut(
        &mut self,
        byte_range: Range<usize>,
    ) -> Option<(&mut [u8], &mut [u8])> {
        let byte_len = self.inner.len() * ::core::mem::size_of::<T>();
        let max = byte_range.start.max(byte_range.end);

        if max > byte_len {
            let new_size = crate::space::boxed::byte_index_to_value_index::<T>(max);
            self.inner.resize(new_size, MaybeUninit::zeroed());
        }

        let bytes = self.as_bytes_mut();
        bytes.get(byte_range.clone())?; // To make sure everything is in range instead of panicking on split_at_mut
        let (previous, allocatable) = bytes.split_at_mut(byte_range.start);

        return Some((
            previous,
            allocatable.get_mut(..byte_range.end - byte_range.start)?,
        ));
    }

    #[inline]
    pub fn cursor(&mut self) -> VecSpaceCursor<T> {
        VecSpaceCursor {
            vec: self,
            allocated_length: 0,
        }
    }
}

pub struct VecSpaceCursor<'vec, T> {
    vec: &'vec mut VecSpace<T>,
    allocated_length: usize,
}

impl<'vec, T: Copy + 'static> SpaceAllocator<'vec> for VecSpaceCursor<'vec, T> {
    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])> {
        let end = self.allocated_length.checked_add(size)?;
        let result = VecSpace::<T>::get_or_allocate_bytes_mut(self.vec, self.allocated_length..end);

        if result.is_some() {
            self.allocated_length = end;
        }

        result
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn rewind(&mut self, byte_count: usize) -> bool {
        if self.allocated_length < byte_count {
            return false;
        }

        self.allocated_length -= byte_count;

        true
    }

    #[inline]
    fn allocated_bytes(&self) -> &[u8] {
        &self.vec.as_bytes()[..self.allocated_length]
    }

    #[inline]
    fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.vec.as_bytes_mut()[..self.allocated_length]
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.vec
            .as_bytes()
            .get(self.allocated_length..)
            .unwrap_or(&[])
    }

    #[inline]
    fn remaining_bytes_mut(&mut self) -> &mut [u8] {
        self.vec
            .as_bytes_mut()
            .get_mut(self.allocated_length..)
            .unwrap_or(&mut [])
    }
}

#[cfg(test)]
mod tests {
    use crate::space::{SpaceAllocator, VecSpace};

    #[test]
    pub fn test_lifetimes() {
        let mut buffer = VecSpace::<u8>::new_with_capacity(16);

        {
            let mut cursor = buffer.cursor();
            let buf1 = cursor.allocate(2).unwrap();
            buf1[0] = 5
        }

        let _other_cursor = buffer.cursor();
        let _other_cursor2 = buffer.cursor();
    }
}
