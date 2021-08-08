//! Smart pointers with safe atom reading and writing methods.

mod list;
mod space;
mod allocatable;
mod atom_writer;
mod boxed;
mod vec;

pub use space::{AtomSpace, Space};
pub use list::{SpaceList, SpaceHead};
pub use allocatable::*;
pub use atom_writer::AtomSpaceWriter;
pub use vec::{VecSpace, VecSpaceCursor};

