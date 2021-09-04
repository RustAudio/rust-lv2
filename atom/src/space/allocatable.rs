use crate::space::{AtomSpaceWriter, Space};
use crate::{Atom, UnidentifiedAtom};
use urid::URID;

use core::mem::size_of_val;
use std::mem::MaybeUninit;

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
pub trait SpaceAllocator<'a> {
    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])>;

    #[must_use]
    unsafe fn rewind(&mut self, byte_count: usize) -> bool;

    fn allocated_bytes(&self) -> &[u8];
    fn allocated_bytes_mut(&mut self) -> &mut [u8];

    fn remaining_bytes(&self) -> &[u8];
    fn remaining_bytes_mut(&mut self) -> &mut [u8];

    /// Try to allocate memory on the internal data slice.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    #[inline]
    fn allocate(&mut self, size: usize) -> Option<&mut [u8]> {
        self.allocate_and_split(size).map(|(_, s)| s)
    }
}
/*
impl<'a> SpaceAllocator<'a> for &'a mut [u8] {
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
    fn remaining_bytes(&self) -> &[u8] {
        self
    }

    #[inline]
    fn remaining_bytes_mut(&mut self) -> &mut [u8] {
        self
    }
}*/

/*
#[inline]
pub fn realign<'a, T: 'static, S: AllocateSpace<'a>>(space: &mut S) -> Option<()> {
    let required_padding = Space::<T>::padding_for(space.as_bytes());
    let _ = space.allocate_unaligned(required_padding)?;
    Some(())
}*/

// This function is separate to ensure proper lifetimes
unsafe fn assume_init_mut<T>(s: &mut MaybeUninit<T>) -> &mut T {
    // SAFETY: the caller must guarantee that `self` is initialized.
    // This also means that `self` must be a `value` variant.
    &mut *s.as_mut_ptr()
}

#[inline]
pub fn allocate<'handle, 'space: 'handle, T: 'static>(
    space: &'handle mut impl SpaceAllocator<'space>,
    size: usize,
) -> Option<&'handle mut Space<T>> {
    let required_padding = Space::<T>::padding_for(space.remaining_bytes());
    let raw = space.allocate(size + required_padding)?;

    Space::try_align_from_bytes_mut(raw)
}

#[inline]
pub fn allocate_values<'handle, 'space: 'handle, T: 'static>(
    space: &'handle mut impl SpaceAllocator<'space>,
    count: usize,
) -> Option<&'handle mut [MaybeUninit<T>]> {
    let space = allocate(space, count * ::core::mem::size_of::<T>())?;
    Some(space.as_uninit_slice_mut())
}

#[inline]
pub fn init_atom<'handle, 'space: 'handle, A: Atom<'handle, 'space>>(
    space: &'handle mut impl SpaceAllocator<'space>,
    atom_type: URID<A>,
    write_parameter: A::WriteParameter,
) -> Option<A::WriteHandle> {
    let space: AtomSpaceWriter<'handle, 'space> = AtomSpaceWriter::write_new(space, atom_type)?;
    A::init(space, write_parameter)
}

#[inline]
pub fn forward_atom<'handle, 'space: 'handle>(
    space: &'handle mut impl SpaceAllocator<'space>,
    atom: &UnidentifiedAtom,
) -> Option<&'handle mut UnidentifiedAtom> {
    let resulting_space = allocate(space, atom.atom_space().len())?;
    resulting_space.as_bytes_mut().copy_from_slice(atom.atom_space().as_bytes());
    // SAFETY: We just wrote those bytes, we know for sure they're the same and aligned
    unsafe { UnidentifiedAtom::from_space_mut(resulting_space) }
}

#[inline]
pub fn write_bytes<'handle, 'space: 'handle>(
    space: &'handle mut impl SpaceAllocator<'space>,
    bytes: &[u8],
) -> Option<&'handle mut [u8]> {
    let space = space.allocate(bytes.len())?;
    space.copy_from_slice(bytes);
    Some(space)
}

#[inline]
pub fn write_value<'handle, 'space: 'handle, T>(
    space: &'handle mut impl SpaceAllocator<'space>,
    value: T,
) -> Option<&'handle mut T>
where
    T: Copy + Sized + 'static,
{
    let space = allocate(space, size_of_val(&value))?;
    // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
    let space = unsafe { space.as_uninit_mut_unchecked() };
    *space = MaybeUninit::new(value);

    // SAFETY: the MaybeUninit has now been properly initialized.
    Some(unsafe { assume_init_mut(space) })
}

pub fn write_values<'handle, 'space: 'handle, T>(
    space: &'handle mut impl SpaceAllocator<'space>,
    values: &[T],
) -> Option<&'handle mut [T]>
where
    T: Copy + Sized + 'static,
{
    let space: &mut Space<T> = allocate(space, size_of_val(values))?;
    let space = space.as_uninit_slice_mut();

    for (dst, src) in space.iter_mut().zip(values.iter()) {
        *dst = MaybeUninit::new(*src)
    }

    // SAFETY: Assume init: we just initialized the memory above
    Some(unsafe { &mut *(space as *mut [_] as *mut [T]) })
}

#[cfg(test)]
mod tests {
    use crate::prelude::{Int, SpaceAllocator};
    use crate::space::cursor::SpaceCursor;
    use crate::space::{init_atom, write_value, AtomSpace};
    use urid::URID;

    const INT_URID: URID<Int> = unsafe { URID::new_unchecked(5) };

    #[test]
    fn test_init_atom_lifetimes() {
        let mut space = AtomSpace::boxed(32);
        assert_eq!(space.as_bytes().as_ptr() as usize % 8, 0); // TODO: move this, this is a test for boxed

        let mut cursor = SpaceCursor::new(space.as_bytes_mut()); // The pointer that is going to be moved as we keep writing.
        let new_value = write_value(&mut cursor, 42u8).unwrap();

        assert_eq!(42, *new_value);
        assert_eq!(31, cursor.remaining_bytes().len());

        {
            init_atom(&mut cursor, INT_URID, 69).unwrap();
            assert_eq!(12, cursor.remaining_bytes().len());
        }

        assert_eq!(
            space.as_bytes(),
            [
                42, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ]
        );
        assert_eq!(32, space.len());
    }
}
