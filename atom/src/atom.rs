use crate::AtomURIDs;
use core::UriBound;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::mem::{size_of};
use urid::URID;

pub trait AtomBody: UriBound {
    fn get_urid(urids: &AtomURIDs) -> URID;

    unsafe fn create_ref(bytes: &[u8]) -> Option<&Self>;
}

pub trait PreHeader: 'static + UriBound + Sized {}

#[repr(transparent)]
pub struct Atom(sys::LV2_Atom);

unsafe impl UriBound for Atom {
    const URI: &'static [u8] = sys::LV2_ATOM__Atom;
}

impl std::ops::Deref for Atom {
    type Target = sys::LV2_Atom;

    fn deref(&self) -> &sys::LV2_Atom {
        &self.0
    }
}

impl Atom {
    pub fn get_body<'a, B: AtomBody>(&'a self, urids: &AtomURIDs) -> Option<&'a B> {
        if self.0.type_ != B::get_urid(urids) {
            return None;
        }
        unsafe {
            let bytes = (self as *const Self).add(1) as *const u8;
            let bytes = std::slice::from_raw_parts(bytes, self.size as usize);
            B::create_ref(bytes)
        }
    }
}

pub struct AtomIter<'a, H = ()> {
    data: &'a [u8],
    position: usize,
    pre_header: PhantomData<H>,
}

impl<'a, H> AtomIter<'a, H> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            pre_header: PhantomData,
        }
    }

    fn has_space(&self, size: usize) -> bool {
        self.position + size < self.data.len()
    }

    unsafe fn retrieve<T: Sized>(&mut self) -> Option<&'a T> {
        if self.has_space(size_of::<H>()) {
            let pre_header = (self.data.as_ptr().add(self.position) as *const T).as_ref()?;
            self.position += size_of::<H>();
            Some(pre_header)
        } else {
            None
        }
    }
}

impl<'a> Iterator for AtomIter<'a> {
    type Item = &'a Atom;

    fn next(&mut self) -> Option<Self::Item> {
        let atom = unsafe {self.retrieve::<Atom>()?};

        if self.has_space(atom.size as usize) {
            self.position += atom.0.size as usize;
            Some(atom)
        } else {
            None
        }
    }
}

impl<'a, H: PreHeader> Iterator for AtomIter<'a, H> {
    type Item = (&'a H, &'a Atom);

    fn next(&mut self) -> Option<Self::Item> {
        let pre_header = unsafe { self.retrieve::<H>()? };
        let atom = unsafe { self.retrieve::<Atom>()?};

        if self.has_space(atom.size as usize) {
            self.position += atom.size as usize;
            Some((pre_header, atom))
        } else {
            None
        }
    }
}
