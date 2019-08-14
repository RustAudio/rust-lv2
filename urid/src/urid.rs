use crate::feature::*;
use core::UriBound;
use std::cmp::{PartialEq, PartialOrd, Ordering};
use std::ffi::CStr;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct URID(NonZeroU32);

impl URID {
    pub fn new(urid: u32) -> Option<Self> {
        Some(Self(NonZeroU32::new(urid)?))
    }

    pub unsafe fn new_unchecked(urid: u32) -> Self {
        Self(NonZeroU32::new_unchecked(urid))
    }

    pub fn from_type<T: UriBound>(map: &Map) -> Option<Self> {
        map.map(T::uri())
    }

    pub fn from_uri(map: &Map, uri: &CStr) -> Option<Self> {
        map.map(uri)
    }

    pub fn to_cstr<'a>(self, unmap: &'a Unmap) -> Option<&'a CStr> {
        unmap.unmap(self)
    }

    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

impl PartialEq<u32> for URID {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl PartialEq<URID> for u32 {
    fn eq(&self, other: &URID) -> bool {
        *self == other.get()
    }
}

impl PartialOrd<u32> for URID {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl PartialOrd<URID> for u32 {
    fn partial_cmp(&self, other: &URID) -> Option<Ordering> {
        self.partial_cmp(&other.get())
    }
}

#[cfg(test)]
#[test]
fn test_urid_size() {
    use std::mem::size_of;

    let size = size_of::<u32>();

    assert_eq!(size, size_of::<URID>());
    assert_eq!(size, size_of::<Option<URID>>());
}