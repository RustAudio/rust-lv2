use crate::space::{AtomSpaceWriter, Space};
use crate::{Atom, UnidentifiedAtom};
use urid::URID;

use core::mem::size_of_val;
use std::mem::MaybeUninit;

// This function is separate to ensure proper lifetimes
unsafe fn assume_init_mut<T>(s: &mut MaybeUninit<T>) -> &mut T {
    // SAFETY: the caller must guarantee that `self` is initialized.
    &mut *s.as_mut_ptr()
}

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
///
// TODO: Find proper name
pub trait SpaceAllocatorImpl<'space> {
    fn allocate_and_split(&mut self, size: usize) -> Option<(&mut [u8], &mut [u8])>;

    #[must_use]
    unsafe fn rewind(&mut self, byte_count: usize) -> bool;

    fn allocated_bytes(&self) -> &[u8];
    fn allocated_bytes_mut(&mut self) -> &mut [u8];

    fn remaining_bytes(&self) -> &[u8];
    fn remaining_bytes_mut(&mut self) -> &mut [u8];
}

// TODO: Find proper name
pub trait SpaceAllocator<'space>: SpaceAllocatorImpl<'space> + Sized {
    /// Try to allocate memory on the internal data slice.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    #[inline]
    fn allocate(&mut self, size: usize) -> Option<&mut [u8]> {
        self.allocate_and_split(size).map(|(_, s)| s)
    }

    #[inline]
    fn allocate_aligned<'handle, T: 'static>(&'handle mut self, size: usize) -> Option<&'handle mut Space<T>>
    where
        'space: 'handle,
    {
        let required_padding = Space::<T>::padding_for(self.remaining_bytes());
        let raw = self.allocate(size + required_padding)?;

        Space::try_align_from_bytes_mut(raw)
    }

    #[inline]
    fn allocate_values<'handle, T: 'static>(
        &'handle mut self,
        count: usize,
    ) -> Option<&'handle mut [MaybeUninit<T>]>
    where
        'space: 'handle,
    {
        let space = self.allocate_aligned(count * std::mem::size_of::<T>())?;
        Some(space.as_uninit_slice_mut())
    }

    #[inline]
    fn init_atom<'handle, A: Atom<'handle, 'space>>(
        &'handle mut self,
        atom_type: URID<A>,
        write_parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle>
    where
        'space: 'handle,
    {
        let space: AtomSpaceWriter<'handle, 'space> = AtomSpaceWriter::write_new(self, atom_type)?;
        A::init(space, write_parameter)
    }

    #[inline]
    fn forward_atom<'handle>(
        &'handle mut self,
        atom: &UnidentifiedAtom,
    ) -> Option<&'handle mut UnidentifiedAtom>
    where
        'space: 'handle,
    {
        let resulting_space = self.allocate_aligned(atom.atom_space().len())?;
        resulting_space
            .as_bytes_mut()
            .copy_from_slice(atom.atom_space().as_bytes());
        // SAFETY: We just wrote those bytes, we know for sure they're the same and aligned
        unsafe { UnidentifiedAtom::from_space_mut(resulting_space) }
    }

    #[inline]
    fn write_bytes<'handle>(&'handle mut self, bytes: &[u8]) -> Option<&'handle mut [u8]>
    where
        'space: 'handle,
    {
        let space = self.allocate(bytes.len())?;
        space.copy_from_slice(bytes);
        Some(space)
    }

    #[inline]
    fn write_value<'handle, T: 'static>(&'handle mut self, value: T) -> Option<&'handle mut T>
    where
        T: Copy + Sized + 'static,
        'space: 'handle,
    {
        let space = self.allocate_aligned(size_of_val(&value))?;
        // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
        let space = unsafe { space.as_uninit_mut_unchecked() };
        *space = MaybeUninit::new(value);

        // SAFETY: the MaybeUninit has now been properly initialized.
        Some(unsafe { assume_init_mut(space) })
    }

    fn write_values<'handle, T>(&'handle mut self, values: &[T]) -> Option<&'handle mut [T]>
    where
        T: Copy + Sized + 'static,
        //'space: 'handle,
    {
        let space: &mut Space<T> = self.allocate_aligned(size_of_val(values))?;
        let space = space.as_uninit_slice_mut();

        for (dst, src) in space.iter_mut().zip(values.iter()) {
            *dst = MaybeUninit::new(*src)
        }

        // SAFETY: Assume init: we just initialized the memory above
        Some(unsafe { &mut *(space as *mut [_] as *mut [T]) })
    }
}

impl<'space, H: SpaceAllocatorImpl<'space>> SpaceAllocator<'space> for H { }

#[cfg(test)]
mod tests {
    use crate::prelude::{Int, SpaceAllocator};
    use crate::space::cursor::SpaceCursor;
    use crate::space::{AtomSpace, SpaceAllocatorImpl};
    use urid::URID;

    const INT_URID: URID<Int> = unsafe { URID::new_unchecked(5) };

    #[test]
    fn test_init_atom_lifetimes() {
        let mut space = AtomSpace::boxed(32);
        assert_eq!(space.as_bytes().as_ptr() as usize % 8, 0); // TODO: move this, this is a test for boxed

        let mut cursor = SpaceCursor::new(space.as_bytes_mut()); // The pointer that is going to be moved as we keep writing.
        let new_value = cursor.write_value(42u8).unwrap();

        assert_eq!(42, *new_value);
        assert_eq!(31, cursor.remaining_bytes().len());

        {
            cursor.init_atom(INT_URID, 69).unwrap();
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
