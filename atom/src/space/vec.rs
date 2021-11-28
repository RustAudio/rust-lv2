#![deny(unsafe_code)]

use crate::space::error::AtomWriteError;
use crate::space::{AlignedSpace, SpaceAllocator, SpaceWriterSplitAllocation};
use std::mem::MaybeUninit;
use std::ops::Range;

/// A heap-allocated growable byte buffer with the alignment of a type `T`.
///
/// This type is useful to create heap-allocated [`AlignedSpace`s](crate::space::AlignedSpace), i.e. heap-allocated
/// aligned byte buffers, to be used for e.g. safely writing properly-aligned atoms.
///
/// # Example
///
/// ```
/// # use lv2_atom::space::{AlignedVec, SpaceWriter};
/// use lv2_atom::AtomHeader;
///
/// let mut buffer = AlignedVec::<AtomHeader>::new_with_capacity(64);
///
/// // This buffer is always aligned!
/// assert_eq!(buffer.as_bytes().as_ptr() as usize % core::mem::align_of::<AtomHeader>(), 0);
///
/// // We can now safely write atoms into it.
/// let mut cursor = buffer.write();
/// // ...
/// # core::mem::drop(cursor)
/// ```
pub struct AlignedVec<T> {
    inner: Vec<MaybeUninit<T>>,
}

impl<T: Copy + 'static> Default for AlignedVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + 'static> Clone for AlignedVec<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Copy + 'static> AlignedVec<T> {
    /// Creates a new, empty buffer.
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a new buffer, with an internal capacity a given amount of `T` items.
    ///
    /// Note that `capacity` is a number of `T` items, *not* a size in bytes.
    #[inline]
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            inner: vec![MaybeUninit::zeroed(); capacity],
        }
    }

    /// Resizes the buffer to a new internal capacity a given amount of `T` items.
    ///
    /// Note that `capacity` is a number of `T` items, *not* a size in bytes.
    #[inline]
    pub fn resize(&mut self, new_len: usize) {
        self.inner.resize(new_len, MaybeUninit::zeroed())
    }

    /// Returns the contents of the buffer as an aligned byte slice.
    #[inline]
    pub fn as_space(&self) -> &AlignedSpace<T> {
        AlignedSpace::from_uninit_slice(&self.inner)
    }

    /// Returns the contents of the buffer as a mutable aligned byte slice.
    #[inline]
    pub fn as_space_mut(&mut self) -> &mut AlignedSpace<T> {
        AlignedSpace::from_uninit_slice_mut(&mut self.inner)
    }

    /// Returns the contents of the buffer as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_space().as_bytes()
    }

    /// Returns the contents of the buffer as a mutable byte slice.
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

    /// Returns a new writer to write into the contents of the buffer.
    ///
    /// Unlike other [`SpaceWriter`](crate::space::SpaceWriter) implementations, this cursor grows the underlying
    /// [`AlignedVec`] buffer if it runs out of space, instead of failing.
    #[inline]
    pub fn write(&mut self) -> AlignedVecCursor<T> {
        AlignedVecCursor {
            vec: self,
            allocated_length: 0,
        }
    }

    #[inline]
    pub fn into_boxed_space(self) -> Box<AlignedSpace<T>> {
        AlignedSpace::from_boxed_uninit_slice(self.inner.into_boxed_slice())
    }

    #[inline]
    pub fn into_vec(self) -> Vec<MaybeUninit<T>> {
        self.inner
    }

    #[inline]
    pub fn from_vec(vec: Vec<MaybeUninit<T>>) -> Self {
        Self { inner: vec }
    }
}

/// A lightweight [`SpaceWriter`](crate::space::SpaceWriter) that writes into a growable byte buffer (backed by [`AlignedVec`]) using a cursor.
///
/// Unlike other [`SpaceWriter`](crate::space::SpaceWriter) implementations, this cursor grows the underlying
/// [`AlignedVec`] buffer if it runs out of space, instead of failing.
///
/// This cursor is obtained through the [`AlignedVec::write`] method.
pub struct AlignedVecCursor<'vec, T> {
    vec: &'vec mut AlignedVec<T>,
    allocated_length: usize,
}

impl<'vec, T: Copy + 'static> SpaceAllocator for AlignedVecCursor<'vec, T> {
    fn allocate_and_split(
        &mut self,
        size: usize,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError> {
        let end = self
            .allocated_length
            .checked_add(size)
            .expect("Allocation overflow");

        let result = AlignedVec::<T>::reallocate_bytes_mut(self.vec, self.allocated_length..end);

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
    #[allow(unsafe_code)]
    unsafe fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.vec.as_bytes_mut()[..self.allocated_length]
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.vec
            .as_bytes()
            .get(self.allocated_length..)
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use crate::space::{AlignedVec, SpaceWriter};
    use crate::AtomHeader;

    #[test]
    pub fn test_lifetimes() {
        let mut buffer = AlignedVec::<u8>::new_with_capacity(16);

        {
            let mut cursor = buffer.write();
            let buf1 = cursor.allocate(2).unwrap();
            buf1[0] = 5
        }

        let _other_cursor = buffer.write();
        let _other_cursor2 = buffer.write();
    }

    #[test]
    pub fn test_alignment() {
        fn aligned_vec<T: Copy + 'static>() {
            let space = AlignedVec::<T>::new_with_capacity(4);
            assert_eq!(
                space.as_bytes().as_ptr() as usize % ::core::mem::align_of::<T>(),
                0
            );
        }

        // Testing with some random types with different alignments
        aligned_vec::<u8>();
        aligned_vec::<u16>();
        aligned_vec::<u32>();
        aligned_vec::<u64>();
        aligned_vec::<AtomHeader>();
    }
}
