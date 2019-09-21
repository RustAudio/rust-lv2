use crate::space::*;
use core::port::PortType;
use std::ffi::c_void;
use std::ptr::NonNull;
use urid::URID;

/// A handle to read atoms from a port.
///
/// If you add an [`AtomPort`](struct.AtomPort.html) to your ports struct, you will receive an instance of this struct to read atoms.
pub struct PortReader<'a> {
    space: Space<'a>,
}

impl<'a> PortReader<'a> {
    /// Create a new port reader.
    fn new(space: Space<'a>) -> Self {
        Self { space }
    }

    /// Read an atom.
    ///
    /// In order to identify the atom, the reader needs to know it's URID. Also, some atoms require a parameter. However, you can simply pass `()` in most cases.
    ///
    /// This method returns `None` if the atom is malformed or simply isn't of the specified type.
    pub fn read<'b, A: crate::Atom<'a, 'b>>(
        &'b self,
        urid: URID<A>,
        parameter: A::ReadParameter,
    ) -> Option<A::ReadHandle> {
        A::read(self.space.split_atom_body(urid)?.0, parameter)
    }
}

/// A handle to write atoms into a port.
///
/// If you add an [`AtomPort`](struct.AtomPort.html) to your ports struct, you will receive an instance of this struct to write atoms.
pub struct PortWriter<'a> {
    space: RootMutSpace<'a>,
    has_been_written: bool,
}

impl<'a> PortWriter<'a> {
    /// Create a new port writer.
    fn new(space: RootMutSpace<'a>) -> Self {
        Self {
            space,
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
    pub fn write<'b, A: crate::Atom<'a, 'b>>(
        &'b mut self,
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        if !self.has_been_written {
            self.has_been_written = true;
            let frame = (&mut self.space as &mut dyn MutSpace).create_atom_frame(urid)?;
            A::write(frame, parameter)
        } else {
            None
        }
    }
}

/// The port type for Atom IO.
///
/// Simply insert it into your ports struct:
///
/// ```
/// extern crate lv2_core as core;
/// extern crate lv2_atom as atom;
///
/// use core::prelude::*;
/// use atom::port::*;
///
/// #[derive(PortContainer)]
/// struct AtomIO {
///     atoms_in: InputPort<AtomPort>,
///     atoms_out: OutputPort<AtomPort>,
/// }
/// ```
pub struct AtomPort;

impl PortType for AtomPort {
    type InputPortType = PortReader<'static>;
    type OutputPortType = PortWriter<'static>;

    unsafe fn input_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> PortReader<'static> {
        let space = Space::from_atom(pointer.cast().as_ref());
        PortReader::new(space)
    }

    unsafe fn output_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> PortWriter<'static> {
        let space = RootMutSpace::from_atom(pointer.cast().as_mut());
        PortWriter::new(space)
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::*;
    use crate::port::*;
    use crate::scalar::*;
    use crate::*;
    use std::mem::size_of;
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    #[test]
    fn test_atom_port() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing a chunk to indicate the size of the space.
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.chunk)
                .unwrap();
            let mut writer = Chunk::write(frame, ()).unwrap();
            writer.allocate(256 - size_of::<sys::LV2_Atom>()).unwrap();
        }

        // Getting a writer with the port.
        {
            let mut writer =
                unsafe { AtomPort::output_from_raw(NonNull::from(raw_space.as_mut()).cast(), 0) };
            writer.write::<Int>(urids.int, 42).unwrap();
        }

        // Reading
        {
            let reader =
                unsafe { AtomPort::input_from_raw(NonNull::from(raw_space.as_mut()).cast(), 0) };
            assert_eq!(reader.read::<Int>(urids.int, ()).unwrap(), 42);
        }
    }
}