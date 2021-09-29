use crate::space::{AlignedSpace, AtomWriter};
use crate::{Atom, AtomHandle, UnidentifiedAtom};
use urid::URID;

use crate::space::error::AtomWriteError;
use crate::space::terminated::Terminated;
use core::mem::{size_of, size_of_val, MaybeUninit};

/// The result of a [`SpaceAllocator`](SpaceAllocator) allocation.
///
/// This structure allows simultaneous access to both the newly allocated slice, and all previously
/// allocated bytes.
pub struct SpaceWriterSplitAllocation<'a> {
    pub previous: &'a mut [u8],
    pub allocated: &'a mut [u8],
}

/// An object-safe trait to allocate bytes from a contiguous buffer to write Atom data into.
///
/// Implementors of this trait act like a sort of cursor, each allocation being contiguous with the
/// previous one.
///
/// This trait is very bare-bones, in order to be trait-object-safe. As an user, you probably want
/// to use the [`SpaceWriter`] trait, a child trait with many more utilities available, and with a
/// blanket implementation for all types that implement [`SpaceAllocator`].
///
/// The term "allocate" is used very loosely here, as even a simple cursor over a mutable byte
/// buffer (e.g. [`SpaceCursor`](crate::space::SpaceCursor)) can "allocate" bytes using this trait.
///
/// This trait is useful to abstract over many types of buffers, including ones than can track the
/// amount of allocated bytes into an atom header (i.e. [`AtomSpaceWriter`]).
pub trait SpaceAllocator {
    /// Allocates a new byte buffer of the requested size. A mutable reference to both the newly
    /// allocated slice and all previously allocated bytes is returned (through [`SpaceWriterSplitAllocation`]),
    /// allowing some implementations to update previous data as well.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer.
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

pub trait SpaceWriter: SpaceAllocator + Sized {
    /// Allocates and returns a new mutable byte buffer of the requested size, in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use lv2_atom::atom_prelude::*;
    ///
    /// let mut buffer = vec![0; 64];
    /// let mut writer = SpaceCursor::new(&mut buffer);
    ///
    /// let allocated = writer.allocate(5).unwrap();
    /// assert_eq!(allocated.len(), 5);
    /// ```
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer.
    #[inline]
    fn allocate(&mut self, size: usize) -> Result<&mut [u8], AtomWriteError> {
        let allocated = self.allocate_and_split(size)?;
        assert_eq!(allocated.allocated.len(), size);
        Ok(allocated.allocated)
    }

    /// Allocates and returns a new aligned mutable byte buffer of the requested size, in bytes.
    ///
    /// The resulting buffer is guaranteed to be aligned to the alignment requirements of the given
    /// type `T`, meaning a value of type `T` can be safely written into it directly.
    ///
    /// # Example
    ///
    /// ```
    /// use lv2_atom::atom_prelude::*;
    ///
    /// let mut buffer = vec![0; 64];
    /// let mut writer = SpaceCursor::new(&mut buffer);
    ///
    /// let allocated = writer.allocate_aligned(5).unwrap();
    /// assert_eq!(allocated.len(), 5);
    /// ```
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    #[inline]
    fn allocate_aligned<T: 'static>(
        &mut self,
        byte_size: usize,
    ) -> Result<&mut AlignedSpace<T>, AtomWriteError> {
        let required_padding = crate::util::try_padding_for::<T>(self.remaining_bytes())?;
        let raw = self.allocate(byte_size + required_padding)?;

        Ok(AlignedSpace::align_from_bytes_mut(raw)?)
    }

    /// Allocates room in the byte buffer for a single value of type `T`.
    ///
    /// A mutable reference to the allocated buffer is returned as a
    /// [`MaybeUninit`](core::mem::maybe_uninit::MaybeUninit),
    /// as the resulting memory buffer is most likely going to be uninitialized.
    ///
    /// # Example
    ///
    /// ```
    /// use lv2_atom::atom_prelude::*;
    /// use std::mem::MaybeUninit;
    ///
    /// let mut buffer = vec![0; 64];
    /// let mut writer = SpaceCursor::new(&mut buffer);
    ///
    /// let allocated = writer.allocate_value().unwrap();
    /// *allocated = MaybeUninit::new(42u32);
    /// assert_eq!(unsafe { allocated.assume_init() }, 42);
    /// ```
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    #[inline]
    fn allocate_value<T: 'static>(&mut self) -> Result<&mut MaybeUninit<T>, AtomWriteError> {
        let space = self.allocate_aligned(size_of::<MaybeUninit<T>>())?;
        // SAFETY: We used size_of, so we are sure that the allocated space is exactly big enough for T.
        Ok(unsafe { space.as_uninit_slice_mut().get_unchecked_mut(0) })
    }

    /// Allocates room in the byte buffer for multiple values of type `T`.
    ///
    /// A mutable reference to the allocated buffer is returned as a slice of
    /// [`MaybeUninit`](core::mem::maybe_uninit::MaybeUninit)s,
    /// as the resulting memory buffer is most likely going to be uninitialized.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    #[inline]
    fn allocate_values<T: 'static>(
        &mut self,
        count: usize,
    ) -> Result<&mut [MaybeUninit<T>], AtomWriteError> {
        let space = self.allocate_aligned(count * std::mem::size_of::<T>())?;
        Ok(space.as_uninit_slice_mut())
    }

    /// Writes an atom of a given type into the buffer.
    ///
    /// This method only initializes the new Atom header with the given type, tracking its
    /// size, and returns the [writer](crate::Atom::write) associated to the given Atom type.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    #[inline]
    fn write_atom<A: Atom>(
        &mut self,
        atom_type: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        let space = AtomWriter::write_new(self, atom_type)?;
        A::write(space)
    }

    /// Copies an already fully initialized atom of any type into the buffer.
    ///
    /// This method will simply copy the atom's bytes into the buffer, unchanged, and unaware of its
    /// internal representation.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    #[inline]
    fn copy_atom(
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

    /// Writes the given bytes into the buffer.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer.
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<&mut [u8], AtomWriteError> {
        let space = self.allocate(bytes.len())?;
        space.copy_from_slice(bytes);
        Ok(space)
    }

    /// Writes the given value into the buffer.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
    #[inline]
    fn write_value<T>(&mut self, value: T) -> Result<&mut T, AtomWriteError>
    where
        T: Copy + Sized + 'static,
    {
        let space = self.allocate_aligned(size_of_val(&value))?;
        // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
        let space = unsafe { space.as_uninit_slice_mut().get_unchecked_mut(0) };

        Ok(crate::util::write_uninit(space, value))
    }

    /// Writes the given values into the buffer.
    ///
    /// # Errors
    ///
    /// This method may return an error if the writer ran out of space in its internal buffer, and
    /// is unable to reallocate the buffer, or if the padding and/or alignment requirements couldn't
    /// be met.
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

    /// Makes all further operations from this writer write a given terminator byte.
    ///
    /// This method is a simple helper for [`Terminated::new`](Terminated::new)
    ///
    /// See the documentation for [`Terminated`](Terminated) for more information.
    #[inline]
    fn terminated(self, terminator: u8) -> Terminated<Self> {
        Terminated::new(self, terminator)
    }
}

impl<H> SpaceWriter for H where H: SpaceAllocator {}

#[cfg(test)]
mod tests {
    use crate::atom_prelude::*;
    use crate::prelude::*;
    use urid::URID;

    #[test]
    fn test_write_value() {
        let mut space = vec![0; 32];

        let mut cursor = SpaceCursor::new(&mut space);
        let new_value = cursor.write_value(42u8).unwrap();

        assert_eq!(42, *new_value);
        assert_eq!(31, cursor.remaining_bytes().len());
    }

    // SAFETY: this is just for testing, values aren't actually read using this URID.
    const INT_URID: URID<Int> = unsafe { URID::new_unchecked(5) };

    #[test]
    fn test_write_atom() {
        let mut space = vec![0; 32];

        let mut cursor = SpaceCursor::new(&mut space);

        {
            cursor.write_atom(INT_URID).unwrap().set(69).unwrap();
            assert_eq!(16, cursor.remaining_bytes().len());
        }

        assert_eq!(space[0..4], 8u32.to_ne_bytes());
        assert_eq!(space[4..8], 5u32.to_ne_bytes());
        assert_eq!(space[8..12], 69u32.to_ne_bytes());
    }
}
