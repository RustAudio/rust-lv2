use std::error::Error;
use std::fmt::{Display, Formatter};
use urid::{Uri, URID};

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

#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum AtomWriteError {
    OutOfSpace {
        used: usize,
        capacity: usize,
        requested: usize,
    },
    AllocatorOverflow,
    ResizeFailed,
    CannotUpdateAtomHeader,
    AtomAlreadyWritten,
    RewindError {
        available: usize,
        requested: usize,
    },
    WritingOutOfBounds {
        available: usize,
        requested: usize,
    },
    WritingIllegalState {
        writing_type_uri: &'static Uri,
    },
    AlignmentError(AlignmentError),

    Unknown,
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

    Unknown,
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
