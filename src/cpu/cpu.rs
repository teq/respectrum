use std::rc::Rc;

use crate::{
    yield_task,
    types::*,
    bus::{
        self,
        CpuBus,
        Device,
        task::{Task, NoReturnTask}
    },
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

    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(move || {

            let bus = &self.bus;

            loop {

                let byte = yield_task!(self.opcode_read());

                // yield 0;

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

    /// Perform instruction opcode fetch operation
    fn opcode_read<'a>(&'a self) -> impl Task<u8> + 'a {
        let bus = &self.bus;
        move || {
            yield bus.clock.rising(1); // T1 rising
            bus.addr.drive(self.pc.word());
            bus.ctrl.drive(0);
            bus.outs.drive(bus::M1);
            yield bus.clock.falling(1); // T1 falling
            bus.ctrl.drive(bus::MREQ | bus::RD);
            yield bus.clock.falling(1); // T2 falling
            while bus.wait.sample().unwrap_or(false) {
                yield bus.clock.falling(1);
            }
            yield bus.clock.rising(2); // T3 rising
            let byte = bus.data.sample().expect("Expecting data on a bus");
            bus.addr.drive(0); // TODO: use R
            bus.ctrl.drive(0);
            bus.outs.drive(bus::RFSH);
            yield bus.clock.falling(1); // T3 falling
            bus.ctrl.drive(bus::MREQ);
            yield bus.clock.falling(1); // T4 falling
            bus.ctrl.drive(0);
            byte
        }
    }

}
