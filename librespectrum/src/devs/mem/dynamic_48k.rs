use std::{cell::Cell, rc::Rc};

use crate::{
    bus::{Clock, CpuBus, Ctrl, NoReturnTask},
    devs::Device, misc::Identifiable,
};
use super::Memory;

/// Standard dynamic 48k memory
pub struct Dynamic48k {
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    memory: Vec<Cell<u8>>,
}

impl Dynamic48k {

    /// Create new memory instance
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self {
            bus, clock,
            memory: vec![Default::default(); usize::pow(2, 16)]
        }
    }

    /// Load data to the memory at the given address
    pub fn load(&self, addr: u16, data: &Vec<u8>) {
        for (index, &byte) in data.iter().enumerate() {
            self.memory[addr as usize + index].set(byte)
        }
    }

}

impl Memory for Dynamic48k {

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

impl Identifiable for Dynamic48k {

    fn id(&self) -> u32 { 2 }

}

impl Device for Dynamic48k {

    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(#[coroutine] move || {

            loop {

                // Wait for MREQ
                while !self.bus.ctrl.probe().unwrap_or(Ctrl::NONE).contains(Ctrl::MREQ) {
                    yield self.clock.rising(1);
                }

                let addr = self.bus.addr.expect();
                let ctrl = self.bus.ctrl.expect();

                // Perform read or write
                if ctrl.contains(Ctrl::RD) {
                    self.bus.data.drive(self, self.read(addr));
                    yield self.clock.rising(3);
                    self.bus.data.release(self);
                } else if ctrl.contains(Ctrl::WR) {
                    let byte = self.bus.data.expect();
                    self.write(addr, byte);
                    yield self.clock.rising(2);
                }

            }

        })

    }

}
