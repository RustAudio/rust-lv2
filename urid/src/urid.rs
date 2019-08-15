use crate::feature::*;
use core::UriBound;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct URID<T = ()>(NonZeroU32, PhantomData<T>);

impl<T> URID<T> {
    pub unsafe fn new_unchecked(urid: u32) -> Self {
        Self(NonZeroU32::new_unchecked(urid), PhantomData)
    }

    pub fn get(&self) -> u32 {
        self.0.get()
    }

    pub fn into_cstr<'a>(self, unmap: &'a Unmap) -> Option<&'a CStr> {
        unmap.unmap(self)
    }
}

impl URID<()> {
    pub fn new(urid: u32) -> Option<Self> {
        Some(Self(NonZeroU32::new(urid)?, PhantomData))
    }

    pub fn from_uri(map: &Map, uri: &CStr) -> Option<Self> {
        map.map_uri(uri)
    }
}

impl<T: UriBound> URID<T> {
    pub fn from_type(map: &Map) -> Option<Self> {
        map.map_type::<T>()
    }

    pub fn into_general(self) -> URID<()> {
        URID::new(self.get()).unwrap()
    }
}

impl<T> fmt::Debug for URID<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> PartialEq<u32> for URID<T> {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl<T> PartialEq<URID<T>> for u32 {
    fn eq(&self, other: &URID<T>) -> bool {
        *self == other.get()
    }
}

impl<T> PartialOrd<u32> for URID<T> {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl<T> PartialOrd<URID<T>> for u32 {
    fn partial_cmp(&self, other: &URID<T>) -> Option<Ordering> {
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
