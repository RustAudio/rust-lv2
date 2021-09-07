use crate::header::AtomHeader;
use crate::space::reader::SpaceReader;
use crate::space::SpaceCursor;
use core::mem::{align_of, size_of};
use std::marker::PhantomData;
use std::mem::{size_of_val, MaybeUninit};
use std::slice::{from_raw_parts, from_raw_parts_mut};

/// An aligned slice of bytes that is designed to contain a given type `T` (by default, Atoms).
///
/// The accessor methods of this struct all behave in a similar way: If the internal slice is big enough, they create a reference to the start of the slice with the desired type and create a new space object that contains the space after the references instance.
#[repr(transparent)]
pub struct Space<T = AtomHeader> {
    _type: PhantomData<T>,
    // Note: this could be [MaybeUninit<T>] for alignment, but Spaces can have extra unaligned bytes at the end.
    data: [u8],
}

pub type AtomSpace = Space<AtomHeader>;

impl<T: 'static> Space<T> {
    /// Creates an empty Space.
    #[inline]
    pub fn empty<'a>() -> &'a Space<T> {
        // SAFETY: empty slices are always aligned
        unsafe { Self::from_bytes_unchecked(&[]) }
    }

    #[inline]
    pub(crate) fn padding_for(data: &[u8]) -> usize {
        let alignment = align_of::<T>();
        let start = data.as_ptr() as usize;
        if start % alignment == 0 {
            0
        } else {
            alignment - start % alignment
        }
    }

    /// Creates a new space from a slice of bytes, without checking for padding correctness.
    ///
    /// # Safety
    ///
    /// The caller of this method is responsible for ensuring that the slice's contents are correctly aligned.
    /// Otherwise, reads will be performed unaligned, which are either slow, a CPU crash, or UB depending on platforms.
    ///
    /// For a safe, checked version, see [`Space::try_from_bytes`].
    // NOTE: This method will always be used internally instead of the constructor, to make sure that
    // the unsafety is explicit and accounted for.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(data: &[u8]) -> &Space<T> {
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
    /// For a safe, checked version, see [`Space::try_from_bytes_mut`].
    // NOTE: This method will always be used internally instead of the constructor, to make sure that
    // the unsafety is explicit and accounted for.
    #[inline(always)]
    pub unsafe fn from_bytes_mut_unchecked(data: &mut [u8]) -> &mut Space<T> {
        // SAFETY: It is safe to transmute, since our type has repr(transparent) with [u8].
        // SAFETY: The caller is responsible to check for slice alignment.
        &mut *(data as *mut _ as *mut Self)
    }

    #[inline]
    pub(crate) fn from_uninit_slice(slice: &[MaybeUninit<T>]) -> &Self {
        // SAFETY: reinterpreting as raw bytes is safe for any value
        let bytes = unsafe { from_raw_parts(slice.as_ptr() as *const u8, size_of_val(slice)) };
        // SAFETY: The pointer is a slice of T, therefore it is already correctly aligned
        unsafe { Self::from_bytes_unchecked(bytes) }
    }

    #[inline]
    pub(crate) fn from_uninit_slice_mut(slice: &mut [MaybeUninit<T>]) -> &mut Self {
        // SAFETY: reinterpreting as raw bytes is safe for any value
        let bytes =
            unsafe { from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, size_of_val(slice)) };
        // SAFETY: The pointer is a slice of T, therefore it is already correctly aligned
        unsafe { Self::from_bytes_mut_unchecked(bytes) }
    }

    /// Creates a new space from a slice of bytes.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's offset is not 64-bit aligned
    /// (i.e. if it's pointer's value is not a multiple of `align_of::<T>()` bytes).
    ///
    /// This is the non-panicking version of [`Space::from_bytes`].
    #[inline]
    pub fn try_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.as_ptr() as usize % align_of::<T>() != 0 {
            return None;
        }

        // SAFETY: We just checked above that the pointer is correctly aligned
        Some(unsafe { Space::from_bytes_unchecked(data) })
    }

    /// Creates a new mutable space from a mutable slice of bytes.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's offset is not 64-bit aligned
    /// (i.e. if it's pointer's value is not a multiple of `align_of::<T>()` bytes).
    ///
    /// This is the non-panicking version of [`Space::from_bytes`].
    #[inline]
    pub fn try_from_bytes_mut(data: &mut [u8]) -> Option<&mut Self> {
        if data.as_ptr() as usize % align_of::<T>() != 0 {
            return None;
        }

        // SAFETY: We just checked above that the pointer is correctly aligned
        Some(unsafe { Space::from_bytes_mut_unchecked(data) })
    }

    /// Creates a new space from a slice of bytes, aligning it if necessary.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's is too small to contain
    /// aligned bytes (e.g. if it's smaller than `align_of::<T>()` bytes).
    #[inline]
    pub fn try_align_from_bytes(data: &[u8]) -> Option<&Self> {
        // SAFETY: We just aligned the slice start
        data.get(Self::padding_for(data)..)
            .map(|data| unsafe { Space::from_bytes_unchecked(data) })
    }

    /// Creates a new space from a slice of bytes, aligning it if necessary.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's is too small to contain
    /// aligned bytes (e.g. if it's smaller than `align_of::<T>()` bytes).
    #[inline]
    pub fn try_align_from_bytes_mut(data: &mut [u8]) -> Option<&mut Self> {
        // SAFETY: We just aligned the slice's start
        data.get_mut(Self::padding_for(data)..)
            .map(|data| unsafe { Space::from_bytes_mut_unchecked(data) })
    }

    #[inline]
    fn split_bytes_at(&self, mid: usize) -> Option<(&Self, &[u8])> {
        if mid > self.data.len() {
            return None;
        }

        let (start, end) = self.data.split_at(mid);
        // SAFETY: Because this data was the start of an existing Space, it was aligned already.
        let start = unsafe { Self::from_bytes_unchecked(start) };

        Some((start, end))
    }

    /// Try to retrieve space.
    ///
    /// This method calls [`split_raw`](#method.split_raw) and wraps the returned slice in an atom space. The second space is the space after the first one.
    #[inline]
    pub fn split_at(&self, mid: usize) -> Option<(&Self, &Self)> {
        let (start, end) = self.split_bytes_at(mid)?;
        let end = Self::try_align_from_bytes(end).unwrap_or_else(Space::empty);

        Some((start, end))
    }

    #[inline]
    pub fn realign<U: 'static>(&self) -> Option<&Space<U>> {
        Space::<U>::try_align_from_bytes(self.as_bytes())
    }

    #[inline]
    pub fn realign_mut<U: 'static>(&mut self) -> Option<&mut Space<U>> {
        Space::<U>::try_align_from_bytes_mut(self.as_bytes_mut())
    }

    #[inline]
    pub fn aligned<U: 'static>(&self) -> Option<&Space<U>> {
        Space::<U>::try_from_bytes(self.as_bytes())
    }

    #[inline]
    pub fn aligned_mut<U: 'static>(&mut self) -> Option<&mut Space<U>> {
        Space::<U>::try_from_bytes_mut(self.as_bytes_mut())
    }

    /// Return the internal slice of the space.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the internal slice of the space.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    #[inline]
    pub(crate) unsafe fn assume_init_value(&self) -> Option<&T> {
        // SAFETY: The caller has to ensure this slice actually points to initialized memory.
        Some(crate::util::assume_init_ref(self.as_uninit()?))
    }

    #[inline]
    pub(crate) unsafe fn assume_init_value_mut(&mut self) -> Option<&mut T> {
        // SAFETY: The caller has to ensure this slice actually points to initialized memory.
        Some(crate::util::assume_init_mut(self.as_uninit_mut()?))
    }

    /// Gets a `T`-aligned pointer to the contents.
    ///split_for_type
    /// This methods returns [`None`](Option::None) if the space is not large enough for a value of type `T`.
    #[inline]
    pub fn as_uninit(&self) -> Option<&MaybeUninit<T>> {
        if self.data.len() < size_of::<T>() {
            return None;
        }

        // SAFETY: We just checked that the space was actually big enough, and the alignment is guaranteed by this type.
        Some(unsafe { self.as_uninit_unchecked() })
    }

    /// Gets a `T`-aligned pointer to the contents.
    ///split_for_type
    /// This methods returns [`None`](Option::None) if the space is not large enough for a value of type `T`.
    #[inline]
    fn as_uninit_mut(&mut self) -> Option<&mut MaybeUninit<T>> {
        if self.data.len() < size_of::<T>() {
            return None;
        }

        // SAFETY: We just checked that the space was actually big enough, and the alignment is guaranteed by this type.
        Some(unsafe { self.as_uninit_mut_unchecked() })
    }

    /// Gets a `T`-aligned pointer to the contents, but without checking that there actually is enough space to hold `T`.
    #[inline]
    unsafe fn as_uninit_unchecked(&self) -> &MaybeUninit<T> {
        // SAFETY: The caller has to ensure that the space is actually big enough.
        &*(self.data.as_ptr() as *const MaybeUninit<T>)
    }

    /// Gets a `T`-aligned mutable pointer to the contents, but without checking that there actually is enough space to hold `T`.
    #[inline]
    pub(crate) unsafe fn as_uninit_mut_unchecked(&mut self) -> &mut MaybeUninit<T> {
        // SAFETY: The caller has to ensure that the space is actually big enough.
        &mut *(self.data.as_ptr() as *mut MaybeUninit<T>)
    }

    /// Gets the contents as a slice of potentially uninitialized `T`s.
    ///
    /// The resulting slice contains as many values as can fit in the original space.
    /// This means there might be less bytes in this slice than in this space, or zero if the space is too small for a single value.
    #[inline]
    pub(crate) fn as_uninit_slice(&self) -> &[MaybeUninit<T>] {
        // SAFETY: This type ensures alignment, so casting aligned bytes to uninitialized memory is safe.
        unsafe {
            ::core::slice::from_raw_parts(
                self.data.as_ptr() as *const MaybeUninit<T>,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    #[inline]
    pub unsafe fn assume_init_slice(&self) -> &[T] {
        crate::util::assume_init_slice(self.as_uninit_slice())
    }

    /// Gets the contents as a slice of potentially uninitialized `T`s.
    ///
    /// The resulting slice contains as many values as can fit in the original space.
    /// This means there might be less bytes in this slice than in this space, or zero if the space is too small for a single value.
    #[inline]
    pub(crate) fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        // SAFETY: This type ensures alignment, so casting aligned bytes to uninitialized memory is safe.
        unsafe {
            ::core::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut MaybeUninit<T>,
                self.data.len() / size_of::<T>(),
            )
        }
    }

    #[inline]
    pub fn read(&self) -> SpaceReader {
        SpaceReader::new(self.as_bytes())
    }

    #[inline]
    pub fn write(&mut self) -> SpaceCursor {
        SpaceCursor::new(self.as_bytes_mut())
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use crate::AtomHeader;
    use std::mem::{size_of, size_of_val};
    use urid::*;

    #[test]
    fn test_space() {
        let mut space = VecSpace::<u32>::new_with_capacity(64);
        let bytes = space.as_bytes_mut();

        for i in 0..128 {
            bytes[i] = i as u8;
        }

        unsafe {
            let ptr = space.as_space_mut().as_mut_ptr().add(128) as *mut u32;
            *(ptr) = 0x42424242;
        }

        let (lower_space, space) = space.as_space_mut().split_at(128).unwrap();
        let lower_space = lower_space.as_bytes();

        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let integer = unsafe { space.assume_init_value() }.unwrap();
        assert_eq!(*integer, 0x42424242);
    }

    #[test]
    fn test_split_atom() {
        let mut space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let space = space.as_space_mut();
        let urid: URID = unsafe { URID::new_unchecked(17) };

        // Writing an integer atom.
        unsafe {
            *(space.as_mut_ptr() as *mut sys::LV2_Atom_Int) = sys::LV2_Atom_Int {
                atom: sys::LV2_Atom {
                    size: size_of::<i32>() as u32,
                    type_: urid.get(),
                },
                body: 42,
            };

            let atom = space.read().next_atom().unwrap();
            let body = atom.body().as_bytes();

            assert_eq!(size_of::<i32>(), size_of_val(body));
            assert_eq!(42, *(body.as_ptr() as *const i32));
        }
    }

    fn test_mut_space<'a>(mut space: impl SpaceAllocator<'a>) {
        let map = HashURIDMapper::new();
        let urids = crate::atoms::AtomURIDCollection::from_map(&map).unwrap();

        let mut test_data: Vec<u8> = vec![0; 128];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
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
        assert_eq!(created_space.len(), size_of::<sys::LV2_Atom>() + 42);

        {
            let space: &mut _ = &mut space;
            let mut atom_frame = AtomSpaceWriter::write_new(space, urids.chunk).unwrap();

            let mut test_data: Vec<u8> = vec![0; 24];
            for i in 0..test_data.len() {
                test_data[i] = i as u8;
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

        test_mut_space(cursor);
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
