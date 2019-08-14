use crate::feature::*;
use core::UriBound;
use std::cmp::PartialEq;
use std::ffi::CStr;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct URID(u32);

impl URID {
    pub fn new(urid: u32) -> Option<Self> {
        if urid != 0 {
            Some(Self(urid))
        } else {
            None
        }
    }

    pub unsafe fn new_unchecked(urid: u32) -> Self {
        Self(urid)
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
}

impl Deref for URID {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

impl PartialEq<u32> for URID {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<URID> for u32 {
    fn eq(&self, other: &URID) -> bool {
        *self == other.0
    }
}
