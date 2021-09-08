use std::mem::MaybeUninit;

// This function is separate to ensure proper lifetimes
#[inline]
pub(crate) unsafe fn assume_init_ref<T>(s: &MaybeUninit<T>) -> &T {
    // SAFETY: the caller must guarantee that `self` is initialized.
    &*s.as_ptr()
}

// This function is separate to ensure proper lifetimes
#[inline]
pub(crate) unsafe fn assume_init_mut<T>(s: &mut MaybeUninit<T>) -> &mut T {
    // SAFETY: the caller must guarantee that `self` is initialized.
    &mut *s.as_mut_ptr()
}

#[inline]
pub(crate) unsafe fn assume_init_slice<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    &*(slice as *const _ as *const [T])
}

#[inline]
pub(crate) fn value_index_to_byte_index<T>(size: usize) -> usize {
    size * ::core::mem::size_of::<T>()
}

#[inline]
pub(crate) fn byte_index_to_value_index<T>(size: usize) -> usize {
    let type_size = ::core::mem::size_of::<T>();
    if type_size == 0 {
        0
    } else {
        size / type_size + if size % type_size > 0 { 1 } else { 0 }
    }
}

#[inline]
pub(crate) fn padding_for<T>(data: &[u8]) -> Option<usize> {
    let value = data.as_ptr().align_offset(::core::mem::align_of::<T>());
    if value == usize::MAX {
        None
    } else {
        Some(value)
    }
}
