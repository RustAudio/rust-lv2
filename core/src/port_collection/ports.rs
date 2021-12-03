use crate::port::{InPlacePortType, InputPort, OutputPort};
use crate::port_collection::{PortCollection, PortConnections};
use std::ffi::c_void;

pub struct Ports<Ins, Outs> {
    inputs: Ins,
    outputs: Outs,
}

impl<Ins: PortCollection, Outs: PortCollection> Ports<Ins, Outs> {
    pub fn inputs(&self) -> &Ins {
        &self.inputs
    }

    pub fn outputs(&mut self) -> &mut Outs {
        &mut self.outputs
    }

    pub fn zip<
        'getter,
        'me: 'getter,
        I: Sized + 'static,
        T: InPlacePortType<InputOutput = (&'static [I], &'static [I])> + 'me,
    >(
        &'me mut self,
        get_ports: impl FnOnce(
                &'getter mut Ins,
                &'getter mut Outs,
            ) -> (&'getter mut InputPort<T>, &'getter mut OutputPort<T>)
            + 'getter,
    ) -> impl Iterator<Item = (&'me I, &'me I)>
    where
        Ins: 'getter,
        Outs: 'getter,
    {
        let (in_port, out_port) = get_ports(&mut self.inputs, &mut self.outputs);
        let (input, output) = T::from_ports(in_port, out_port);
        input.iter().zip(output)
    }
}

pub struct PortsConnections<Ins, Outs> {
    inputs: Ins,
    outputs: Outs,
}

impl<Ins: PortConnections, Outs: PortConnections> PortConnections for PortsConnections<Ins, Outs> {
    const SIZE: usize = Ins::SIZE + Outs::SIZE;

    #[inline]
    fn new() -> Self {
        Self {
            inputs: Ins::new(),
            outputs: Outs::new(),
        }
    }

    #[inline]
    fn set_connection(&mut self, index: u32) -> Option<&mut *mut c_void> {
        let ins_start: u32 = 0;
        let ins_end: u32 = ins_start + Ins::SIZE as u32 - 1;

        let outs_start: u32 = ins_end + 1;
        let outs_end: u32 = outs_start + Outs::SIZE as u32 - 1;

        if index >= ins_start && index < ins_end {
            self.inputs.set_connection(index - ins_start)
        } else if index >= outs_start && index < outs_end {
            self.outputs.set_connection(index - outs_start)
        } else {
            None
        }
    }
}

impl<Ins: PortCollection, Outs: PortCollection> PortCollection for Ports<Ins, Outs> {
    type Connections = PortsConnections<Ins::Connections, Outs::Connections>;

    unsafe fn from_connections(cache: &Self::Connections, sample_count: u32) -> Option<Self> {
        todo!()
    }
}
