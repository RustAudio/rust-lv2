use std::error::Error;
use std::fmt::{Display, Formatter};
use urid::{Uri, URID};

/// A Helper struct to store data about a type for alignment error messages
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TypeData {
    name: &'static str,
    size: usize,
    align: usize,
}

impl TypeData {
    pub(crate) fn of<T: 'static>() -> Self {
        Self {
            name: core::any::type_name::<T>(),
            size: core::mem::size_of::<T>(),
            align: core::mem::align_of::<T>(),
        }
    }
}

impl Display for TypeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (size: {}, align: {})",
            self.name, self.size, self.align
        )
    }
}

/// The actual, currently private, alignment error
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum AlignmentErrorInner {
    CannotComputeAlignment {
        type_id: TypeData,
        ptr: *const u8,
    },
    UnalignedBuffer {
        type_id: TypeData,
        ptr: *const u8,
    },
    NotEnoughSpaceToRealign {
        type_id: TypeData,
        ptr: *const u8,
        required_padding: usize,
        available_size: usize,
    },
}

/// An alignment error, returned by [`AlignedSpace`].
///
/// This error occurs when a byte buffer is unaligned, or could not be aligned.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlignmentError(pub(crate) AlignmentErrorInner);

impl From<AlignmentError> for AtomWriteError {
    #[inline]
    fn from(error: AlignmentError) -> Self {
        AtomWriteError::AlignmentError(error)
    }
}

impl From<AlignmentError> for AtomReadError {
    #[inline]
    fn from(error: AlignmentError) -> Self {
        AtomReadError::AlignmentError(error)
    }
}

/// Errors that can occur while writing atoms to a byte buffer.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum AtomWriteError {
    /// A write operation could not proceed because there is not enough space in the allocatable buffer.
    OutOfSpace {
        /// The amount currently used in the buffer, in bytes.
        used: usize,
        /// The total capacity of the buffer, in bytes.
        capacity: usize,
        /// The requested amount of bytes to be allocated in the buffer, in bytes.
        ///
        /// If this error occurred, most likely this is higher than the remaining amount of bytes available.
        requested: usize,
    },
    /// An allocator tried to be rewound beyond the amount of already allocated bytes
    RewindBeyondAllocated {
        /// The amount of already allocated bytes
        allocated: usize,
        /// The amount of bytes requested to be rewound
        ///
        /// If this error occurred, most likely this is higher than the amount of allocated bytes
        requested: usize,
    },
    /// A write operation tried to occur outside of the buffer's bounds
    WritingOutOfBounds {
        /// The amount of available bytes in the buffer
        available: usize,
        /// The requested amount of bytes
        requested: usize,
    },
    WritingIllegalState {
        writing_type_uri: &'static Uri,
    },
    AlignmentError(AlignmentError),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum AtomReadError {
    InvalidAtomUrid {
        expected_uri: &'static Uri,
        expected_urid: URID,
        found_urid: URID,
    },
    InvalidUrid {
        expected_uri: &'static Uri,
        expected_urid: URID,
        found_urid: u32,
    },
    ReadingOutOfBounds {
        available: usize,
        requested: usize,
    },
    InvalidAtomValue {
        reading_type_uri: &'static Uri,
    },
    AlignmentError(AlignmentError),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AtomError {
    ReadError(AtomReadError),
    WriteError(AtomWriteError),
}

impl Display for AtomError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for AtomError {}

impl From<AtomReadError> for AtomError {
    #[inline]
    fn from(error: AtomReadError) -> Self {
        AtomError::ReadError(error)
    }
}

impl From<AtomWriteError> for AtomError {
    #[inline]
    fn from(error: AtomWriteError) -> Self {
        AtomError::WriteError(error)
    }
}
