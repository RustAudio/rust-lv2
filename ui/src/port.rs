use lv2_atom as atom;

use atom::prelude::*;
use urid::*;

use crate::space::*;

/// Trait for an UIPort
///
/// UIPorts are either Control Ports or Atom Ports. The trait defines
/// the interface to the host. The trait functions are usually not
/// relevant for the plugin UI developer. When developing a plugin UI
/// the actual port struct implementations are the ones to be used.
///
pub trait UIPort {
    /// The index of the port
    fn index(&self) -> u32;

    /// The protocol of the port (0 for Control ports or the URID for Atom ports)
    fn protocol(&self) -> u32;

    /// The size to the data transmitted
    fn size(&self) -> usize;

    /// The pointer to the data transmitted
    fn data(&self) -> *const std::ffi::c_void;
}

/// A UI port for a Control Port
pub struct UIControlPort {
    value: f32,
    changed: bool,
    index: u32,
}

impl UIControlPort {
    /// Instantiates an UIControlPort.
    ///
    /// Not to be called manually
    pub fn new(index: u32) -> Self {
        UIControlPort {
            value: 0.0,
            changed: false,
            index,
        }
    }

    /// Sets the value of the port.
    ///
    /// Can be used to communicate a change of the value to the Plugin
    pub fn set_value(&mut self, v: f32) {
        self.value = v;
        self.changed = true;
    }

    /// Returns the changed value if it has been changed, otherwise None.
    ///
    pub fn changed_value(&mut self) -> Option<f32> {
        match self.changed {
            false => None,
            true => {
                self.changed = false;
                Some(self.value)
            }
        }
    }
}

impl UIPort for UIControlPort {
    fn index(&self) -> u32 {
        self.index
    }
    fn protocol(&self) -> u32 {
        0
    }
    fn size(&self) -> usize {
        std::mem::size_of::<f32>()
    }
    fn data(&self) -> *const std::ffi::c_void {
        &self.value as *const f32 as *const std::ffi::c_void
    }
}

/// UI Port for a LV2 Atom port
pub struct UIAtomPort {
    space_to_plugin: SelfAllocatingSpace,
    space_to_ui: SelfAllocatingSpace,
    urid: URID<atom::uris::EventTransfer>,
    index: u32,
}

impl UIAtomPort {
    /// Instantiates an UIAtomPort.
    ///
    /// Not to be called manually
    pub fn new(urid: URID<atom::uris::EventTransfer>, index: u32) -> UIAtomPort {
        UIAtomPort {
            space_to_plugin: SelfAllocatingSpace::new(),
            space_to_ui: SelfAllocatingSpace::new(),
            urid,
            index,
        }
    }

    /// Reads an atom from an UI Atom port
    ///
    /// See `lv2_atom` for details
    pub fn read<'a, A: atom::Atom<'a, 'a>>(
        &'a mut self,
        urid: URID<A>,
        parameter: A::ReadParameter,
    ) -> Option<A::ReadHandle> {
        A::read(self.space_to_ui.take()?.split_atom_body(urid)?.0, parameter)
    }

    /// Initiates atom writing to an UI Atom port
    ///
    /// See `lv2_atom` for details
    pub fn init<'a, A: atom::Atom<'a, 'a>>(
        &'a mut self,
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        self.space_to_plugin = SelfAllocatingSpace::new();
        (&mut self.space_to_plugin as &mut dyn MutSpace).init(urid, parameter)
    }

    pub(crate) unsafe fn put_buffer(
        &mut self,
        buffer: std::ptr::NonNull<std::ffi::c_void>,
        size: usize,
    ) {
        self.space_to_ui.put_buffer(buffer, size);
    }

    pub(crate) fn urid(&mut self) -> u32 {
        self.urid.get()
    }
}

impl UIPort for UIAtomPort {
    fn index(&self) -> u32 {
        self.index
    }
    fn protocol(&self) -> u32 {
        self.urid.get()
    }
    fn size(&self) -> usize {
        self.space_to_plugin.len()
    }
    fn data(&self) -> *const std::ffi::c_void {
        self.space_to_plugin.as_ptr()
    }
}

/// Trait for a UIPort collection
pub trait UIPortCollection: Sized {
    fn port_event(
        &mut self,
        port_index: u32,
        buffer_size: u32,
        format: u32,
        buffer: *const std::ffi::c_void,
    ) {
        match format {
            0 => {
                let value: f32 = unsafe { *(buffer as *const f32) };
                match self.map_control_port(port_index) {
                    Some(ref mut port) => port.set_value(value),
                    None => eprintln!("unknown control port: {}", port_index),
                }
            }
            urid => match self.map_atom_port(port_index) {
                Some(ref mut port) => {
                    if port.urid() == urid {
                        if let Some(pointer) =
                            std::ptr::NonNull::new(buffer as *mut std::ffi::c_void)
                        {
                            unsafe {
                                port.put_buffer(pointer, buffer_size as usize);
                            }
                        }
                    } else {
                        eprintln!("urids of port {} don't match", port_index);
                    }
                }
                None => eprintln!("unknown atom port: {}", port_index),
            },
        }
    }

    fn map_control_port(&mut self, port_index: u32) -> Option<&mut UIControlPort>;

    fn map_atom_port(&mut self, port_index: u32) -> Option<&mut UIAtomPort>;
}
