use crate::Atom;
use urid::URID;
use crate::space::{AtomSpaceWriter, Space};

use core::mem::size_of_val;
use std::mem::MaybeUninit;

/// A smart pointer that writes atom data to an internal slice.
///
/// The methods provided by this trait are fairly minimalistic. More convenient writing methods are implemented for `dyn MutSpace`.
pub trait AllocateSpace<'a> {
    /// Try to allocate memory on the internal data slice.
    ///
    /// If `apply_padding` is `true`, the method will assure that the allocated memory is 64-bit-aligned. The first return value is the number of padding bytes that has been used and the second return value is a mutable slice referencing the allocated data.
    ///
    /// After the memory has been allocated, the `MutSpace` can not allocate it again. The next allocated slice is directly behind it.
    fn allocate_unaligned(&mut self, size: usize) -> Option<&mut [u8]>;

    fn as_bytes(&self) -> &[u8];
    fn as_bytes_mut(&mut self) -> &mut [u8];
}

impl<'a> AllocateSpace<'a> for &'a mut [u8] {
    #[inline]
    fn allocate_unaligned(&mut self, size: usize) -> Option<&'a mut [u8]> {
        if size > self.len() {
            return None
        }

        let slice: &'a mut [u8] = ::core::mem::replace(self, &mut []); // Lifetime workaround
        let (allocated, remaining) = slice.split_at_mut(size);
        *self = remaining;
        Some(allocated)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self
    }

    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        self
    }
}

#[inline]
pub fn realign<'a, T: 'static, S: AllocateSpace<'a>>(space: &mut S) -> Option<()> {
    let required_padding = Space::<T>::padding_for(space.as_bytes());
    let _ = space.allocate_unaligned(required_padding)?;
    Some(())
}

#[inline]
pub fn allocate<'handle, 'space: 'handle, T: 'static>(space: &'handle mut impl AllocateSpace<'space>, size: usize) -> Option<&'handle mut Space<T>> {
    let required_padding = Space::<T>::padding_for(space.as_bytes());
    let raw = space.allocate_unaligned(size + required_padding)?;

    Space::try_align_from_bytes_mut(raw)
}

#[inline]
pub fn allocate_values<'handle, 'space: 'handle, T: 'static>(space: &'handle mut impl AllocateSpace<'space>, count: usize) -> Option<&'handle mut [MaybeUninit<T>]> {
    let space = allocate(space, count * ::core::mem::size_of::<T>())?;
    Some(space.as_uninit_slice_mut())
}

#[inline]
pub fn init_atom<'handle, 'space: 'handle, A: Atom<'handle, 'space>>(space: &'handle mut impl AllocateSpace<'space>, atom_type: URID<A>, write_parameter: A::WriteParameter) -> Option<A::WriteHandle> {
    let space: AtomSpaceWriter<'handle, 'space> = AtomSpaceWriter::write_new(space, atom_type)?;
    A::init(space, write_parameter)
}

#[inline]
pub fn write_bytes<'handle, 'space: 'handle>(space: &'handle mut impl AllocateSpace<'space>, bytes: &[u8]) -> Option<&'handle mut [u8]> {
    let space = space.allocate_unaligned(bytes.len())?;
    space.copy_from_slice(bytes);
    Some(space)
}

#[inline]
pub fn write_value<'handle, 'space: 'handle, T>(space: &'handle mut impl AllocateSpace<'space>, value: T) -> Option<&'handle mut T>
    where T: Copy + Sized + 'static {
    let space = allocate(space, size_of_val(&value))?;
    // SAFETY: We used size_of_val, so we are sure that the allocated space is exactly big enough for T.
    let space = unsafe { space.as_uninit_mut_unchecked() };
    *space = MaybeUninit::new(value);

    // SAFETY: the MaybeUninit has now been properly initialized.
    Some (unsafe { &mut *(space.as_mut_ptr()) })
}

pub fn write_values<'handle, 'space: 'handle, T>(space: &'handle mut impl AllocateSpace<'space>, values: &[T]) -> Option<&'handle mut [T]>
    where T: Copy + Sized + 'static {
    let space: &mut Space<T> = allocate(space, size_of_val(values))?;
    let space = space.as_uninit_slice_mut();

    for (dst, src) in space.iter_mut().zip(values.iter()) {
        *dst = MaybeUninit::new(*src)
    }

    // SAFETY: Assume init: we just initialized the memory above
    Some(unsafe { &mut *(space as *mut [_] as *mut [T]) })
}

#[cfg(test)]
mod tests {
    use crate::space::{AtomSpace, write_value, init_atom};
    use crate::prelude::Int;
    use urid::URID;

    const INT_URID: URID<Int> = unsafe { URID::new_unchecked(5) };

    #[test]
    fn test_init_atom_lifetimes () {
        assert_eq!(AtomSpace::alignment(), 8);

        let mut space = AtomSpace::boxed(32);
        assert_eq!(space.as_bytes().as_ptr() as usize % 8, 0); // TODO: move this, this is a test for boxed

        let mut cursor: &mut _ = space.as_bytes_mut(); // The pointer that is going to be moved as we keep writing.
        let new_value = write_value(&mut cursor, 42u8).unwrap();

        assert_eq!(42, *new_value);
        assert_eq!(31, cursor.len());

        {
            let int_atom: &mut _ = init_atom(&mut cursor, INT_URID, 69).unwrap();
            assert_eq!(69, *int_atom);
            assert_eq!(12, cursor.len());
        }
        // let new_value = write_value(&mut cursor, 42u8).unwrap();
        /*{
            // Remaining once aligned: 24, with 8 bytes for atom header: 16
            let writer = AtomSpaceWriter::write_new(&mut cursor, INT_URID).unwrap();
            let int_atom = Int::init(writer, 69).unwrap();
            assert_eq!(69, *int_atom);
            assert_eq!(12, cursor.len());
        }*/

        assert_eq!(space.as_bytes(), [
            42, 0, 0, 0, 0, 0, 0, 0,
            4, 0, 0, 0, 5, 0, 0, 0,
            69, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        assert_eq!(32, space.len());
    }
}