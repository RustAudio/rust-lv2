//! Integration for plugin ports.
//!
//! This module contains a `PortType` for plugin ports that supports atom IO. This will most common
//! way to use atoms and is also used in most examples.
//!
//! # Example
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
//!     // Read an integer from the port and print it.
//!     println!("My input is: {}", ports.input.read(urids.int, ()).unwrap());
//!     // Write the integer `42` to the port.
//!     ports.output.init(urids.int, 42).unwrap();
//! }
//! ```
use crate::space::*;
use lv2_core::port::PortType;
use std::ffi::c_void;
use std::ptr::NonNull;
use urid::URID;

/// A handle to read atoms from a port.
///
/// If you add an [`AtomPort`](struct.AtomPort.html) to your ports struct, you will receive an instance of this struct to read atoms.
pub struct PortReader<'space> {
    space: &'space Space,
}

impl<'space> PortReader<'space> {
    /// Create a new port reader.
    fn new(space: &'space Space) -> Self {
        Self { space }
    }

    /// Read an atom.
    ///
    /// In order to identify the atom, the reader needs to know it's URID. Also, some atoms require a parameter. However, you can simply pass `()` in most cases.
    ///
    /// This method returns `None` if the atom is malformed or simply isn't of the specified type.
    pub fn read<'handle, A: crate::Atom<'handle, 'space>>(
        &self,
        urid: URID<A>,
        parameter: A::ReadParameter,
    ) -> Option<A::ReadHandle> {
        // SAFETY: The port's space has been initialized by the host
        unsafe { A::read(self.space.split_atom_body(urid)?.0, parameter) }
    }
}

/// A handle to write atoms into a port.
///
/// If you add an [`AtomPort`](struct.AtomPort.html) to your ports struct, you will receive an instance of this struct to write atoms.
pub struct PortWriter<'a> {
    space: SpaceCursor<'a>,
    has_been_written: bool,
}

impl<'a> PortWriter<'a> {
    /// Create a new port writer.
    fn new(space: &'a mut Space) -> Self {
        Self {
            space: SpaceCursor::new(space.as_bytes_mut()),
            has_been_written: false,
        }
    }

    /// Write an atom.
    ///
    /// In order to write an atom to a port, you need to pass the URID of the atom and an atom-specific parameter.
    ///
    /// Please note that you can call this method once only, because any atoms written behind the first one will not be identified.
    ///
    /// This method returns `None` if the space of the port isn't big enough or if the method was called multiple times.
    pub fn init<'b, 'read, 'write, A: crate::Atom<'read, 'write>>(
        &'b mut self, // SAFETY: 'write should be :'a , but for now we have to return 'static arbitrary lifetimes.
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        if !self.has_been_written {
            self.has_been_written = true;
            // SAFETY: Nope. That's super unsound, but we need it because ports are 'static right now.
            let space: &'write mut SpaceCursor<'write> = unsafe {
                ::core::mem::transmute::<_, &'write mut SpaceCursor<'write>>(&mut self.space)
            };
            crate::space::init_atom(space, urid, parameter)
        } else {
            None
        }
    }
}

/// The port type for Atom IO.
///
/// Port types should not include `Port`, but in this case it is needed since it would collide with the `Atom` trait. Therefore, this port type is named `AtomPort`.
///
/// [See also the module documentation.](index.html)
pub struct AtomPort;

impl PortType for AtomPort {
    type InputPortType = PortReader<'static>;
    type OutputPortType = PortWriter<'static>;

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> PortReader<'static> {
        let space = Space::from_atom(*pointer.cast().as_ref());
        PortReader::new(space)
    }

    #[inline]
    unsafe fn output_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> PortWriter<'static> {
        let space = Space::from_atom_mut(&mut *pointer.cast().as_ptr());
        PortWriter::new(space)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use lv2_core::prelude::*;
    use std::mem::size_of;
    use std::ptr::NonNull;
    use urid::*;

    #[test]
    fn test_atom_port() {
        let map = HashURIDMapper::new();
        let urids = AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = AtomSpace::boxed(256);

        // writing a chunk to indicate the size of the space.
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = crate::space::init_atom(&mut space, urids.chunk, ()).unwrap();
            writer.allocate(256 - size_of::<sys::LV2_Atom>()).unwrap();
        }

        // Getting a writer with the port.
        {
            let mut writer = unsafe {
                AtomPort::output_from_raw(NonNull::from(raw_space.as_bytes_mut()).cast(), 0)
            };
            writer.init::<Int>(urids.int, 42).unwrap();
        }

        // Reading
        {
            let reader = unsafe {
                AtomPort::input_from_raw(NonNull::from(raw_space.as_bytes_mut()).cast(), 0)
            };
            assert_eq!(reader.read::<Int>(urids.int, ()).unwrap(), 42);
        }
    }
}
