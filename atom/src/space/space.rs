use core::mem::{align_of, size_of, size_of_val};
use std::mem::MaybeUninit;
use std::marker::PhantomData;

#[inline]
pub(crate) fn required_padding_for_alignment(data: &[u8], alignment: usize) -> usize {
    let start = data.as_ptr() as usize;
    if start % alignment == 0 { 0 } else { alignment - start % alignment }
}

#[inline]
pub(crate) fn required_padding_for_type<T>(data: &[u8]) -> usize {
    required_padding_for_alignment(data, align_of::<T>())
}

#[inline]
pub(crate) fn as_bytes<T>(value: &T) -> &[u8] {
    // SAFETY: any type can safely be transmuted to a byte slice
    unsafe {
        std::slice::from_raw_parts(value as *const T as *const u8, size_of_val(value))
    }
}

/// An aligned slice of bytes that is designed to contain a given type `T` (by default, Atoms).
///
/// The accessor methods of this struct all behave in a similar way: If the internal slice is big enough, they create a reference to the start of the slice with the desired type and create a new space object that contains the space after the references instance.
pub struct Space<T = lv2_sys::LV2_Atom> {
    _type: PhantomData<T>,
    data: [u8]
}

impl<T> Space<T> {
    /// Creates an empty Space.
    #[inline]
    pub const fn empty() -> &'static Space<T> {
        &Space { _type: PhantomData, data: *&[][..] }
    }

    /// Creates an empty mutable Space.
    #[inline]
    pub const fn empty_mut() -> &'static mut Space<T> {
        &mut Space { _type: PhantomData, data: *&mut [][..] }
    }

    /// Creates a new space from a slice of bytes, without checking for padding correctness.
    ///
    /// # Safety
    ///
    /// The caller of this method is responsible for ensuring that the slice's contents are correctly aligned.
    /// Otherwise, atom reads will be performed unaligned, which are either slow, a CPU crash, or UB depending on platforms.
    ///
    /// For a safe, checked version, see [`Space::from_bytes`].
    // NOTE: This method will always be used internally instead of the constructor, to make sure that
    // the unsafety is explicit and accounted for.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(data: &[u8]) -> &Space<T> {
        &Space { _type: PhantomData, data: *data }
    }

    /// Creates a new mutable space from a slice of bytes, without checking for padding correctness.
    ///
    /// # Safety
    ///
    /// The caller of this method is responsible for ensuring that the slice's contents are correctly aligned.
    /// Otherwise, atom reads will be performed unaligned, which are either slow, a CPU crash, or UB depending on platforms.
    ///
    /// For a safe, checked version, see [`Space::from_bytes`].
    // NOTE: This method will always be used internally instead of the constructor, to make sure that
    // the unsafety is explicit and accounted for.
    #[inline(always)]
    pub unsafe fn from_bytes_mut_unchecked(data: &mut [u8]) -> &mut Space<T> {
        &mut Space { _type: PhantomData, data: *data }
    }

    /// Create a new space from an atom pointer.
    ///
    /// The method creates a space that contains the atom as well as its body.
    ///
    /// # Safety
    ///
    /// Since the body is not included in the atom reference, this method has to assume that it is valid memory and therefore is unsafe but sound.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn from_atom(atom: &sys::LV2_Atom) -> &Space<sys::LV2_Atom> {
        let data = std::slice::from_raw_parts(
            atom as *const sys::LV2_Atom as *const u8,
            atom.size as usize + size_of::<sys::LV2_Atom>(),
        );
        Self::from_bytes(data)
    }

    /// Create a new mutable space from an atom pointer.
    ///
    /// The method creates a space that contains the atom as well as its body.
    ///
    /// # Safety
    ///
    /// Since the body is not included in the atom reference, this method has to assume that it is valid memory and therefore is unsafe but sound.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn from_atom_mut(atom: &mut sys::LV2_Atom) -> &mut Space<sys::LV2_Atom> {
        let data = std::slice::from_raw_parts_mut(
            atom as *mut sys::LV2_Atom as *mut u8,
            atom.size as usize + size_of::<sys::LV2_Atom>(),
        );
        Self::from_bytes_mut(data)
    }

    /// Creates a new space from a slice of bytes.
    ///
    /// # Panics
    ///
    /// This method panics if the given slice's offset is not 64-bit aligned
    /// (i.e. if it's pointer's value is not a multiple of `align_of::<T>()` bytes).
    ///
    /// For a non-panicking version, see [`Space::try_from_bytes`].
    #[inline]
    pub fn from_bytes(data: &[u8]) -> &Self {
        Space::try_from_bytes(data).unwrap()
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

    /// Creates a new space from a slice of bytes, aligning it if necessary.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's is too small to contain
    /// aligned bytes (e.g. if it's smaller than `align_of::<T>()` bytes).
    #[inline]
    pub fn try_align_from_bytes(data: &[u8]) -> Option<&Self> {
        // SAFETY: The slice was just aligned by align_slice
        data.get(required_padding_for_type::<T>(data)..).map(|data| unsafe { Space::from_bytes_unchecked(data) })
    }

    /// Creates a new space from a slice of bytes, aligning it if necessary.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's is too small to contain
    /// aligned bytes (e.g. if it's smaller than `align_of::<T>()` bytes).
    #[inline]
    pub fn try_align_from_bytes_mut(data: &mut [u8]) -> Option<&mut Self> {
        // SAFETY: The slice was just aligned by align_slice_mut
        data.get_mut(required_padding_for_type::<T>(data)..).map(|data| unsafe { Space::from_bytes_mut_unchecked(data) })
    }

    #[inline]
    pub fn split_bytes_at(&self, mid: usize) -> Option<(&Self, &[u8])> {
        if mid > self.data.len() {
            return None;
        }

        let (start, end) = self.data.split_at(mid);
        // SAFETY: Because this data was the start of an existing Space, it was aligned already.
        let start = unsafe { Self::from_bytes_unchecked(start) };

        Some((start, end))
    }

    #[inline]
    pub fn split_bytes_at_mut(&mut self, mid: usize) -> Option<(&mut Self, &mut [u8])> {
        if mid > self.data.len() {
            return None;
        }

        let (start, end) = self.data.split_at_mut(mid);
        // SAFETY: Because this data was the start of an existing Space, it was aligned already.
        let start = unsafe { Self::from_bytes_unchecked_mut(start) };

        Some((start, end))
    }

    /// Try to retrieve space.
    ///
    /// This method calls [`split_raw`](#method.split_raw) and wraps the returned slice in an atom space. The second space is the space after the first one.
    #[inline]
    pub fn split_at(&self, mid: usize) -> Option<(&Self, &Self)> {
        let (start, end) = self.split_bytes_at(mid)?;
        let end = Self::try_align_from_bytes(end).unwrap_or(Space::empty());

        Some((start, end))
    }

    #[inline]
    pub fn split_for_value(&self) -> Option<(&MaybeUninit<T>, &Self)> {
        let (value, rest) = self.split_at(size_of::<T>())?;
        let value = value.as_uninit()?;

        Some((value, rest))
    }

    #[inline]
    pub fn split_at_mut(&mut self, mid: usize) -> Option<(&mut Space, &mut Space)> {
        let (start, end) = self.split_bytes_at_mut(mid)?;
        let end = Self::try_align_from_bytes_mut(end).unwrap_or(Space::empty_mut());

        Some((start, end))
    }

    /// Create a space from a reference.
    ///
    /// # Panics
    ///
    /// This method panics if the given instance pointer isn't 64-bit aligned.
    #[inline]
    pub fn from_ref(instance: &T) -> &Self {
        // SAFETY: references are always aligned.
        unsafe { Space::from_bytes_unchecked(as_bytes(instance)) }
    }

    #[inline]
    pub fn realigned<U>(&self) -> Option<&Space<U>> {
        Space::<U>::try_from_bytes(self.as_bytes())
    }

    /// Return the internal slice of the space.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Return the internal slice of the space.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    #[inline]
    pub unsafe fn read(&self) -> Option<&T> {
        // SAFETY: The caller has to ensure this slice actually points to initialized memory.
        Some(unsafe { &*(self.as_uninit()?) })
    }

    #[inline]
    pub unsafe fn read_as<U>(&self) -> Option<U> {
        self.realigned()?.read()
    }

    /// Gets a `T`-aligned pointer to the contents.
    ///
    /// This methods returns [`None`](Option::None) if the space is not large enough for a value of type `T`.
    #[inline]
    pub fn as_uninit(&self) -> Option<&MaybeUninit<T>> {
        if self.data.len() < size_of::<T>() {
            return None;
        }

        // SAFETY: We just checked that the space was actually big enough.
        Some(unsafe { self.as_uninit_unchecked() })
    }

    /// Gets a `T`-aligned pointer to the contents, but without checking that there actually is enough space to hold `T`.
    #[inline]
    pub unsafe fn as_uninit_unchecked(&self) -> &MaybeUninit<T> {
        // SAFETY: The caller has to ensure that the space is actually big enough.
        unsafe { &*(self.data.as_ptr() as *const MaybeUninit<T>) }
    }

    /// Gets a `T`-aligned mutable pointer to the contents, but without checking that there actually is enough space to hold `T`.
    #[inline]
    pub unsafe fn as_uninit_mut_unchecked(&mut self) -> &mut MaybeUninit<T> {
        // SAFETY: The caller has to ensure that the space is actually big enough.
        unsafe { &mut *(self.data.as_ptr() as *mut MaybeUninit<T>) }
    }
}

impl Space {

}