use std::{rc::Rc, cell::RefCell};

use crate::{
    bus::{Clock, CpuBus, NoReturnTask, Ctrl},
    devs::Device, misc::{Identifiable, RingBuff},
};

#[derive(Default, Clone, Copy)]
pub struct BusState {
    pub htcyc: u64,
    pub addr: Option<(usize, u16)>,
    pub data: Option<(usize, u8)>,
    pub ctrl: Option<(usize, Ctrl)>,
    pub m1: Option<(usize, bool)>,
    pub busak: Option<(usize, bool)>,
    pub halt: Option<(usize, bool)>,
    pub wait: Option<(usize, bool)>,
    pub int: Option<(usize, bool)>,
    pub nmi: Option<(usize, bool)>,
    pub reset: Option<(usize, bool)>,
    pub busrq: Option<(usize, bool)>,
}

/// CPU bus logger
pub struct BusLogger {
    id: usize,
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    pub readings: RefCell<RingBuff<BusState, 64>>,
}

impl BusLogger {
    pub fn new(id: usize, bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self { id, bus, clock, readings: RefCell::new(RingBuff::new()) }
    }
}

impl Identifiable for BusLogger {
    fn id(&self) -> usize { self.id }
}

impl Device for BusLogger {

    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(#[coroutine] move || {

            loop {

                let state = BusState {
                    htcyc: self.clock.get(),
                    addr: self.bus.addr.state(),
                    data: self.bus.data.state(),
                    ctrl: self.bus.ctrl.state(),
                    m1: self.bus.m1.state(),
                    busak: self.bus.busak.state(),
                    halt: self.bus.halt.state(),
                    wait: self.bus.wait.state(),
                    int: self.bus.int.state(),
                    nmi: self.bus.nmi.state(),
                    reset: self.bus.reset.state(),
                    busrq: self.bus.busrq.state(),
                };

                self.readings.borrow_mut().push(state);

                yield 1; // Advance to the next T-cycle edge (rising or falling)

            }

        })

    }

}
