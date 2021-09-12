use crate::space::{AtomError, SpaceAllocatorImpl};

pub struct SpaceCursor<'a> {
    data: &'a mut [u8],
    allocated_length: usize,
}

impl<'a> SpaceCursor<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self {
            data,
            allocated_length: 0,
        }
    }
}

impl<'a> SpaceAllocatorImpl for SpaceCursor<'a> {
    #[inline]
    fn allocate_and_split(&mut self, size: usize) -> Result<(&mut [u8], &mut [u8]), AtomError> {
        let allocated_length = self.allocated_length;
        let data_len = self.data.len();
        let (allocated, allocatable) = self.data.split_at_mut(allocated_length);

        let new_allocation = allocatable
            .get_mut(..size)
            .ok_or_else(|| AtomError::OutOfSpace {
                used: allocated_length,
                capacity: data_len,
                requested: size,
            })?;

        self.allocated_length = self
            .allocated_length
            .checked_add(size)
            .ok_or(AtomError::AllocatorOverflow)?;

        Ok((allocated, new_allocation))
    }

    #[inline]
    unsafe fn rewind(&mut self, byte_count: usize) -> bool {
        if self.allocated_length < byte_count {
            return false;
        }

        self.allocated_length -= byte_count;

        true
    }

    #[inline]
    fn allocated_bytes(&self) -> &[u8] {
        &self.data[..self.allocated_length]
    }

    #[inline]
    fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data[..self.allocated_length]
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        &self.data[self.allocated_length..]
    }

    #[inline]
    fn remaining_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data[self.allocated_length..]
    }
}
