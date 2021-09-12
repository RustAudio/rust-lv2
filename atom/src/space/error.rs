use std::error::Error;
use std::fmt::{Display, Formatter};
use urid::{Uri, URID};

#[derive(Debug, Clone)]
pub enum AtomError {
    OutOfSpace {
        used: usize,
        capacity: usize,
        requested: usize,
    },
    CannotComputeAlignment {
        ptr: *const u8,
    },
    AllocatorOverflow,
    ResizeFailed,
    CannotUpdateAtomHeader,
    AtomAlreadyWritten,

    // Reading
    InvalidAtomUrid {
        expected_uri: &'static Uri,
        expected_urid: URID,
        found_urid: URID,
    },
    ReadingOutOfBounds {
        capacity: usize,
        requested: usize,
    },
    InvalidAtomValue {
        reading_type_uri: &'static Uri,
    },

    Unknown,
}

impl Display for AtomError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for AtomError {}
