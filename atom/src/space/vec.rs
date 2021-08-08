use std::mem::MaybeUninit;
use crate::space::{AllocateSpace, Space};
use std::ops::Range;

pub struct VecSpace<T> {
    inner: Vec<MaybeUninit<T>>
}

impl<T: Copy + 'static> VecSpace<T> {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: vec![MaybeUninit::zeroed(); capacity] }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        Space::<T>::from_uninit_slice(&self.inner).as_bytes()
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        Space::<T>::from_uninit_slice_mut(&mut self.inner).as_bytes_mut()
    }

    #[inline]
    fn get_or_allocate_bytes_mut(&mut self, byte_range: Range<usize>) -> Option<&mut [u8]> {
        let byte_len = self.inner.len() * ::core::mem::size_of::<T>();
        let max = byte_range.start.max(byte_range.end);

        if max > byte_len {
            let new_size = crate::space::boxed::byte_index_to_value_index::<T>(max);
            self.inner.resize(new_size, MaybeUninit::zeroed());
        }

        self.as_bytes_mut().get_mut(byte_range)
    }

    #[inline]
    pub fn cursor(&mut self) -> VecSpaceCursor<T> {
        VecSpaceCursor { vec: self, byte_index: 0 }
    }
}

pub struct VecSpaceCursor<'vec, T> {
    vec: &'vec mut VecSpace<T>,
    byte_index: usize
}

impl<'vec, T: Copy + 'static> AllocateSpace<'vec> for VecSpaceCursor<'vec, T> {
    fn allocate_unaligned(&mut self, size: usize) -> Option<&mut [u8]> {
        let end = self.byte_index.checked_add(size)?;
        VecSpace::<T>::get_or_allocate_bytes_mut(self.vec, self.byte_index..end)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.vec.as_bytes()
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.vec.as_bytes_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::space::{VecSpace, AllocateSpace};

    #[test]
    pub fn test_lifetimes () {
        let mut buffer = VecSpace::<u8>::with_capacity(16);

        {
            let mut cursor = buffer.cursor();
            let buf1 = cursor.allocate_unaligned(2).unwrap();
            buf1[0] = 5
        }

        let _other_cursor = buffer.cursor();
        let _other_cursor2 = buffer.cursor();
    }
}