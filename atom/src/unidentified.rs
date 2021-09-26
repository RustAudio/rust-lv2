use crate::space::error::{AtomReadError, AtomWriteError};
use crate::space::AtomSpace;
use crate::{Atom, AtomHandle, AtomHeader};
use urid::URID;

/// An atom of yet unknown type.
///
/// This is used by reading handles that have to return a reference to an atom, but can not check it's type. This struct contains a `Space` containing the header and the body of the atom and can identify/read the atom from it.
#[repr(C)]
pub struct UnidentifiedAtom {
    header: AtomHeader,
}

impl UnidentifiedAtom {
    /// Construct a new unidentified atom.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the given space actually contains both a valid atom header, and a valid corresponding atom body.
    #[inline]
    pub unsafe fn from_space(space: &AtomSpace) -> Result<&Self, AtomReadError> {
        Ok(Self::from_header(space.read().next_value()?))
    }

    /// Construct a new unidentified atom.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the given space actually contains both a valid atom header, and a valid corresponding atom body.
    #[inline]
    pub unsafe fn from_space_mut(space: &mut AtomSpace) -> Result<&mut Self, AtomWriteError> {
        let available = space.bytes_len();

        Ok(Self::from_header_mut(
            space
                .assume_init_slice_mut()
                .get_mut(0)
                .ok_or(AtomWriteError::WritingOutOfBounds {
                    available,
                    requested: ::core::mem::size_of::<AtomHeader>(),
                })?,
        ))
    }

    #[inline]
    pub(crate) unsafe fn from_header(header: &AtomHeader) -> &Self {
        // SAFETY: UnidentifiedAtom is repr(C) and has AtomHeader as its only field, so transmuting between the two is safe.
        &*(header as *const _ as *const _)
    }

    #[inline]
    pub(crate) unsafe fn from_header_mut(header: &mut AtomHeader) -> &mut Self {
        // SAFETY: UnidentifiedAtom is repr(C) and has AtomHeader as its only field, so transmuting between the two is safe.
        &mut *(header as *mut _ as *mut _)
    }

    /// Try to read the atom.
    ///
    /// To identify the atom, its URID and an atom-specific parameter is needed. If the atom was identified, a reading handle is returned.
    pub fn read<A: Atom>(
        &self,
        urid: URID<A>,
    ) -> Result<<A::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        self.header.check_urid(urid)?;

        // SAFETY: the fact that this contains a valid instance of A is checked above.
        unsafe { A::read(self.body()) }
    }

    #[inline]
    pub fn header(&self) -> &AtomHeader {
        &self.header
    }

    #[inline]
    fn body_bytes(&self) -> &[u8] {
        if self.header.size_of_body() == 0 {
            &[]
        } else {
            // SAFETY: This type's constructor ensures the atom's body is valid
            // The edge case of an empty body is also checked above.
            let ptr = unsafe { (self as *const UnidentifiedAtom).add(1) };

            // SAFETY: This type's constructor ensures the atom's body is valid
            unsafe { ::core::slice::from_raw_parts(ptr.cast(), self.header.size_of_body()) }
        }
    }

    #[inline]
    fn body_bytes_mut(&mut self) -> &mut [u8] {
        if self.header.size_of_body() == 0 {
            &mut []
        } else {
            // SAFETY: This type's constructor ensures the atom's body is valid
            // The edge case of an empty body is also checked above.
            let ptr = unsafe { (self as *mut UnidentifiedAtom).add(1) };

            // SAFETY: This type's constructor ensures the atom's body is valid
            unsafe { ::core::slice::from_raw_parts_mut(ptr.cast(), self.header.size_of_body()) }
        }
    }

    #[inline]
    pub fn atom_space(&self) -> &AtomSpace {
        let ptr = self as *const UnidentifiedAtom as *const u8;
        let bytes = unsafe { ::core::slice::from_raw_parts(ptr, self.header.size_of_atom()) };

        // SAFETY: the bytes are necessarily aligned, since they point to the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_unchecked(bytes) }
    }

    #[inline]
    pub fn atom_space_mut(&mut self) -> &mut AtomSpace {
        let ptr = self as *mut UnidentifiedAtom as *mut u8;
        let bytes = unsafe { ::core::slice::from_raw_parts_mut(ptr, self.header.size_of_atom()) };

        // SAFETY: the bytes are necessarily aligned, since they point to the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_mut_unchecked(bytes) }
    }

    #[inline]
    pub fn body(&self) -> &AtomSpace {
        // SAFETY: the bytes are necessarily aligned, since they are right after the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_unchecked(self.body_bytes()) }
    }

    #[inline]
    pub fn body_mut(&mut self) -> &mut AtomSpace {
        // SAFETY: the bytes are necessarily aligned, since they are right after the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_mut_unchecked(self.body_bytes_mut()) }
    }
}
