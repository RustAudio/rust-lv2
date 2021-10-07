//! Utilities for safely accessing ports by index

use std::marker::PhantomData;

pub struct PortIndex<T: ?Sized, C: ?Sized = ()>(u32, PhantomData<(Box<T>, Box<C>)>);

impl<T: ?Sized, C: ?Sized> Clone for PortIndex<T, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T: ?Sized, C: ?Sized> Copy for PortIndex<T, C> {}

impl<T, C> PortIndex<T, C> {
    #[inline]
    pub unsafe fn new_unchecked(index: u32) -> Self {
        Self(index, PhantomData)
    }

    #[inline]
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl PortIndex<()> {
    #[inline]
    pub fn new(index: u32) -> Self {
        Self(index, PhantomData)
    }
}

pub unsafe trait PortIndexable {
    type Indexes: Sized + Copy + 'static;

    fn indexes() -> Self::Indexes;
}
