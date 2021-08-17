use crate::space::SpaceAllocator;

pub struct SpaceCursor<'a> {
    data: &'a mut [u8],
    index: usize
}

impl<'a> SpaceCursor<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data, index: 0 }
    }
}

impl<'a> SpaceAllocator<'a> for SpaceCursor<'a> {
    fn allocate_unaligned(&mut self, size: usize) -> Option<&mut [u8]> {
        todo!()
    }

    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])> {
        todo!()
    }

    fn allocated_bytes(&self) -> &[u8] {
        todo!()
    }

    fn allocated_bytes_mut(&mut self) -> &mut [u8] {
        todo!()
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        self.data
    }

    #[inline]
    fn remaining_bytes_mut(&mut self) -> &mut [u8] {
        self.data
    }
}