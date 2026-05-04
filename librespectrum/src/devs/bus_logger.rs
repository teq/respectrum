use std::{cell::RefCell, rc::Rc};

use crate::{
    core::{Clock, CpuBus, Ctrl, Identifiable, Identifier, NoReturnTask, RingBuff},
    devs::Device, yield_wait,
};

#[derive(Default, Clone, Copy)]
pub struct BusState {
    pub htcyc: u64,
    pub addr: Option<(Identifier, u16)>,
    pub data: Option<(Identifier, u8)>,
    pub ctrl: Option<(Identifier, Ctrl)>,
    pub m1: Option<(Identifier, bool)>,
    pub busak: Option<(Identifier, bool)>,
    pub halt: Option<(Identifier, bool)>,
    pub wait: Option<(Identifier, bool)>,
    pub int: Option<(Identifier, bool)>,
    pub nmi: Option<(Identifier, bool)>,
    pub reset: Option<(Identifier, bool)>,
    pub busrq: Option<(Identifier, bool)>,
}

/// CPU bus logger
pub struct BusLogger {
    id: Identifier,
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    pub readings: RefCell<RingBuff<BusState, 64>>,
}

impl BusLogger {
    pub fn new(id: Identifier, bus: &Rc<CpuBus>, clock: &Rc<Clock>) -> Self {
        Self {
            id,
            bus: Rc::clone(bus),
            clock: Rc::clone(clock),
            readings: RefCell::new(RingBuff::new())
        }
    }
}

impl Identifiable for BusLogger {
    fn id(&self) -> Identifier { self.id }
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

                yield_wait!(1); // Advance to the next T-cycle edge (rising or falling)

            }

        })

    }

}
