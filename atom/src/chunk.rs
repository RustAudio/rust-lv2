use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use urid::{URIDBound, URID};

pub struct Chunk;

unsafe impl UriBound for Chunk {
    const URI: &'static [u8] = sys::LV2_ATOM__Chunk;
}

impl URIDBound for Chunk {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.chunk
    }
}

impl Chunk {
    pub fn get(space: Space<Self>) -> Option<&[u8]> {
        space.data()
    }
}