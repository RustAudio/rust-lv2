//! An utility module exposing `Cell`-like utilities for realtime overwritable buffers.
//!
//! Those type are useful to enforce the safety guarantees of e.g. audio buffers that are both the
//! read-only input buffer and the write-only output buffer, i.e. for audio plugins that do in-place
//! processing.

use std::cell::Cell;

/// A wrapper around `Cell<T>` that only allows reading a `Copy` value out of a buffer.
///
/// This type is useful to enforce the guarantees of a read-only buffer that may be overwritten at
/// any time, such as a
#[repr(transparent)]
pub struct ReadCell<T: ?Sized> {
    value: Cell<T>,
}

unsafe impl<T: ?Sized> Send for ReadCell<T> where T: Send {}

impl<T> ReadCell<T> {
    /// Creates a new `ReadCell` containing the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::ReadCell;
    ///
    /// let c = ReadCell::new(5);
    /// ```
    #[inline]
    pub const fn new(value: T) -> ReadCell<T> {
        ReadCell { value: Cell::new(value) }
    }
}

impl<T: ?Sized> ReadCell<T> {
    /// Returns a `&ReadCell<T>` from a `&T`
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::ReadCell;
    ///
    /// let slice: &[i32] = &[1, 2, 3];
    /// let cell_slice: &ReadCell<[i32]> = ReadCell::from_ref(slice);
    /// let slice_cell: &[ReadCell<i32>] = cell_slice.as_slice_of_cells();
    ///
    /// assert_eq!(slice_cell.len(), 3);
    /// ```
    #[inline]
    pub fn from_ref(t: &T) -> &ReadCell<T> {
        // SAFETY: `&mut` ensures unique access.
        unsafe { &*(t as *const T as  *const ReadCell<T>) }
    }
}

impl<T: Copy> ReadCell<T> {
    /// Returns a copy of the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::ReadCell;
    ///
    /// let c = ReadCell::new(5);
    ///
    /// let five = c.get();
    /// ```
    #[inline]
    pub fn get(&self) -> T {
        self.value.get()
    }
}

impl<T> ReadCell<[T]> {
    /// Returns a `&[ReadCell<T>]` from a `&ReadCell<[T]>`
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::ReadCell;
    ///
    /// let slice: &mut [i32] = &mut [1, 2, 3];
    /// let cell_slice: &ReadCell<[i32]> = ReadCell::from_ref(slice);
    /// let slice_cell: &[ReadCell<i32>] = cell_slice.as_slice_of_cells();
    ///
    /// assert_eq!(slice_cell.len(), 3);
    /// ```
    #[inline]
    pub fn as_slice_of_cells(&self) -> &[ReadCell<T>] {
        // SAFETY: `Cell<T>` has the same memory layout as `T`.
        unsafe { &*(self as *const ReadCell<[T]> as *const [ReadCell<T>]) }
    }
}

/// A wrapper around `Cell<T>` that only allows writing a value into a buffer.
#[repr(transparent)]
pub struct WriteCell<T: ?Sized> {
    value: Cell<T>,
}

impl<T: ?Sized> WriteCell<T> {
    /// Returns a `&WriteCell<T>` from a `&mut T`
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::WriteCell;
    ///
    /// let slice: &mut [i32] = &mut [1, 2, 3];
    /// let cell_slice: &WriteCell<[i32]> = WriteCell::from_mut(slice);
    /// let slice_cell: &[WriteCell<i32>] = cell_slice.as_slice_of_cells();
    ///
    /// assert_eq!(slice_cell.len(), 3);
    /// ```
    #[inline]
    pub fn from_mut(t: &mut T) -> &WriteCell<T> {
        // SAFETY: `&mut` ensures unique access, and WriteCell<T> has the same memory layout as T.
        unsafe { &*(t as *mut T as *const WriteCell<T>) }
    }
}

impl<T> WriteCell<T> {
    /// Creates a new `WriteCell` containing the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::WriteCell;
    ///
    /// let c = WriteCell::new(5);
    /// ```
    #[inline]
    pub const fn new(value: T) -> WriteCell<T> {
        WriteCell { value: Cell::new(value) }
    }

    /// Sets the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::WriteCell;
    ///
    /// let c = WriteCell::new(5);
    ///
    /// c.set(10);
    /// ```
    #[inline]
    pub fn set(&self, val: T) {
        self.value.set(val);
    }
}

impl<T> WriteCell<[T]> {
    /// Returns a `&[WriteCell<T>]` from a `&WriteCell<[T]>`
    ///
    /// # Examples
    ///
    /// ```
    /// use lv2_core::cell::WriteCell;
    ///
    /// let slice: &mut [i32] = &mut [1, 2, 3];
    /// let cell_slice: &WriteCell<[i32]> = WriteCell::from_mut(slice);
    /// let slice_cell: &[WriteCell<i32>] = cell_slice.as_slice_of_cells();
    ///
    /// assert_eq!(slice_cell.len(), 3);
    /// ```
    pub fn as_slice_of_cells(&self) -> &[WriteCell<T>] {
        // SAFETY: `Cell<T>` has the same memory layout as `T`.
        unsafe { &*(self as *const WriteCell<[T]> as *const [WriteCell<T>]) }
    }
}