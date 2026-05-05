use std::{cell::Cell, rc::Rc};

use crate::{
    core::{Clock, CpuBus, Ctrl, Identifiable, Identifier, NoReturnTask},
    devs::{BreakpointManager, Device},
    yield_break_if, yield_wait
};

use super::Memory;

/// Static 48k memory
#[derive(Default)]
pub struct Static48k {
    id: Identifier,
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    memory: Vec<Cell<u8>>,
    breakpoint_manager: Rc<BreakpointManager>,
}

impl Static48k {

    /// Create new memory instance
    pub fn new(id: Identifier, bus: &Rc<CpuBus>, clock: &Rc<Clock>, breakpoint_manager: &Rc<BreakpointManager>) -> Self {
        Self {
            id,
            bus: Rc::clone(bus),
            clock: Rc::clone(clock),
            memory: vec![Default::default(); usize::pow(2, 16)],
            breakpoint_manager: Rc::clone(breakpoint_manager),
        }
    }

    /// Load data to the memory at the given address
    pub fn load(&self, addr: u16, data: &Vec<u8>) {
        for (index, &byte) in data.iter().enumerate() {
            self.memory[addr as usize + index].set(byte)
        }
    }

}

impl Memory for Static48k {

    fn writable(&self, addr: u16) -> bool {
        addr & 0xc000 != 0 // First 16KB are not writable (ROM)
    }

    fn write(&self, addr: u16, byte: u8) {
        if self.writable(addr) {
            self.memory[addr as usize].set(byte);
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize].get()
    }

}

impl Identifiable for Static48k {
    fn id(&self) -> Identifier { self.id }
}

impl Device for Static48k {

    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(#[coroutine] move || {

            loop {

                let ctrl = self.bus.ctrl.probe().unwrap_or(Ctrl::NONE);
                let mreq = ctrl.contains(Ctrl::MREQ);
                let rd = ctrl.contains(Ctrl::RD);
                let wr = ctrl.contains(Ctrl::WR);

                // Memory read: drive the bus while MREQ+RD are asserted.
                if mreq && rd && !wr {
                    let addr = self.bus.addr.expect();
                    let val = self.read(addr);
                    self.bus.data.drive(self, val);
                    yield_break_if!(self.breakpoint_manager.hits_after_memory_read(addr));
                }

                // Memory write: drive the bus while MREQ+WR are asserted.
                else if mreq && wr && !rd {
                    self.bus.data.release(self);
                    let addr = self.bus.addr.expect();
                    let val = self.bus.data.expect();
                    self.write(addr, val);
                    yield_break_if!(self.breakpoint_manager.hits_after_memory_write(addr));
                }

                // Any non-memory cycle or ambiguous control state.
                else {
                    self.bus.data.release(self);
                }

                yield_wait!(self.clock.rising(1));

            }

        })

    }

}
