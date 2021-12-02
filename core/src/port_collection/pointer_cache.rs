use core::ffi::c_void;
use core::ptr::null_mut;

/// Cache for port connection pointers.
///
/// The host will pass the port connection pointers one by one and in an undefined order. Therefore, the `PortCollection` struct can not be created instantly. Instead, the pointers will be stored in a cache, which is then used to create a proper port collection for the plugin.
pub trait PortPointerCache: Sized {
    const SIZE: usize;
    fn new() -> Self;
    fn set_connection(&mut self, index: u32) -> Option<&mut *mut c_void>;
}

impl PortPointerCache for () {
    const SIZE: usize = 0;

    #[inline]
    fn new() -> Self {}

    #[inline]
    fn set_connection(&mut self, _index: u32) -> Option<&mut *mut c_void> {
        None
    }
}

impl PortPointerCache for *mut c_void {
    const SIZE: usize = 1;

    #[inline]
    fn new() -> Self {
        null_mut()
    }

    #[inline]
    fn set_connection(&mut self, _index: u32) -> Option<&mut *mut c_void> {
        Some(self)
    }
}

impl<T: PortPointerCache + Copy, const N: usize> PortPointerCache for [T; N] {
    const SIZE: usize = N;

    #[inline]
    fn new() -> Self {
        [T::new(); N]
    }

    #[inline]
    fn set_connection(&mut self, index: u32) -> Option<&mut *mut c_void> {
        self.get_mut(index as usize)
            .and_then(|cache| cache.set_connection(0))
    }
}
