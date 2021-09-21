use crate::space::error::AlignmentError;
use std::any::TypeId;
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
pub(crate) fn write_uninit<T>(uninit: &mut MaybeUninit<T>, value: T) -> &mut T {
    *uninit = MaybeUninit::new(value);
    // SAFETY: we just wrote the value, therefore it is initialized now
    unsafe { assume_init_mut(uninit) }
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
pub(crate) fn try_padding_for<T: 'static>(data: &[u8]) -> Result<usize, AlignmentError> {
    let value = data.as_ptr().align_offset(::core::mem::align_of::<T>());
    if value == usize::MAX {
        Err(AlignmentError::CannotComputeAlignment {
            type_id: TypeId::of::<T>(),
            ptr: data.as_ptr(),
        })
    } else {
        Ok(value)
    }
}
