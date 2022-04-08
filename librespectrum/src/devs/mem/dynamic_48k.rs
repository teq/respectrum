use std::{cell::Cell, rc::Rc, ops::{Index, Deref}};

use crate::bus::{Clock, CpuBus, Ctls, NoReturnTask};
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
    /// Check if given address is writable (located in RAM)
    fn writable(&self, addr: u16) -> bool {
        addr & 0xc000 != 0 // First 16KB are not writable (ROM)
    }
}

impl Index<u16> for Dynamic48k {
    type Output = Cell<u8>;
    fn index(&self, addr: u16) -> &Self::Output {
        &self.memory[addr as usize]
    }
}

impl Deref for Dynamic48k {
    type Target = Vec<Cell<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl Dynamic48k {

    pub fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(move || {

            loop {

                // Wait for MREQ
                while !self.bus.ctrl.sample().unwrap_or(Ctls::NONE).contains(Ctls::MREQ) {
                    yield self.clock.rising(1);
                }

                let addr = self.bus.addr.sample().unwrap();
                let ctrl = self.bus.ctrl.sample().unwrap();

                // Perform read or write
                if ctrl.contains(Ctls::RD) {
                    let release = self.bus.data.drive_and_release(self[addr].get());
                    yield self.clock.rising(3);
                    release();
                } else if ctrl.contains(Ctls::WR) {
                    let data = self.bus.data.sample().unwrap();
                    if self.writable(addr) {
                        self[addr].set(data);
                    }
                    yield self.clock.rising(2);
                }

            }

        })

    }

}
