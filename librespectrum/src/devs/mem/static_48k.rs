use std::{cell::Cell, rc::Rc};

use crate::{
    core::{Clock, CpuBus, Ctrl, Identifiable, NoReturnTask, Task},
    devs::Device,
    yield_break, yield_from, yield_wait
};

use super::{Memory, MemoryBreakpoint};

/// Static 48k memory
#[derive(Default)]
pub struct Static48k {
    id: usize,
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    memory: Vec<Cell<u8>>,
    breakpoint: Cell<Option<MemoryBreakpoint>>,
}

impl Static48k {

    /// Create new memory instance
    pub fn new(id: usize, bus: &Rc<CpuBus>, clock: &Rc<Clock>) -> Self {
        Self {
            id,
            bus: Rc::clone(bus),
            clock: Rc::clone(clock),
            memory: vec![Default::default(); usize::pow(2, 16)],
            ..Default::default()
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
    fn id(&self) -> usize { self.id }
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
                    yield_from!(self.read_breakpoints(addr));
                    yield_from!(self.access_breakpoints(addr));
                    self.bus.data.drive(self, val);
                }

                // Memory write: drive the bus while MREQ+WR are asserted.
                else if mreq && wr && !rd {
                    self.bus.data.release(self);
                    let addr = self.bus.addr.expect();
                    let val = self.bus.data.expect();
                    self.write(addr, val);
                    yield_from!(self.write_breakpoints(addr));
                    yield_from!(self.access_breakpoints(addr));
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

impl Static48k {

    fn access_breakpoints<'a>(&'a self, addr: u16) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let breakpoint = self.breakpoint.take();
            match breakpoint {
                Some(MemoryBreakpoint::Access(break_addr)) if break_addr == addr => yield_break!(),
                Some(MemoryBreakpoint::AccessRange(start, end)) if (start..=end).contains(&addr) => yield_break!(),
                _ => {}
            }
            self.breakpoint.set(breakpoint);
        }
    }

    fn write_breakpoints<'a>(&'a self, addr: u16) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let breakpoint = self.breakpoint.take();
            match breakpoint {
                Some(MemoryBreakpoint::Write(break_addr)) if break_addr == addr => yield_break!(),
                _ => {}
            }
            self.breakpoint.set(breakpoint);
        }
    }

    fn read_breakpoints<'a>(&'a self, addr: u16) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let breakpoint = self.breakpoint.take();
            match breakpoint {
                Some(MemoryBreakpoint::Read(break_addr)) if break_addr == addr => yield_break!(),
                _ => {}
            }
            self.breakpoint.set(breakpoint);
        }
    }

}
