use std::{
    rc::Rc,
};

use crate::{
    types::*,
    bus::{Device, CpuBus, clock::Task},
};

#[derive(Default)]
struct RegFile {
    af: Word,
    bc: Word,
    de: Word,
    hl: Word,
}

/// Z80 CPU device
pub struct Cpu {
    pri: RegFile,
    alt: RegFile,
    ix: Word,
    iy: Word,
    sp: Word,
    pc: Word,
    bus: Rc<CpuBus>,
}

impl Device for Cpu {

    fn run<'a>(&'a self) -> Box<dyn Task + 'a> {
        Box::new(move || {
            loop {
                yield self.bus.clock.rising(4);
            }
        })
    }

}

impl Cpu {

    /// Create new CPU instance
    pub fn new(bus: Rc<CpuBus>) -> Cpu {
        Cpu {
            pri: Default::default(),
            alt: Default::default(),
            ix: Default::default(),
            iy: Default::default(),
            sp: Default::default(),
            pc: Default::default(),
            bus
        }
    }

}
