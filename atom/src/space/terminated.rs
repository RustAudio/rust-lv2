use crate::space::{AtomError, SpaceAllocatorImpl};

pub struct Terminated<W: SpaceAllocatorImpl> {
    inner: W,
    terminator: u8,
    wrote_terminator_byte: bool,
}

impl<W: SpaceAllocatorImpl> Terminated<W> {
    pub fn new(inner: W, terminator: u8) -> Self {
        Self {
            inner,
            terminator,
            wrote_terminator_byte: false,
        }
    }
}

impl<W: SpaceAllocatorImpl> SpaceAllocatorImpl for Terminated<W> {
    fn allocate_and_split(&mut self, size: usize) -> Result<(&mut [u8], &mut [u8]), AtomError> {
        if self.wrote_terminator_byte {
            // SAFETY: We checked we already wrote the terminator byte, and it is safe to be overwritten
            unsafe { self.inner.rewind(1)? };
        }

        let (allocated, allocatable) = self.inner.allocate_and_split(size + 1)?;
        allocatable[size] = self.terminator;
        self.wrote_terminator_byte = true;

        Ok((allocated, &mut allocatable[..size]))
    }

    #[inline]
    unsafe fn rewind(&mut self, byte_count: usize) -> Result<(), AtomError> {
        self.inner.rewind(byte_count)
    }

    #[inline]
    fn allocated_bytes(&self) -> &[u8] {
        self.inner.allocated_bytes()
    }

    #[inline]
    fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        self.inner.allocated_bytes_mut()
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.inner.remaining_bytes()
    }

    #[inline]
    fn remaining_bytes_mut(&mut self) -> &mut [u8] {
        self.inner.remaining_bytes_mut()
    }
}
