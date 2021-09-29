//! A collection of tools to assist reading and writing custom Atom types in Atom byte buffers (referred as **Spaces**).

mod aligned;
mod allocator;
mod atom_writer;
mod cursor;
pub mod error;
mod reader;
mod terminated;
mod vec;

pub use aligned::{AlignedSpace, AtomSpace};
pub use allocator::*;
pub use atom_writer::AtomWriter;
pub use cursor::SpaceCursor;
pub use reader::SpaceReader;
pub use terminated::Terminated;
pub use vec::{VecSpace, VecSpaceCursor};
