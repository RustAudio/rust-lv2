#![deny(unsafe_code)]

use crate::space::error::AtomWriteError;
use crate::space::{AlignedSpace, SpaceWriterImpl, SpaceWriterSplitAllocation};
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
    pub fn as_space(&self) -> &AlignedSpace<T> {
        AlignedSpace::from_uninit_slice(&self.inner)
    }

    #[inline]
    pub fn as_space_mut(&mut self) -> &mut AlignedSpace<T> {
        AlignedSpace::from_uninit_slice_mut(&mut self.inner)
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
    fn reallocate_bytes_mut(
        &mut self,
        byte_range: Range<usize>,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError> {
        let byte_len = self.inner.len() * std::mem::size_of::<T>();
        let max = byte_range.start.max(byte_range.end);

        if max > byte_len {
            let new_size = crate::util::byte_index_to_value_index::<T>(max);
            self.inner.resize(new_size, MaybeUninit::zeroed());
        }

        let bytes = self.as_bytes_mut();

        // PANIC: We just resized to the accommodate the maximum value in the given range.
        let (previous, allocatable) = bytes.split_at_mut(byte_range.start);

        Ok(SpaceWriterSplitAllocation {
            previous,
            allocated: &mut allocatable[..byte_range.end - byte_range.start],
        })
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

impl<'vec, T: Copy + 'static> SpaceWriterImpl for VecSpaceCursor<'vec, T> {
    fn allocate_and_split(
        &mut self,
        size: usize,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError> {
        let end = self
            .allocated_length
            .checked_add(size)
            .expect("Allocation overflow");

        let result = VecSpace::<T>::reallocate_bytes_mut(self.vec, self.allocated_length..end);

        if result.is_ok() {
            self.allocated_length = end;
        }

        result
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn rewind(&mut self, byte_count: usize) -> Result<(), AtomWriteError> {
        if self.allocated_length < byte_count {
            return Err(AtomWriteError::RewindBeyondAllocated {
                requested: byte_count,
                allocated: self.allocated_length,
            });
        }

        self.allocated_length -= byte_count;

        Ok(())
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
    use crate::space::{SpaceWriter, VecSpace};

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
