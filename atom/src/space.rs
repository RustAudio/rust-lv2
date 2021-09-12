//! Smart pointers with safe atom reading and writing methods.

mod allocatable;
mod atom_writer;
mod cursor;
mod error;
pub mod reader;
mod space;
mod vec;

pub use allocatable::*;
pub use atom_writer::{AtomSpaceWriter, AtomSpaceWriterHandle};
pub use cursor::SpaceCursor;
pub use error::AtomError;
pub use space::{AlignedSpace, AtomSpace};
pub use vec::{VecSpace, VecSpaceCursor};
