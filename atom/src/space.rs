//! Smart pointers with safe atom reading and writing methods.

mod list;
mod space;
mod allocatable;
mod atom;

pub use space::{AtomSpace, Space};
pub use list::{SpaceList, SpaceHead};
pub use allocatable::*;
pub use atom::AtomSpaceWriter;

