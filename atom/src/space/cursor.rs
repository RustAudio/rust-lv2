use crate::space::SpaceAllocatorImpl;

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

impl<'a> SpaceAllocatorImpl<'a> for SpaceCursor<'a> {
    #[inline]
    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])> {
        let (allocated, allocatable) = self.data.split_at_mut(self.allocated_length);
        let new_allocation = allocatable.get_mut(..size)?;
        self.allocated_length = self.allocated_length.checked_add(size)?;

        Some((allocated, new_allocation))
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
