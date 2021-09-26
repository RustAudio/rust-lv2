use crate::space::{AlignedSpace, AtomSpaceWriter};
use crate::{Atom, AtomHandle, UnidentifiedAtom};
use urid::URID;

use crate::space::error::AtomWriteError;
use crate::space::terminated::Terminated;
use core::mem::{size_of, size_of_val, MaybeUninit};

/// The result of a [`SpaceWriter`](SpaceWriterImpl) allocation.
///
/// This structure allows simultaneous access to both the newly allocated slice, and all previously
/// allocated bytes.
pub struct SpaceWriterSplitAllocation<'a> {
    pub previous: &'a mut [u8],
    pub allocated: &'a mut [u8],
}

/// An object-safe trait to allocate bytes from a contiguous buffer to write Atom data into.
///
/// Implementors of this trait act like a sort of cursor, continuously
///
/// This trait is very bare-bones, in order to be trait-object-safe. As an user, you probably want
/// to use the [`SpaceWriter`] trait, a child trait with many more utilities available, and with a
/// blanket implementation for all types that implement [`SpaceWriterImpl`].
///
/// The term "allocate" is used very loosely here, as even a simple cursor over a mutable byte
/// buffer (e.g. [`SpaceCursor`](crate::space::SpaceCursor)) can "allocate" bytes using this trait.
///
/// This trait is useful to abstract over many types of buffers, including ones than can track the
/// amount of allocated bytes into an atom header (i.e. [`AtomSpaceWriter`]).
pub trait SpaceWriterImpl {
    /// Allocates a new byte buffer of the requested size. A mutable reference to both the newly
    /// allocated slice and all previously allocated bytes is returned (through [`SpaceWriterSplitAllocation`]),
    /// allowing some implementations to update previous data as well.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate..
    ///
    /// # Panics
    ///
    /// This function may panic if the given size, added to the length of the total allocated bytes,
    /// overflows an [`usize`].
    fn allocate_and_split(
        &mut self,
        size: usize,
    ) -> Result<SpaceWriterSplitAllocation, AtomWriteError>;

    /// Rewinds the writer by a given amount of bytes, allowing to overwrite previously allocated
    /// bytes.
    ///
    /// # Errors
    ///
    /// This method may return an error if `byte_count` is greater than the amount of all already
    /// allocated bytes.
    ///
    /// # Safety
    ///
    /// Rewinding may allow other atoms to be overwritten, and thus completely invalidate their
    /// contents and internal structure. The caller is responsible to ensure that the exposed data
    /// is safe to be overwritten.
    unsafe fn rewind(&mut self, byte_count: usize) -> Result<(), AtomWriteError>;

    /// Returns a slice pointing to the previously allocated bytes.
    fn allocated_bytes(&self) -> &[u8];

    /// Returns a mutable slice pointing to the previously allocated bytes.
    ///
    /// # Safety
    ///
    /// Accessing allocated bytes may allow other atoms to be overwritten, and thus completely
    /// invalidate their contents and internal structure. The caller is responsible to ensure that
    /// the exposed data is safe to be overwritten.
    unsafe fn allocated_bytes_mut(&mut self) -> &mut [u8];

    /// Returns a slice pointing to the remaining, uninitialized bytes.
    fn remaining_bytes(&self) -> &[u8];
}

pub trait SpaceWriter: SpaceWriterImpl + Sized {
    /// Try to allocate memory on the internal data slice.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    #[inline]
    fn allocate(&mut self, size: usize) -> Result<&mut [u8], AtomWriteError> {
        let allocated = self.allocate_and_split(size)?;
        assert_eq!(allocated.allocated.len(), size);
        Ok(allocated.allocated)
    }

    #[inline]
    fn allocate_padding_for<T: 'static>(&mut self) -> Result<(), AtomWriteError> {
        let required_padding = crate::util::try_padding_for::<T>(self.remaining_bytes())?;
        self.allocate(required_padding)?;

        Ok(())
    }

    #[inline]
    fn allocate_aligned<T: 'static>(
        &mut self,
        size: usize,
    ) -> Result<&mut AlignedSpace<T>, AtomWriteError> {
        let required_padding = crate::util::try_padding_for::<T>(self.remaining_bytes())?;
        let raw = self.allocate(size + required_padding)?;

        Ok(AlignedSpace::align_from_bytes_mut(raw)?)
    }

    #[inline]
    fn allocate_value<T: 'static>(&mut self) -> Result<&mut MaybeUninit<T>, AtomWriteError> {
        let space = self.allocate_aligned(size_of::<MaybeUninit<T>>())?;
        // SAFETY: We used size_of, so we are sure that the allocated space is exactly big enough for T.
        Ok(unsafe { space.as_uninit_slice_mut().get_unchecked_mut(0) })
    }

    #[inline]
    fn allocate_values<T: 'static>(
        &mut self,
        count: usize,
    ) -> Result<&mut [MaybeUninit<T>], AtomWriteError> {
        let space = self.allocate_aligned(count * std::mem::size_of::<T>())?;
        Ok(space.as_uninit_slice_mut())
    }

    #[inline]
    fn init_atom<A: Atom>(
        &mut self,
        atom_type: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        let space = AtomSpaceWriter::write_new(self, atom_type)?;
        A::write(space)
    }

    #[inline]
    fn forward_atom(
        &mut self,
        atom: &UnidentifiedAtom,
    ) -> Result<&mut UnidentifiedAtom, AtomWriteError> {
        let resulting_space = self.allocate_aligned(atom.atom_space().bytes_len())?;
        resulting_space
            .as_bytes_mut()
            .copy_from_slice(atom.atom_space().as_bytes());

        // SAFETY: We just wrote those bytes, we know for sure they're the same and aligned
        unsafe { UnidentifiedAtom::from_space_mut(resulting_space) }
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<&mut [u8], AtomWriteError> {
        let space = self.allocate(bytes.len())?;
        space.copy_from_slice(bytes);
        Ok(space)
    }

    #[inline]
    fn write_value<T: 'static>(&mut self, value: T) -> Result<&mut T, AtomWriteError>
    where
        T: Copy + Sized + 'static,
    {
        let space = self.allocate_aligned(size_of_val(&value))?;
        // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
        let space = unsafe { space.as_uninit_slice_mut().get_unchecked_mut(0) };

        Ok(crate::util::write_uninit(space, value))
    }

    fn write_values<T>(&mut self, values: &[T]) -> Result<&mut [T], AtomWriteError>
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

    #[inline]
    fn terminated(self, terminator: u8) -> Terminated<Self> {
        Terminated::new(self, terminator)
    }
}

impl<H> SpaceWriter for H where H: SpaceWriterImpl {}

#[cfg(test)]
mod tests {
    use crate::atom_prelude::*;
    use crate::prelude::*;
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
            cursor.init_atom(INT_URID).unwrap().set(69).unwrap();
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
