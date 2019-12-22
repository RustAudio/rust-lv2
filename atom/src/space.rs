//! Smart pointers with safe atom reading and writing methods.
//!
//! # Safety
//!
//! The only unsafe things that happen in this module is when either space is created from a reference to a `sys::LV2_Atom` and when space is re-interpreted as typed data.
//!
//! In the first case, we have to trust that the space behind the atom header is accessible since we have no way to check whether it is or not. Therefore, we have to assume that it is sound.
//!
//! The second case is sound since a) the data is contained in a slice and therefore is accessible, b) generic type parameter bounds assure that the type is plain-old-data and c) 64-bit padding is assured.
use std::cell::Cell;
use std::marker::Unpin;
use std::mem::{size_of, size_of_val};
use urid::URID;

/// Specialized smart pointer to retrieve struct instances from a slice of memory.
///
/// The accessor methods of this struct all behave in a similar way: If the internal slice is big enough, they create a reference to the start of the slice with the desired type and create a new space object that contains the space after the references instance.
#[derive(Clone, Copy)]
pub struct Space<'a> {
    data: Option<&'a [u8]>,
}

impl<'a> Space<'a> {
    /// Create a new space from an atom pointer.
    ///
    /// The method creates a space that contains the atom as well as it's body.
    ///
    /// # Safety
    ///
    /// Since the body is not included in the atom reference, this method has to assume that it is valid memory and therefore is unsafe but sound.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn from_atom(atom: &sys::LV2_Atom) -> Self {
        let size = atom.size as usize;
        let data = std::slice::from_raw_parts(
            atom as *const sys::LV2_Atom as *const u8,
            size + size_of::<sys::LV2_Atom>(),
        );
        Self::from_slice(data)
    }

    /// Create a new space from a slice.
    ///
    /// Since everything regarding atoms is 64-bit-aligned, this method panics if the data slice is not 64-bit-aligned.
    pub fn from_slice(data: &'a [u8]) -> Self {
        Space { data: Some(data) }
    }

    /// Try to retrieve a slice of bytes.
    ///
    /// This method basically splits off the lower part of the internal bytes slice and creates a new atom space pointer of the upper part. Since atoms have to be 64-bit-aligned, there might be a padding space that's neither in the lower nor in the upper part.
    pub fn split_raw(self, size: usize) -> Option<(&'a [u8], Self)> {
        let data = self.data?;

        if size > data.len() {
            return None;
        }
        let (lower_space, upper_space) = data.split_at(size);

        // Apply padding.
        let padding = if size % 8 == 0 { 0 } else { 8 - size % 8 };
        let upper_space = if padding <= upper_space.len() {
            let upper_space = upper_space.split_at(padding).1;
            Some(upper_space)
        } else {
            None
        };
        let upper_space = Self { data: upper_space };

        Some((lower_space, upper_space))
    }

    /// Try to retrieve space.
    ///
    /// This method calls [`split_raw`](#method.split_raw) and wraps the returned slice in an atom space. The second space is the space after the first one.
    pub fn split_space(self, size: usize) -> Option<(Self, Self)> {
        self.split_raw(size)
            .map(|(data, rhs)| (Self::from_slice(data), rhs))
    }

    /// Try to retrieve a reference to a sized type.
    ///
    /// This method retrieves a slice of memory using the [`split_raw`](#method.split_raw) method and interprets it as an instance of `T`. Since there is no way to check that the memory is actually a valid instance of `T`, this method is unsafe. The second return value is the space after the instance of `T`.
    pub fn split_type<T>(self) -> Option<(&'a T, Self)>
    where
        T: Unpin + Copy + Send + Sync + Sized + 'static,
    {
        self.split_raw(size_of::<T>())
            .map(|(data, rhs)| (unsafe { &*(data.as_ptr() as *const T) }, rhs))
    }

    /// Try to retrieve the space occupied by an atom.
    ///
    /// This method assumes that the space contains an atom and retrieves the space occupied by the atom, including the atom header. The second return value is the rest of the space behind the atom.
    ///
    /// The difference to [`split_atom_body`](#method.split_atom_body) is that the returned space contains the header of the atom and that the type of the atom is not checked.
    pub fn split_atom(self) -> Option<(Self, Self)> {
        let (header, _) = self.split_type::<sys::LV2_Atom>()?;
        self.split_space(size_of::<sys::LV2_Atom>() + header.size as usize)
    }

    /// Try to retrieve the body of the atom.
    ///
    /// This method retrieves the header of the atom. If the type URID in the header matches the given URID, it returns the body of the atom. If not, it returns `None`. The first space is the body of the atom, the second one is the space behind it.
    ///
    /// The difference to [`split_atom`](#method.split_atom) is that the returned space does not contain the header of the atom and that the type of the atom is checked.
    pub fn split_atom_body<T: ?Sized>(self, urid: URID<T>) -> Option<(Self, Self)> {
        let (header, space) = self.split_type::<sys::LV2_Atom>()?;
        if header.type_ != urid.get() {
            return None;
        }
        space.split_space(header.size as usize)
    }

    /// Create a space from a reference.
    pub fn from_reference<T: ?Sized>(instance: &'a T) -> Self {
        let data = unsafe {
            std::slice::from_raw_parts(instance as *const T as *const u8, size_of_val(instance))
        };
        assert_eq!(data.as_ptr() as usize % 8, 0);
        Space { data: Some(data) }
    }

    /// Concatenate two spaces.
    ///
    /// There are situations where a space is split too often and you might want to reunite these two adjacent spaces. This method checks if the given spaces are adjacent, which means that the left space has to end exactly where the right one begins. In this case, the concatenated space is returned. If this is not the case, this method returns `None`.
    pub fn concat(lhs: Self, rhs: Self) -> Option<Self> {
        let lhs_data = match lhs.data {
            Some(data) => data,
            None => return Some(rhs),
        };
        let rhs_data = match rhs.data {
            Some(data) => data,
            None => return Some(lhs),
        };
        if unsafe { lhs_data.as_ptr().add(lhs_data.len()) } == rhs_data.as_ptr() {
            Some(Self::from_slice(unsafe {
                std::slice::from_raw_parts(lhs_data.as_ptr(), lhs_data.len() + rhs_data.len())
            }))
        } else {
            None
        }
    }

    /// Return the internal slice of the space.
    pub fn data(&self) -> Option<&'a [u8]> {
        self.data
    }

    /// Return a mutable reference to the internal slice of the space.
    pub fn mut_data(&mut self) -> &mut Option<&'a [u8]> {
        &mut self.data
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
    fn allocate(&mut self, size: usize, apply_padding: bool) -> Option<(usize, &'a mut [u8])>;

    /// Try to write data to the internal data slice.
    ///
    /// The method allocates a slice with the [`allocate`](#tymethod.allocate) method and copies the data to the slice.
    fn write_raw(&mut self, data: &[u8], apply_padding: bool) -> Option<&'a mut [u8]> {
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

/// A `MutSpace` that notes the amount of allocated space in an atom header.
pub struct FramedMutSpace<'a, 'b> {
    atom: &'a mut sys::LV2_Atom,
    parent: &'b mut dyn MutSpace<'a>,
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

    /// Create new `FramedMutSpace` to write an atom.
    ///
    /// Simply pass the URID of the atom as an argument.
    pub fn create_atom_frame<'c, A: ?Sized>(
        &'c mut self,
        urid: URID<A>,
    ) -> Option<FramedMutSpace<'a, 'c>> {
        let atom = sys::LV2_Atom {
            size: 0,
            type_: urid.get(),
        };
        let atom: &'a mut sys::LV2_Atom = self.write(&atom, true)?;
        Some(FramedMutSpace { atom, parent: self })
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::space::*;
    use std::mem::{size_of, size_of_val};
    use urid::mapper::*;
    use urid::prelude::*;

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

        let space = Space::from_slice(vector.as_slice());
        let (lower_space, space) = space.split_raw(128).unwrap();
        for i in 0..128 {
            assert_eq!(lower_space[i], i as u8);
        }

        let (integer, _) = space.split_type::<u32>().unwrap();
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
            }
        }

        let space = Space::from_reference(data.as_ref());
        let (atom, _) = space.split_atom().unwrap();
        let (body, _) = atom.split_atom_body(urid).unwrap();
        let body = body.data().unwrap();

        assert_eq!(size_of::<i32>(), size_of_val(body));
        assert_eq!(42, unsafe { *(body.as_ptr() as *const i32) });
    }

    #[test]
    fn test_from_reference() {
        let value: u64 = 0x42424242;
        let space = Space::from_reference(&value);
        assert_eq!(value, *space.split_type::<u64>().unwrap().0);
    }

    #[test]
    fn test_concat() {
        let data: Box<[u64]> = Box::new([0; 64]);
        let space = Space::from_reference(data.as_ref());
        let (lhs, rhs) = space.split_space(8).unwrap();
        let concated_space = Space::concat(lhs, rhs).unwrap();
        assert_eq!(
            space.data().unwrap().as_ptr(),
            concated_space.data().unwrap().as_ptr()
        );
        assert_eq!(
            space.data().unwrap().len(),
            concated_space.data().unwrap().len()
        );
    }

    #[test]
    fn test_root_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let mut frame: RootMutSpace = RootMutSpace::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        let mut test_data: Vec<u8> = vec![0; 24];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }

        match frame.write_raw(test_data.as_slice(), true) {
            Some(written_data) => assert_eq!(test_data.as_slice(), written_data),
            None => panic!("Writing failed!"),
        }

        let test_atom = sys::LV2_Atom { size: 42, type_: 1 };
        let written_atom = (&mut frame as &mut dyn MutSpace)
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
    }

    #[test]
    fn test_framed_mut_space() {
        const MEMORY_SIZE: usize = 256;
        let mut memory: [u64; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let mut root: RootMutSpace = RootMutSpace::new(unsafe {
            std::slice::from_raw_parts_mut(
                (&mut memory).as_mut_ptr() as *mut u8,
                MEMORY_SIZE * size_of::<u64>(),
            )
        });

        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut atom_frame: FramedMutSpace = (&mut root as &mut dyn MutSpace)
            .create_atom_frame(urids.chunk)
            .unwrap();

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
            let mut frame = (&mut root as &mut dyn MutSpace)
                .create_atom_frame(unsafe { URID::<()>::new_unchecked(1) })
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
