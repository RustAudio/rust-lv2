//! Smart pointers with safe atom reading and writing methods.

mod allocatable;
mod atom_writer;
mod cursor;
pub mod reader;
mod space;
mod vec;

pub use allocatable::*;
pub use atom_writer::AtomSpaceWriter;
pub use cursor::SpaceCursor;
pub use space::{AtomSpace, Space};
pub use vec::{VecSpace, VecSpaceCursor};
