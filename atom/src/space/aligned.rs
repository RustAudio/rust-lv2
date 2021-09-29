use crate::header::AtomHeader;
use crate::space::error::{AlignmentError, AlignmentErrorInner, TypeData};
use crate::space::SpaceCursor;
use crate::space::SpaceReader;
use core::mem::{align_of, size_of};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{size_of_val, MaybeUninit};
use std::slice::{from_raw_parts, from_raw_parts_mut};

/// An slice of bytes with the alignment of a type `T`.
///
/// This type is a simple byte slice that guarantees its start is properly aligned for containing a type `T`.
/// This buffer can be split and realigned, effectively allowing to read a stream of arbitrary types
/// from a byte buffer, such as [`Atom`s](crate::Atom).
///
/// Any operation that may lead to a misaligned `AlignedSpace` is considered unsafe.
///
/// Note that only the start of the slice is aligned, not the end. This allows having a buffer that
/// is bigger than a multiple of `T`'s alignment.
///
/// Although they are aligned, `AlignedSpace`s do not consider their contents to be initialized. Therefore,
/// the only safe reading operations will return `MaybeUninit<T>`. Unsafe helper methods that assume
/// the contents are initialized are also provided, for convenience.
///
/// # Example
///
/// The following example reads `u64`s from an aligned byte slice.
///
/// ```
/// # use lv2_atom::space::AlignedSpace;
/// let values = &[42u64, 69];
/// // Transmuting to a slice of bytes.
/// let bytes: &[u8] = unsafe { values.align_to().1 };
///
/// // --- Imagine those bytes are sent over and then received from an external buffer
///
/// // Bytes are already aligned, the whole slice will be available
/// let space: &AlignedSpace<u64> = AlignedSpace::align_from_bytes(bytes).unwrap();
/// // SAFETY: we know the slice was initialized with proper u64 values.
/// let read_values = unsafe { space.assume_init_slice() };
/// assert_eq!(read_values, [42u64, 69]);
/// ```
///
#[derive(Eq, PartialEq)]
#[repr(transparent)]
pub struct AlignedSpace<T> {
    _type: PhantomData<T>,
    data: [u8],
}

pub type AtomSpace = AlignedSpace<AtomHeader>;

impl<T: 'static> AlignedSpace<T> {
    /// Creates a new space from a slice of bytes.
    ///
    /// # Errors
    ///
    /// This method returns an [`AlignmentError`] if the given slice is not aligned
    /// (i.e. if it's pointer's value is not a multiple of `align_of::<T>()` bytes).
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::AlignedSpace;
    /// let values = &[42u64, 69];
    /// // Transmuting to a slice of bytes
    /// let bytes: &[u8] = unsafe { values.align_to().1 };
    ///
    /// assert!(AlignedSpace::<u64>::from_bytes(bytes).is_ok());
    /// assert!(AlignedSpace::<u64>::from_bytes(&bytes[1..]).is_err());
    /// ```
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<&Self, AlignmentError> {
        check_alignment::<T>(data)?;

        // SAFETY: We just checked above that the pointer is correctly aligned
        Ok(unsafe { AlignedSpace::from_bytes_unchecked(data) })
    }

    /// Creates a new mutable space from a mutable slice of bytes.
    ///
    /// # Errors
    ///
    /// This method returns an [`AlignmentError`] if the given slice is not aligned
    /// (i.e. if it's pointer's value is not a multiple of `align_of::<T>()` bytes).
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::AlignedSpace;
    /// let values = &mut [42u64, 69];
    /// // Transmuting to a slice of bytes
    /// let bytes: &mut [u8] = unsafe { values.align_to_mut().1 };
    ///
    /// assert!(AlignedSpace::<u64>::from_bytes_mut(bytes).is_ok());
    /// assert!(AlignedSpace::<u64>::from_bytes_mut(&mut bytes[1..]).is_err());
    /// ```
    #[inline]
    pub fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, AlignmentError> {
        check_alignment::<T>(data)?;

        // SAFETY: We just checked above that the pointer is correctly aligned
        Ok(unsafe { AlignedSpace::from_bytes_mut_unchecked(data) })
    }

    /// Creates a new space from a slice of bytes, slicing some bytes off its start if necessary.
    ///
    /// # Errors
    ///
    /// This method returns an [`AlignmentError`] if the given slice's is too small to contain
    /// aligned bytes (e.g. if it's smaller than `align_of::<T>()` bytes).
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::{AlignedSpace, error::AtomReadError};
    /// let values = &[42u64, 69];
    /// // Transmuting to a slice of bytes
    /// let bytes: &[u8] = unsafe { values.align_to().1 };
    ///
    /// // The slice has space for both values
    /// assert_eq!(AlignedSpace::<u64>::align_from_bytes(bytes).unwrap().values_len(), 2);
    /// // The slice now only has space for a single value
    /// assert_eq!(AlignedSpace::<u64>::align_from_bytes(&bytes[1..]).unwrap().values_len(), 1);
    /// // The slice doesn't have space for any value anymore
    /// assert_eq!(AlignedSpace::<u64>::align_from_bytes(&bytes[9..]).unwrap().values_len(), 0);
    /// // The slice cannot be aligned
    /// assert!(AlignedSpace::<u64>::align_from_bytes(&bytes[10..11]).is_err());
    /// ```
    #[inline]
    pub fn align_from_bytes(data: &[u8]) -> Result<&Self, AlignmentError> {
        let padding = crate::util::try_padding_for::<T>(data)?;
        let data_len = data.len();

        let data = data.get(padding..).ok_or_else(|| {
            AlignmentError(AlignmentErrorInner::NotEnoughSpaceToRealign {
                ptr: data.as_ptr(),
                available_size: data_len,
                required_padding: padding + 1,
                type_id: TypeData::of::<T>(),
            })
        })?;

        // SAFETY: We just aligned the slice start
        Ok(unsafe { AlignedSpace::from_bytes_unchecked(data) })
    }

    /// Creates a new mutable space from a mutable slice of bytes, slicing some bytes off its start if necessary.
    ///
    /// # Errors
    ///
    /// This method returns an [`AlignmentError`] if the given slice's is too small to contain
    /// aligned bytes (e.g. if no byte in it is properly aligned).
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::{AlignedSpace, error::AtomWriteError};
    /// let values = &mut [42u64, 69];
    /// // Transmuting to a slice of bytes
    /// let bytes: &mut [u8] = unsafe { values.align_to_mut().1 };
    ///
    /// // The slice has space for both values
    /// assert_eq!(AlignedSpace::<u64>::align_from_bytes_mut(bytes).unwrap().values_len(), 2);
    /// // The slice now only has space for a single value
    /// assert_eq!(AlignedSpace::<u64>::align_from_bytes_mut(&mut bytes[1..]).unwrap().values_len(), 1);
    /// // The slice doesn't have space for any value anymore
    /// assert_eq!(AlignedSpace::<u64>::align_from_bytes_mut(&mut bytes[9..]).unwrap().values_len(), 0);
    /// // The slice cannot be aligned
    /// assert!(AlignedSpace::<u64>::align_from_bytes_mut(&mut bytes[10..11]).is_err());
    /// ```
    #[inline]
    pub fn align_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, AlignmentError> {
        let padding = crate::util::try_padding_for::<T>(data)?;
        let data_len = data.len();
        let data_ptr = data.as_ptr();

        let data = data.get_mut(padding..).ok_or_else(|| {
            AlignmentError(AlignmentErrorInner::NotEnoughSpaceToRealign {
                ptr: data_ptr,
                available_size: data_len,
                required_padding: padding + 1,
                type_id: TypeData::of::<T>(),
            })
        })?;

        // SAFETY: We just aligned the slice start
        Ok(unsafe { AlignedSpace::from_bytes_mut_unchecked(data) })
    }

    /// Creates a new space from a slice of bytes, without checking for padding correctness.
    ///
    /// # Safety
    ///
    /// The caller of this method is responsible for ensuring that the slice's contents are correctly aligned.
    /// Calling this method with an unaligned slice will result in Undefined Behavior.
    ///
    /// For a safe, checked version, see [`Space::from_bytes`](Space::from_bytes).
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::{AlignedSpace, error::AtomWriteError};
    /// let values = &[42u64, 69];
    /// // Transmuting to a slice of bytes
    /// let bytes: &[u8] = unsafe { values.align_to().1 };
    ///
    /// assert_eq!(unsafe { AlignedSpace::<u64>::from_bytes_unchecked(bytes) }.values_len(), 2);
    /// ```
    // NOTE: This method will always be used internally instead of the constructor, to make sure that
    // the unsafety is explicit and accounted for.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(data: &[u8]) -> &AlignedSpace<T> {
        // SAFETY: It is safe to transmute, since our type has repr(transparent) with [u8].
        // SAFETY: The caller is responsible to check for slice alignment.
        &*(data as *const _ as *const Self)
    }

    /// Creates a new mutable space from a slice of bytes, without checking for padding correctness.
    ///
    /// # Safety
    ///
    /// The caller of this method is responsible for ensuring that the slice's contents are correctly aligned.
    /// Otherwise, reads will be performed unaligned, which are either slow, a CPU crash, or UB depending on platforms.
    ///
    /// For a safe, checked version, see [`Space::from_bytes_mut`](Space::from_bytes_mut).
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::{AlignedSpace, error::AtomWriteError};
    /// let values = &mut [42u64, 69];
    /// // Transmuting to a slice of bytes
    /// let bytes: &mut [u8] = unsafe { values.align_to_mut().1 };
    ///
    /// assert_eq!(unsafe { AlignedSpace::<u64>::from_bytes_mut_unchecked(bytes) }.values_len(), 2);
    // NOTE: This method will always be used internally instead of the constructor, to make sure that
    // the unsafety is explicit and accounted for.
    #[inline(always)]
    pub unsafe fn from_bytes_mut_unchecked(data: &mut [u8]) -> &mut AlignedSpace<T> {
        // SAFETY: It is safe to transmute, since our type has repr(transparent) with [u8].
        // SAFETY: The caller is responsible to check for slice alignment.
        &mut *(data as *mut _ as *mut Self)
    }

    /// Creates a new space from an already aligned slice of T values.
    ///
    /// The slice type guarantees alignment, therefore this operation is infallible.
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::{AlignedSpace, error::AtomWriteError};
    /// let values = &[42u64, 69];
    ///
    /// let space: &AlignedSpace<u64> = AlignedSpace::from_slice(values);
    /// assert_eq!(space.values_len(), 2);
    /// assert_eq!(space.bytes_len(), 2 * ::core::mem::size_of::<u64>());
    /// ```
    #[inline]
    pub fn from_slice(slice: &[T]) -> &Self {
        // SAFETY: reinterpreting as raw bytes is safe for any value
        let bytes = unsafe { from_raw_parts(slice.as_ptr() as *const u8, size_of_val(slice)) };
        // SAFETY: The pointer is a slice of T, therefore it is already correctly aligned
        unsafe { Self::from_bytes_unchecked(bytes) }
    }

    /// Creates a new mutable space from an already aligned slice of T values.
    ///
    /// The slice type guarantees alignment, therefore this operation is infallible.
    ///
    /// # Example
    ///
    /// ```
    /// # use lv2_atom::space::{AlignedSpace, error::AtomWriteError};
    /// let values = &[42u64, 69];
    ///
    /// let space: &AlignedSpace<u64> = AlignedSpace::from_slice(values);
    /// assert_eq!(space.values_len(), 2);
    /// assert_eq!(space.bytes_len(), 2 * ::core::mem::size_of::<u64>());
    #[inline]
    pub fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        // SAFETY: reinterpreting as raw bytes is safe for any value
        let bytes =
            unsafe { from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, size_of_val(slice)) };
        // SAFETY: The pointer is a slice of T, therefore it is already correctly aligned
        unsafe { Self::from_bytes_mut_unchecked(bytes) }
    }

    /// Creates a new space from an already aligned, potentially uninitialized slice of T.
    #[inline]
    pub(crate) fn from_uninit_slice(slice: &[MaybeUninit<T>]) -> &Self {
        // SAFETY: reinterpreting as raw bytes is safe for any value
        let bytes = unsafe { from_raw_parts(slice.as_ptr() as *const u8, size_of_val(slice)) };
        // SAFETY: The pointer is a slice of T, therefore it is already correctly aligned
        unsafe { Self::from_bytes_unchecked(bytes) }
    }

    /// Creates a new space from an already aligned, potentially uninitialized slice of T.
    #[inline]
    pub(crate) fn from_uninit_slice_mut(slice: &mut [MaybeUninit<T>]) -> &mut Self {
        // SAFETY: reinterpreting as raw bytes is safe for any value
        let bytes =
            unsafe { from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, size_of_val(slice)) };
        // SAFETY: The pointer is a slice of T, therefore it is already correctly aligned
        unsafe { Self::from_bytes_mut_unchecked(bytes) }
    }

    /// A checked version of slice::split_at, which returns the first part as an already-aligned slice.
    #[inline]
    pub fn split_at(&self, mid: usize) -> Option<(&Self, &[u8])> {
        if mid > self.data.len() {
            return None;
        }

        let (start, end) = self.data.split_at(mid);
        // SAFETY: Because this data was the start of an existing Space, it was aligned already.
        let start = unsafe { Self::from_bytes_unchecked(start) };

        Some((start, end))
    }

    /// A checked version of slice::split_at, which returns the first part as an already-aligned slice.
    #[inline]
    pub fn split_at_mut(&mut self, mid: usize) -> Option<(&mut Self, &mut [u8])> {
        if mid > self.data.len() {
            return None;
        }

        let (start, end) = self.data.split_at_mut(mid);
        // SAFETY: Because this data was the start of an existing Space, it was aligned already.
        let start = unsafe { Self::from_bytes_mut_unchecked(start) };

        Some((start, end))
    }

    /// Return the internal slice of the space.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns the internal slice of the space.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns the total length of the space, in bytes.
    #[inline]
    pub fn bytes_len(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of values that can fit in the space.
    ///
    /// If `T` is a zero-sized type, this returns [`usize::MAX`](usize::MAX).
    #[inline]
    pub fn values_len(&self) -> usize {
        self.data
            .len()
            .checked_div(size_of::<T>())
            .unwrap_or(usize::MAX)
    }

    /// Gets the contents as a slice of potentially uninitialized `T`s.
    ///
    /// The resulting slice contains as many values as can fit in the original space.
    /// This means there might be less bytes in this slice than in this space, or zero if the space is too small for a single value.
    #[inline]
    pub fn as_uninit_slice(&self) -> &[MaybeUninit<T>] {
        // SAFETY: This type ensures alignment, so casting aligned bytes to uninitialized memory is safe.
        unsafe {
            ::core::slice::from_raw_parts(
                self.data.as_ptr() as *const MaybeUninit<T>,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    /// Gets the contents as a slice of potentially uninitialized `T`s.
    ///
    /// The resulting slice contains as many values as can fit in the original space.
    /// This means there might be less bytes in this slice than in this space, or zero if the space is too small for a single value.
    #[inline]
    pub fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        // SAFETY: This type ensures alignment, so casting aligned bytes to uninitialized memory is safe.
        unsafe {
            ::core::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut MaybeUninit<T>,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    /// Gets the contents as a slice of `T`s that are all assumed to be initialized.
    ///
    /// # Safety
    ///
    /// Calling this when the space's content is not yet fully initialized causes undefined behavior.
    /// It is up to the caller to guarantee that the underlying buffer really is in an initialized state.
    #[inline]
    pub unsafe fn assume_init_slice(&self) -> &[T] {
        crate::util::assume_init_slice(self.as_uninit_slice())
    }

    /// Gets the contents as a mutable slice of `T`s that are all assumed to be initialized.
    ///
    /// # Safety
    ///
    /// Calling this when the space's content is not yet fully initialized causes undefined behavior.
    /// It is up to the caller to guarantee that the underlying buffer really is in an initialized state.
    #[inline]
    pub unsafe fn assume_init_slice_mut(&mut self) -> &mut [T] {
        crate::util::assume_init_slice_mut(self.as_uninit_slice_mut())
    }

    /// An helper method that creates a new [`SpaceReader`] from the space's contents.
    #[inline]
    pub fn read(&self) -> SpaceReader {
        SpaceReader::new(self.as_bytes())
    }

    /// An helper method that creates a new [`Cursor`] from the mutable space's contents.
    #[inline]
    pub fn write(&mut self) -> SpaceCursor {
        SpaceCursor::new(self.as_bytes_mut())
    }
}

impl<T> Debug for AlignedSpace<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.data, f)
    }
}

#[inline]
fn check_alignment<T: 'static>(data: &[u8]) -> Result<(), AlignmentError> {
    if data.as_ptr() as usize % align_of::<T>() != 0 {
        return Err(AlignmentError(AlignmentErrorInner::UnalignedBuffer {
            type_id: TypeData::of::<T>(),
            ptr: data.as_ptr(),
        }));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::space::error::{AlignmentError, AlignmentErrorInner, TypeData};
    use crate::space::*;
    use crate::AtomHeader;
    use std::mem::{size_of, size_of_val};
    use urid::*;

    #[test]
    fn from_bytes() {
        let values = &mut [42u64, 69];
        let bytes = unsafe { values.align_to_mut().1 };

        assert_eq!(
            AlignedSpace::<u64>::from_bytes(bytes).unwrap().bytes_len(),
            ::core::mem::size_of::<u64>() * 2
        );
        assert_eq!(
            AlignedSpace::<u64>::from_bytes_mut(bytes)
                .unwrap()
                .bytes_len(),
            ::core::mem::size_of::<u64>() * 2
        );

        assert_eq!(
            unsafe { AlignedSpace::<u64>::from_bytes_unchecked(bytes) }.bytes_len(),
            ::core::mem::size_of::<u64>() * 2
        );

        assert_eq!(
            unsafe { AlignedSpace::<u64>::from_bytes_mut_unchecked(bytes) }.bytes_len(),
            ::core::mem::size_of::<u64>() * 2
        );

        assert_eq!(
            AlignedSpace::<u64>::from_bytes(&bytes[1..]),
            Err(AlignmentError(AlignmentErrorInner::UnalignedBuffer {
                type_id: TypeData::of::<u64>(),
                ptr: bytes[1..].as_ptr()
            }))
        );

        let ptr = bytes[1..].as_ptr();
        assert_eq!(
            AlignedSpace::<u64>::from_bytes_mut(&mut bytes[1..]),
            Err(AlignmentError(AlignmentErrorInner::UnalignedBuffer {
                type_id: TypeData::of::<u64>(),
                ptr
            }))
        );
    }

    #[test]
    fn align_from_bytes() {
        let values = &mut [42u64, 69];
        let bytes = unsafe { values.align_to_mut().1 };

        let size = ::core::mem::size_of::<u64>();
        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes(bytes)
                .unwrap()
                .bytes_len(),
            size * 2
        );

        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes_mut(bytes)
                .unwrap()
                .bytes_len(),
            size * 2
        );

        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes(&bytes[1..])
                .unwrap()
                .bytes_len(),
            size
        );

        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes_mut(&mut bytes[1..])
                .unwrap()
                .bytes_len(),
            size
        );

        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes(&bytes[9..])
                .unwrap()
                .bytes_len(),
            0
        );

        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes_mut(&mut bytes[9..])
                .unwrap()
                .bytes_len(),
            0
        );

        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes(&bytes[9..11]),
            Err(AlignmentError(
                AlignmentErrorInner::NotEnoughSpaceToRealign {
                    type_id: TypeData::of::<u64>(),
                    ptr: bytes[9..11].as_ptr(),
                    available_size: 2,
                    required_padding: 8
                }
            ))
        );

        let ptr = bytes[9..11].as_ptr();
        assert_eq!(
            AlignedSpace::<u64>::align_from_bytes_mut(&mut bytes[9..11]),
            Err(AlignmentError(
                AlignmentErrorInner::NotEnoughSpaceToRealign {
                    type_id: TypeData::of::<u64>(),
                    ptr,
                    available_size: 2,
                    required_padding: 8
                }
            ))
        );
    }

    fn test_writer(mut space: impl SpaceWriter) {
        let map = HashURIDMapper::new();
        let urids = crate::atoms::AtomURIDCollection::from_map(&map).unwrap();

        let mut test_data: Vec<u8> = vec![0; 128];
        for (i, data) in test_data.iter_mut().enumerate() {
            *data = i as u8;
        }

        let written_data = space.write_bytes(test_data.as_slice()).unwrap();
        assert_eq!(test_data.as_slice(), written_data);

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = space.write_value(test_atom).unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
        let written_atom_addr = written_atom as *mut _ as *mut _;

        let created_space = unsafe {
            AtomHeader::from_raw(written_atom)
                .assume_full_atom()
                .atom_space()
        };

        assert!(::core::ptr::eq(
            written_atom_addr,
            created_space.as_bytes().as_ptr()
        ));
        assert_eq!(created_space.bytes_len(), size_of::<sys::LV2_Atom>() + 42);

        {
            let space: &mut _ = &mut space;
            let mut atom_frame = AtomWriter::write_new(space, urids.chunk).unwrap();

            let mut test_data: Vec<u8> = vec![0; 24];
            for (i, data) in test_data.iter_mut().enumerate() {
                *data = i as u8;
            }

            let written_data = atom_frame.write_bytes(&test_data).unwrap();
            assert_eq!(test_data.as_slice(), written_data);
            assert_eq!(atom_frame.atom_header().size_of_body(), test_data.len());

            let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
            let written_atom = atom_frame.write_value(test_atom).unwrap();
            assert_eq!(written_atom.size, test_atom.size);
            assert_eq!(written_atom.type_, test_atom.type_);
            assert_eq!(
                atom_frame.atom_header().size_of_body(),
                test_data.len() + size_of_val(&test_atom)
            );
        }
    }

    #[test]
    fn test_root_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory = [0; MEMORY_SIZE];
        let cursor = SpaceCursor::new(&mut memory[..]);

        test_writer(cursor);
    }

    #[test]
    fn unaligned_root_write() {
        let mut raw_space = Box::new([0u8; 8]);

        {
            let mut root_space = SpaceCursor::new(&mut raw_space[3..]);
            root_space.write_value(42u8).unwrap();
        }

        assert_eq!(&[0, 0, 0, 42, 0, 0, 0, 0], raw_space.as_ref());
    }
}
