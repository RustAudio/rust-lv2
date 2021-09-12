use crate::space::{AlignedSpace, AtomSpaceWriter};
use crate::{Atom, AtomHandle, UnidentifiedAtom};
use urid::URID;

use crate::space::error::AtomError;
use core::mem::{size_of, size_of_val, MaybeUninit};

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
///
// TODO: Find proper name
pub trait SpaceAllocatorImpl {
    fn allocate_and_split(&mut self, size: usize) -> Result<(&mut [u8], &mut [u8]), AtomError>;

    #[must_use]
    unsafe fn rewind(&mut self, byte_count: usize) -> bool;

    fn allocated_bytes(&self) -> &[u8];
    fn allocated_bytes_mut(&mut self) -> &mut [u8];

    fn remaining_bytes(&self) -> &[u8];
    fn remaining_bytes_mut(&mut self) -> &mut [u8];
}

// TODO: Find proper name
pub trait SpaceAllocator: SpaceAllocatorImpl + Sized {
    /// Try to allocate memory on the internal data slice.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    #[inline]
    fn allocate(&mut self, size: usize) -> Result<&mut [u8], AtomError> {
        self.allocate_and_split(size).map(|(_, s)| s)
    }

    #[inline]
    fn allocate_padding_for<T: 'static>(&mut self) -> Result<(), AtomError> {
        let required_padding = crate::util::try_padding_for::<T>(self.remaining_bytes())?;
        self.allocate(required_padding)?;

        Ok(())
    }

    #[inline]
    fn allocate_aligned<T: 'static>(
        &mut self,
        size: usize,
    ) -> Result<&mut AlignedSpace<T>, AtomError> {
        let required_padding = crate::util::try_padding_for::<T>(self.remaining_bytes())?;
        let raw = self.allocate(size + required_padding)?;

        AlignedSpace::align_from_bytes_mut(raw).ok_or_else(|| AtomError::Unknown)
    }

    #[inline]
    fn allocate_value<T: 'static>(&mut self) -> Result<&mut MaybeUninit<T>, AtomError> {
        let space = self.allocate_aligned(size_of::<MaybeUninit<T>>())?;
        // SAFETY: We used size_of, so we are sure that the allocated space is exactly big enough for T.
        Ok(unsafe { space.as_uninit_mut_unchecked() })
    }

    #[inline]
    fn allocate_values<T: 'static>(
        &mut self,
        count: usize,
    ) -> Result<&mut [MaybeUninit<T>], AtomError> {
        let space = self.allocate_aligned(count * std::mem::size_of::<T>())?;
        Ok(space.as_uninit_slice_mut())
    }

    #[inline]
    fn init_atom<A: Atom>(
        &mut self,
        atom_type: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle>::Handle, AtomError> {
        let space = AtomSpaceWriter::write_new(self, atom_type)?;
        A::init(space)
    }

    #[inline]
    fn forward_atom(
        &mut self,
        atom: &UnidentifiedAtom,
    ) -> Result<&mut UnidentifiedAtom, AtomError> {
        let resulting_space = self.allocate_aligned(atom.atom_space().len())?;
        resulting_space
            .as_bytes_mut()
            .copy_from_slice(atom.atom_space().as_bytes());

        // SAFETY: We just wrote those bytes, we know for sure they're the same and aligned
        unsafe { UnidentifiedAtom::from_space_mut(resulting_space) }
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<&mut [u8], AtomError> {
        let space = self.allocate(bytes.len())?;
        space.copy_from_slice(bytes);
        Ok(space)
    }

    #[inline]
    fn write_value<T: 'static>(&mut self, value: T) -> Result<&mut T, AtomError>
    where
        T: Copy + Sized + 'static,
    {
        let space = self.allocate_aligned(size_of_val(&value))?;
        // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
        let space = unsafe { space.as_uninit_mut_unchecked() };

        Ok(crate::util::write_uninit(space, value))
    }

    fn write_values<T>(&mut self, values: &[T]) -> Result<&mut [T], AtomError>
    where
        T: Copy + Sized + 'static,
    {
        let space: &mut AlignedSpace<T> = self.allocate_aligned(size_of_val(values))?;
        let space = space.as_uninit_slice_mut();

        for (dst, src) in space.iter_mut().zip(values.iter()) {
            *dst = MaybeUninit::new(*src)
        }

        // SAFETY: Assume init: we just initialized the memory above
        Ok(unsafe { &mut *(space as *mut [_] as *mut [T]) })
    }
}

impl<H: SpaceAllocatorImpl> SpaceAllocator for H {}

#[cfg(test)]
mod tests {
    use crate::prelude::{Int, SpaceAllocator};
    use crate::space::cursor::SpaceCursor;
    use crate::space::{SpaceAllocatorImpl, VecSpace};
    use crate::AtomHeader;
    use urid::URID;

    // SAFETY: this is just for testing, values aren't actually read using this URID.
    const INT_URID: URID<Int> = unsafe { URID::new_unchecked(5) };

    #[test]
    fn test_init_atom_lifetimes() {
        let mut space = VecSpace::<AtomHeader>::new_with_capacity(4);
        assert_eq!(space.as_bytes().as_ptr() as usize % 8, 0); // TODO: move this, this is a test for boxed

        let mut cursor = SpaceCursor::new(space.as_bytes_mut()); // The pointer that is going to be moved as we keep writing.
        let new_value = cursor.write_value(42u8).unwrap();

        assert_eq!(42, *new_value);
        assert_eq!(31, cursor.remaining_bytes().len());

        {
            cursor.init_atom(INT_URID).unwrap().set(69);
            assert_eq!(8, cursor.remaining_bytes().len());
        }

        /*assert_eq!(
            space.as_bytes(),
            [
                // FIXME: this test is endianness-sensitive
                42, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 5, 0, 0, 0, 69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ]
        );*/
        assert_eq!(32, space.as_bytes().len());
    }
}
