use crate::Atom;
use urid::URID;
use crate::space::{AtomSpace, Space};

use core::mem::size_of_val;
use std::mem::MaybeUninit;

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
pub trait AllocateSpace<'a> {
    /// Try to allocate memory on the internal data slice.
    ///
    /// If `apply_padding` is `true`, the method will assure that the allocated memory is 64-bit-aligned. The first return value is the number of padding bytes that has been used and the second return value is a mutable slice referencing the allocated data.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]>;

    fn as_bytes(&self) -> &[u8];
}

impl<'a> AllocateSpace<'a> for &'a mut [u8] {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        if size > self.space.len() {
            return None
        }

        let (allocated, remaining) = self.space.split_at_mut(size);
        self.space = remaining;
        Some(allocated)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

#[inline]
pub fn allocate<'a, T>(space: &mut impl AllocateSpace<'a>, size: usize) -> Option<&'a mut Space<T>> {
    let required_padding = crate::space::space::required_padding_for_type::<T>(space.as_bytes());
    let raw = space.allocate_unaligned(size + required_padding)?;

    Space::try_align_from_bytes_mut(raw)
}

#[inline]
pub fn init_atom<'a, 'b, A: Atom<'a, 'b>>(space: &mut impl AllocateSpace<'b>, atom_type: URID<A>, write_parameter: A::WriteParameter) -> Option<A::WriteHandle> {
    let space = AtomSpace::write_new(space, atom_type)?;
    A::init(space, write_parameter)
}

#[inline]
pub fn write_bytes<'a>(space: &mut impl AllocateSpace<'a>, bytes: &[u8]) -> Option<&'a mut [u8]> {
    let (_, space) = space.allocate_unaligned(bytes.len())?;
    space.as_bytes_mut().copy_from_slice(bytes);
    Some(space)
}

#[inline]
pub fn write_value<'a, T>(space: &mut impl AllocateSpace<'a>, value: T) -> Option<&'a mut T>
    where T: Copy + Sized + 'static
{
    let space = allocate(space, size_of_val(&value))?;
    // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
    let space = unsafe { space.as_uninit_mut_unchecked() };
    *space = MaybeUninit::new(value);

    // SAFETY: the MaybeUninit has now been properly initialized.
    Some (unsafe { &mut *(space.as_mut_ptr()) })
}
