use crate::space::error::AtomWriteError;
use crate::space::{SpaceWriterImpl, SpaceWriterSplitAllocation};

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

impl<'a> SpaceWriterImpl for SpaceCursor<'a> {
    #[inline]
    fn allocate_and_split(
        &mut self,
        size: usize,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError> {
        let allocated_length = self.allocated_length;
        let data_len = self.data.len();
        let (previous, allocatable) = self.data.split_at_mut(allocated_length);

        let allocated = allocatable
            .get_mut(..size)
            .ok_or(AtomWriteError::OutOfSpace {
                used: allocated_length,
                capacity: data_len,
                requested: size,
            })?;

        self.allocated_length = self
            .allocated_length
            .checked_add(size)
            .expect("Allocation overflow");

        Ok(SpaceWriterSplitAllocation {
            previous,
            allocated,
        })
    }

    #[inline]
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
        &self.data[..self.allocated_length]
    }

    #[inline]
    unsafe fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data[..self.allocated_length]
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        &self.data[self.allocated_length..]
    }
}
