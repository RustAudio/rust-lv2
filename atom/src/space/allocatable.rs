use crate::Atom;
use urid::URID;
use crate::space::{AtomSpaceWriter, Space};

use core::mem::size_of_val;
use std::mem::MaybeUninit;

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
pub trait AllocateSpace<'a>: 'a {
    /// Try to allocate memory on the internal data slice.
    ///
    /// If `apply_padding` is `true`, the method will assure that the allocated memory is 64-bit-aligned. The first return value is the number of padding bytes that has been used and the second return value is a mutable slice referencing the allocated data.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]>;

    fn as_bytes(&self) -> &[u8];
    fn as_bytes_mut(&mut self) -> &mut [u8];
}

impl<'a> AllocateSpace<'a> for &'a mut [u8] {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        if size > self.len() {
            return None
        }

        let slice: &'a mut [u8] = ::core::mem::replace(self, &mut []); // Lifetime workaround
        let (allocated, remaining) = slice.split_at_mut(size);
        *self = remaining;
        Some(allocated)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl<'a> AllocateSpace<'a> for &'a mut dyn AllocateSpace<'a> {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        (*self).allocate_unaligned(size)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        let s = Clone::clone(&self);
        s.as_bytes()
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] { (*self).as_bytes_mut() }
}

#[inline]
pub fn allocate<'a, T: 'static>(space: &mut impl AllocateSpace<'a>, size: usize) -> Option<&'a mut Space<T>> {
    let required_padding = Space::<T>::padding_for(space.as_bytes());
    let raw = space.allocate_unaligned(size + required_padding)?;

    Space::try_align_from_bytes_mut(raw)
}

#[inline]
pub fn allocate_values<'a, T: 'static>(space: &mut impl AllocateSpace<'a>, count: usize) -> Option<&'a mut [MaybeUninit<T>]> {
    let space = allocate(space, count * ::core::mem::size_of::<T>())?;
    Some(space.as_uninit_slice_mut())
}

#[inline]
pub fn init_atom<'a, 'b, A: Atom<'a, 'b>>(space: &'b mut impl AllocateSpace<'b>, atom_type: URID<A>, write_parameter: A::WriteParameter) -> Option<A::WriteHandle> {
    let space = AtomSpaceWriter::write_new(space, atom_type)?;
    A::init(space, write_parameter)
}

#[inline]
pub fn write_bytes<'a>(space: &mut impl AllocateSpace<'a>, bytes: &[u8]) -> Option<&'a mut [u8]> {
    let space = space.allocate_unaligned(bytes.len())?;
    space.copy_from_slice(bytes);
    Some(space)
}

#[inline]
pub fn write_value<'a, T>(space: &mut impl AllocateSpace<'a>, value: T) -> Option<&'a mut T>
    where T: Copy + Sized + 'static {
    let space = allocate(space, size_of_val(&value))?;
    // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
    let space = unsafe { space.as_uninit_mut_unchecked() };
    *space = MaybeUninit::new(value);

    // SAFETY: the MaybeUninit has now been properly initialized.
    Some (unsafe { &mut *(space.as_mut_ptr()) })
}

pub fn write_values<'a, T>(space: &mut impl AllocateSpace<'a>, values: &[T]) -> Option<&'a mut [T]>
    where T: Copy + Sized + 'static {
    let space: &mut Space<T> = allocate(space, size_of_val(values))?;
    let space = space.as_uninit_slice_mut();

    for (dst, src) in space.iter_mut().zip(values.iter()) {
        *dst = MaybeUninit::new(*src)
    }

    // SAFETY: Assume init: we just initialized the memory above
    Some(unsafe { &mut *(space as *mut [_] as *mut [T]) })
}
