//! Smart pointers with safe atom reading and writing methods.
//!
//! # Safety
//!
//! The only unsafe things that happen in this module is when either space is created from a reference to a `sys::LV2_Atom` and when space is re-interpreted as typed data.
//!
//! In the first case, we have to trust that the space behind the atom header is accessible since we have no way to check whether it is or not. Therefore, we have to assume that it is sound.
//!
//! The second case is sound since a) the data is contained in a slice and therefore is accessible, b) generic type parameter bounds assure that the type is plain-old-data and c) 64-bit padding is assured.
use crate::Atom;
use std::cell::Cell;
use std::marker::Unpin;
use std::mem::{size_of, size_of_val};
use urid::URID;

#[inline]
fn pad_slice(data: &[u8]) -> Option<&[u8]> {
    let start = data.as_ptr() as usize;
    let padding = if start % 8 == 0 { 0 } else { 8 - start % 8 };

    data.get(padding..)
}

#[inline]
fn as_bytes<T: ?Sized>(value: &T) -> &[u8] {
    // SAFETY: any type can safely be transmuted to a byte slice
    unsafe {
        std::slice::from_raw_parts(value as *const T as *const u8, size_of_val(value))
    }
}

#[inline]
fn as_bytes_mut<T: ?Sized>(value: &mut T) -> &mut [u8] {
    // SAFETY: any type can safely be transmuted to a byte slice
    unsafe {
        std::slice::from_raw_parts_mut(value as *mut T as *mut u8, size_of_val(value))
    }
}

/// A 64-bit aligned slice of bytes that is designed to contain Atoms.
///
/// The accessor methods of this struct all behave in a similar way: If the internal slice is big enough, they create a reference to the start of the slice with the desired type and create a new space object that contains the space after the references instance.
#[derive(Clone, Copy)]
pub struct Space {
    data: [u8],
}

impl Space {
    /// Creates an empty Space.
    #[inline]
    pub const fn empty() -> &'static Space {
        &Space { data: *&[][..] }
    }

    /// Create a new space from an atom pointer.
    ///
    /// The method creates a space that contains the atom as well as it's body.
    ///
    /// # Safety
    ///
    /// Since the body is not included in the atom reference, this method has to assume that it is valid memory and therefore is unsafe but sound.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn from_atom(atom: &sys::LV2_Atom) -> &Self {
        let data = std::slice::from_raw_parts(
            atom as *const sys::LV2_Atom as *const u8,
            atom.size as usize + size_of::<sys::LV2_Atom>(),
        );
        Self::from_bytes(data)
    }

    /// Creates a new space from a slice of bytes.
    ///
    /// # Panics
    ///
    /// This method panics if the given slice's offset is not 64-bit aligned
    /// (i.e. if it's pointer's value is not a multiple of 8 bytes).
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
    /// (i.e. if it's pointer's value is not a multiple of 8 bytes).
    ///
    /// This is the non-panicking version of [`Space::from_bytes`].
    #[inline]
    pub fn try_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.as_ptr() as usize % 8 != 0 {
            return None;
        }

        Some(&Space { data: *data })
    }

    /// Creates a new space from a slice of bytes.
    ///
    /// # Errors
    ///
    /// This method returns [`None`](Option::None) if the given slice's offset is not 64-bit aligned
    /// (i.e. if it's pointer's value is not a multiple of 8 bytes).
    ///
    /// This is the non-panicking version of [`Space::from_bytes`].
    #[inline]
    pub fn try_from_bytes_padded(data: &[u8]) -> Option<&Self> {
        pad_slice(data).map(|data| &Self { data: *data })
    }

    /// Try to retrieve a slice of bytes.
    ///
    /// This method basically splits off the lower part of the internal bytes slice and creates a new atom space pointer of the upper part. Since atoms have to be 64-bit-aligned, there might be a padding space that's neither in the lower nor in the upper part.
    pub fn split_bytes_at(&self, size: usize) -> Option<(&[u8], &Space)> {
        if size > self.data.len() {
            return None;
        }

        let (lower_space, upper_space) = self.data.split_at(size);
        let upper_space = Self::try_from_bytes_padded(upper_space).unwrap_or(Space::empty());
        Some((lower_space, upper_space))
    }

    /// Try to retrieve space.
    ///
    /// This method calls [`split_raw`](#method.split_raw) and wraps the returned slice in an atom space. The second space is the space after the first one.
    pub fn split_at(&self, size: usize) -> Option<(&Space, &Space)> {
        self.split_bytes_at(size)
            .map(|(data, rhs)| (Self::from_bytes(data), rhs))
    }

    /// Try to retrieve a reference to a sized type.
    ///
    /// This method retrieves a slice of memory using the [`split_raw`](#method.split_raw) method and interprets it as an instance of `T`. Since there is no way to check that the memory is actually a valid instance of `T`, this method is unsafe. The second return value is the space after the instance of `T`.
    pub unsafe fn split_for_type<'a, T>(&'a self) -> Option<(&'a T, &'a Space)> where T: 'a,
    {
        self.split_bytes_at(size_of::<T>())
            .map(|(data, rhs)| (unsafe { &*(data.as_ptr() as *const T) }, rhs))
    }

    /// Try to retrieve the space occupied by an atom.
    ///
    /// This method assumes that the space contains an atom and retrieves the space occupied by the atom, including the atom header. The second return value is the rest of the space behind the atom.
    ///
    /// The difference to [`split_atom_body`](#method.split_atom_body) is that the returned space contains the header of the atom and that the type of the atom is not checked.
    pub unsafe fn split_atom(&self) -> Option<(&Self, &Self)> {
        let (header, _) = self.split_for_type::<sys::LV2_Atom>()?;
        self.split_at(size_of::<sys::LV2_Atom>() + header.size as usize)
    }

    /// Try to retrieve the body of the atom.
    ///
    /// This method retrieves the header of the atom. If the type URID in the header matches the given URID, it returns the body of the atom. If not, it returns `None`. The first space is the body of the atom, the second one is the space behind it.
    ///
    /// The difference to [`split_atom`](#method.split_atom) is that the returned space does not contain the header of the atom and that the type of the atom is checked.
    pub unsafe fn split_atom_body<T: ?Sized>(&self, urid: URID<T>) -> Option<(&Self, &Self)> {
        let (header, space) = self.split_for_type::<sys::LV2_Atom>()?;
        if header.type_ != urid.get() {
            return None;
        }
        space.split_at(header.size as usize)
    }

    /// Create a space from a reference.
    ///
    /// # Panics
    ///
    /// This method panics if the given instance pointer isn't 64-bit aligned.
    pub fn from_ref<T: ?Sized>(instance: &T) -> &Self {
        Space::try_from_bytes_padded(as_bytes(instance)).unwrap()
    }

    /// Return the internal slice of the space.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
pub trait MutSpace<'a> {
    /// Try to allocate memory on the internal data slice.
    ///
    /// If `apply_padding` is `true`, the method will assure that the allocated memory is 64-bit-aligned. The first return value is the number of padding bytes that has been used and the second return value is a mutable slice referencing the allocated data.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    fn allocate(&mut self, size: usize) -> Option<(usize, &'a mut Space)>;

    fn allocate_unpadded(&mut self, size: usize) -> Option<&'a mut [u8]>;

    fn write_raw_unpadded(&mut self, data: &[u8]) -> Option<&'a mut [u8]> {
        let space = self.allocate_unpadded(data.len())?;
        space.copy_from_slice(data);
        Some(space)
    }

    /// Try to write data to the internal data slice.
    ///
    /// The method allocates a slice with the [`allocate`](#tymethod.allocate) method and copies the data to the slice.
    fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut Space> {
        self.allocate(data.len(), apply_padding).map(|(_, space)| {
            space.copy_from_slice(data);
            space
        })
    }
}

/// A `MutSpace` that directly manages it's own internal data slice.
pub struct RootMutSpace<'a> {
    space: Cell<Option<&'a mut [u8]>>,
    allocated_bytes: usize,
}

impl<'a> RootMutSpace<'a> {
    /// Create new space from an atom.
    ///
    /// The method creates a space that contains the atom as well as it's body.
    ///
    /// # Safety
    ///
    /// Since the body is not included in the atom reference, this method has to assume that it is valid memory and therefore is unsafe.
    pub unsafe fn from_atom(atom: &mut sys::LV2_Atom) -> Self {
        let space = std::slice::from_raw_parts_mut(
            atom as *mut _ as *mut u8,
            atom.size as usize + size_of::<sys::LV2_Atom>(),
        );
        Self::new(space)
    }

    /// Create a new instance.
    ///
    /// This method takes the space reserved for the value and interprets it as a slice of bytes (`&mut [u8]`).
    pub fn new(space: &'a mut [u8]) -> Self {
        RootMutSpace {
            space: Cell::new(Some(space)),
            allocated_bytes: 0,
        }
    }
}

impl<'a> MutSpace<'a> for RootMutSpace<'a> {
    fn allocate(&mut self, size: usize, apply_padding: bool) -> Option<(usize, &'a mut [u8])> {
        if self.space.get_mut().is_none() {
            return None;
        }
        let mut space = self.space.replace(None).unwrap();

        let padding = if apply_padding {
            let alignment = self.allocated_bytes % 8;
            let padding = if alignment == 0 { 0 } else { 8 - alignment };
            if padding > space.len() {
                return None;
            }
            space = space.split_at_mut(padding).1;
            self.allocated_bytes += padding;
            padding
        } else {
            0
        };

        if size > space.len() {
            return None;
        }
        let (lower_slice, upper_slice) = space.split_at_mut(size);
        self.allocated_bytes += size;

        self.space.set(Some(upper_slice));
        Some((padding, lower_slice))
    }
}

/// Linked list element for dynamic atom writing.
///
/// This struct works in conjunction with [`SpaceHead`](struct.SpaceHead.html) to provide a way to write atoms to dynamically allocated memory.
pub struct SpaceElement {
    next: Option<(Box<Self>, Box<[u8]>)>,
}

impl Default for SpaceElement {
    fn default() -> Self {
        Self { next: None }
    }
}

impl SpaceElement {
    /// Append an element to the list.
    ///
    /// If this is the last element of the list, allocate a slice of the required length and append a new element to the list. If not, do nothing and return `None`.
    pub fn allocate(&mut self, size: usize) -> Option<(&mut Self, &mut [u8])> {
        if self.next.is_some() {
            return None;
        }

        let new_data = vec![0u8; size].into_boxed_slice();
        let new_element = Box::new(Self::default());
        self.next = Some((new_element, new_data));
        self.next
            .as_mut()
            .map(|(new_element, new_data): &mut (Box<Self>, Box<[u8]>)| {
                (new_element.as_mut(), new_data.as_mut())
            })
    }

    /// Create a vector containing the data from all elements following this one.
    pub fn to_vec(&self) -> Vec<u8> {
        self.iter()
            .map(|slice| slice.iter())
            .flatten()
            .cloned()
            .collect()
    }

    /// Return an iterator over the chunks of all elements following this one.
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        std::iter::successors(self.next.as_ref(), |element| element.0.next.as_ref())
            .map(|(_, data)| data.as_ref())
    }
}

/// A mutable space that dynamically allocates memory.
///
/// This space uses a linked list of [`SpaceElement`s](struct.SpaceElement.html) to allocate memory. Every time `allocate` is called, a new element is appended to the list and a new byte slice is created.
///
/// In order to use this space and retrieve the written data once it was written, you create a `SpaceElement` and create a new head with it. Then, you use the head like any other `MutSpace` and when you're done, you retrieve the written data by either calling [`to_vec`](struct.SpaceElement.html#method.to_vec) or [`iter`](struct.SpaceElement.html#iter).
///
/// # Usage example
///
/// ```
/// # use lv2_core::prelude::*;
/// # use lv2_atom::prelude::*;
/// # use lv2_atom::space::*;
/// # use urid::*;
/// # use std::pin::Pin;
/// # let map = HashURIDMapper::new();
/// // URID cache creation is omitted.
/// let urids: AtomURIDCollection = map.populate_collection().unwrap();
///
/// // Creating the first element in the list and the writing head.
/// let mut element = SpaceElement::default();
/// let mut head = SpaceHead::new(&mut element);
///
/// // Writing an integer.
/// (&mut head as &mut dyn MutSpace).init(urids.int, 42).unwrap();
///
/// // Retrieving a continuos vector with the written data and verifying it's contents.
/// let written_data: Vec<u8> = element.to_vec();
/// let atom = unsafe { UnidentifiedAtom::new_unchecked(Space::from_slice(written_data.as_ref())) };
/// assert_eq!(42, atom.read(urids.int, ()).unwrap());
/// ```
pub struct SpaceHead<'a> {
    element: Option<&'a mut SpaceElement>,
    allocated_space: usize,
}

impl<'a> SpaceHead<'a> {
    /// Create a new head that references the given element.
    pub fn new(element: &'a mut SpaceElement) -> Self {
        Self {
            element: Some(element),
            allocated_space: 0,
        }
    }

    fn internal_allocate(&mut self, size: usize) -> Option<&'a mut [u8]> {
        let element = self.element.take()?;
        let (new_element, new_space) = element.allocate(size)?;
        self.element = Some(new_element);
        self.allocated_space += size;
        Some(new_space)
    }
}

impl<'a> MutSpace<'a> for SpaceHead<'a> {
    fn allocate(&mut self, size: usize, apply_padding: bool) -> Option<(usize, &'a mut [u8])> {
        let padding: usize = if apply_padding {
            (8 - self.allocated_space % 8) % 8
        } else {
            0
        };

        if padding != 0 {
            self.internal_allocate(padding);
        }

        self.internal_allocate(size)
            .map(|new_space| (padding, new_space))
    }
}

/// A `MutSpace` that notes the amount of allocated space in an atom header.
pub struct FramedMutSpace<'a, 'b> {
    atom: &'a mut sys::LV2_Atom,
    parent: &'b mut dyn MutSpace<'a>,
}

impl<'a, 'b> FramedMutSpace<'a, 'b> {
    /// Create a new framed space with the given parent and type URID.
    pub fn new<A: ?Sized>(parent: &'b mut dyn MutSpace<'a>, urid: URID<A>) -> Option<Self> {
        let atom = sys::LV2_Atom {
            size: 0,
            type_: urid.get(),
        };
        let atom: &'a mut sys::LV2_Atom = parent.write(&atom, true)?;
        Some(Self { atom, parent })
    }
}

impl<'a, 'b> MutSpace<'a> for FramedMutSpace<'a, 'b> {
    fn allocate(&mut self, size: usize, apply_padding: bool) -> Option<(usize, &'a mut [u8])> {
        self.parent
            .allocate(size, apply_padding)
            .map(|(padding, data)| {
                self.atom.size += (size + padding) as u32;
                (padding, data)
            })
    }
}

impl<'a, 'b> dyn MutSpace<'a> + 'b {
    /// Write a sized object to the space.
    ///
    /// If `apply_padding` is `true`, the method will assure that the written instance is 64-bit-aligned.
    pub fn write<T>(&mut self, instance: &T, apply_padding: bool) -> Option<&'a mut T>
    where
        T: Unpin + Copy + Send + Sync + Sized + 'static,
    {
        let size = std::mem::size_of::<T>();
        let input_data =
            unsafe { std::slice::from_raw_parts(instance as *const T as *const u8, size) };

        let output_data = self.write_raw(input_data, apply_padding)?;

        assert_eq!(size, output_data.len());
        Some(unsafe { &mut *(output_data.as_mut_ptr() as *mut T) })
    }

    /// Initialize a new atom in the space.
    pub fn init<'c, A: Atom<'a, 'c>>(
        &'c mut self,
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        let new_space = FramedMutSpace::new(self, urid)?;
        A::init(new_space, parameter)
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use std::mem::{size_of, size_of_val};
    use urid::*;

    #[test]
    fn test_space() {
        let mut vector: Vec<u8> = vec![0; 256];
        for i in 0..128 {
            vector[i] = i as u8;
        }
        unsafe {
            let ptr = vector.as_mut_slice().as_mut_ptr().add(128) as *mut u32;
            *(ptr) = 0x42424242;
        }

        let space = Space::from_bytes(vector.as_slice());
        let (lower_space, space) = space.split_bytes_at(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let (integer, _) = unsafe { space.split_for_type::<u32>() }.unwrap();
        assert_eq!(*integer, 0x42424242);
    }

    #[test]
    fn test_split_atom() {
        let mut data: Box<[u64]> = Box::new([0; 256]);
        let urid: URID = unsafe { URID::new_unchecked(17) };

        // Writing an integer atom.
        unsafe {
            *(data.as_mut_ptr() as *mut sys::LV2_Atom_Int) = sys::LV2_Atom_Int {
                atom: sys::LV2_Atom {
                    size: size_of::<i32>() as u32,
                    type_: urid.get(),
                },
                body: 42,
            };

            let space = Space::from_ref(data.as_ref());
            let (atom, _) = space.split_atom().unwrap();
            let (body, _) = atom.split_atom_body(urid).unwrap();
            let body = body.as_bytes();

            assert_eq!(size_of::<i32>(), size_of_val(body));
            assert_eq!(42, unsafe { *(body.as_ptr() as *const i32) });
        }
    }

    #[test]
    fn test_from_reference() {
        let value: u64 = 0x42424242;
        let space = Space::from_ref(&value);
        assert_eq!(value, *unsafe { space.split_for_type::<u64>() }.unwrap().0);
    }

    fn test_mut_space<'a, S: MutSpace<'a>>(mut space: S) {
        let map = HashURIDMapper::new();
        let urids = crate::AtomURIDCollection::from_map(&map).unwrap();

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        match space.write_raw(test_data.as_slice(), true) {
            Some(written_data) => assert_eq!(test_data.as_slice(), written_data),
            None => panic!("Writing failed!"),
        }

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = (&mut space as &mut dyn MutSpace)
            .write(&test_atom, true)
            .unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);

        let created_space = unsafe { RootMutSpace::from_atom(written_atom) }
            .space
            .take()
            .unwrap();
        assert_eq!(
            created_space.as_ptr() as usize,
            written_atom as *mut _ as usize
        );
        assert_eq!(created_space.len(), size_of::<sys::LV2_Atom>() + 42);

        let mut atom_frame =
            FramedMutSpace::new(&mut space as &mut dyn MutSpace, urids.chunk).unwrap();

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        let written_data = atom_frame.write_raw(test_data.as_slice(), true).unwrap();
        assert_eq!(test_data.as_slice(), written_data);
        assert_eq!(atom_frame.atom.size, test_data.len() as u32);

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let borrowed_frame = &mut atom_frame as &mut dyn MutSpace;
        let written_atom = borrowed_frame.write(&test_atom, true).unwrap();
        assert_eq!(written_atom.size, test_atom.size);
        assert_eq!(written_atom.type_, test_atom.type_);
        assert_eq!(
            atom_frame.atom.size as usize,
            test_data.len() + size_of_val(&test_atom)
        );
    }

    #[test]
    fn test_root_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let frame: RootMutSpace = RootMutSpace::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        test_mut_space(frame);
    }

    #[test]
    fn test_space_head() {
        let mut space = SpaceElement::default();
        let head = SpaceHead::new(&mut space);
        test_mut_space(head);
    }

    #[test]
    fn test_padding_inside_frame() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let raw_space: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        };

        // writing
        {
            let mut root: RootMutSpace = RootMutSpace::new(raw_space);
            let mut frame =
                FramedMutSpace::new(&mut root as &mut dyn MutSpace, URID::<()>::new(1).unwrap())
                    .unwrap();
            {
                let frame = &mut frame as &mut dyn MutSpace;
                frame.write::<u32>(&42, true).unwrap();
                frame.write::<u32>(&17, true).unwrap();
            }
        }

        // checking
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, 1);
            assert_eq!(atom.size as usize, 12);

            let (value, space) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 42);
            let (_, space) = space.split_at(4);

            let (value, _) = space.split_at(size_of::<u32>());
            let value = unsafe { *(value.as_ptr() as *const u32) };
            assert_eq!(value, 17);
        }
    }

    #[test]
    fn unaligned_root_write() {
        let mut raw_space = Box::new([0u8; 8]);

        {
            let mut root_space = RootMutSpace::new(&mut raw_space[3..]);
            (&mut root_space as &mut dyn MutSpace)
                .write(&42u8, true)
                .unwrap();
        }

        assert_eq!(&[0, 0, 0, 42, 0, 0, 0, 0], raw_space.as_ref());
    }
}
