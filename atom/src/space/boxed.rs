use crate::prelude::Space;
use std::mem::{size_of, MaybeUninit};
use std::ops::{Deref, DerefMut};

pub(crate) fn byte_index_to_value_index<T>(size: usize) -> usize {
    let type_size = size_of::<T>();
    if type_size == 0 {
        0
    } else {
        size / type_size + if size % type_size > 0 { 1 } else { 0 }
    }
}

pub(crate) struct BoxedSpace<T: 'static> {
    pub(crate) inner: Box<[MaybeUninit<T>]>,
}

impl<T: Copy + 'static> BoxedSpace<T> {
    #[inline]
    pub fn new_zeroed(size: usize) -> Self {
        Self {
            inner: vec![MaybeUninit::zeroed(); byte_index_to_value_index::<T>(size)]
                .into_boxed_slice(),
        }
    }
}

impl<T: 'static> Deref for BoxedSpace<T> {
    type Target = Space<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Space::<T>::from_uninit_slice(&self.inner)
    }
}

impl<T: 'static> DerefMut for BoxedSpace<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Space::<T>::from_uninit_slice_mut(&mut self.inner)
    }
}
