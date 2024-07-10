use std::{rc::Rc, cell::RefCell};

use crate::{
    bus::{Clock, CpuBus, NoReturnTask, Outs, Ctrl},
    devs::Device, misc::{Identifiable, RingBuff},
};

#[derive(Default, Clone, Copy)]
pub struct BusState {
    pub htcyc: u64,
    pub addr: Option<u16>,
    pub data: Option<u8>,
    pub ctrl: Option<Ctrl>,
    pub outs: Option<Outs>,
    pub wait: Option<bool>,
    pub int: Option<bool>,
    pub nmi: Option<bool>,
    pub reset: Option<bool>,
    pub busrq: Option<bool>,
}

/// CPU bus logger
pub struct BusLogger {
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    pub readings: RefCell<RingBuff<BusState, 64>>,
}

impl BusLogger {
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self { bus, clock, readings: RefCell::new(RingBuff::new()) }
    }
}

impl Identifiable for BusLogger {
    fn id(&self) -> u32 { 10 }
}

impl Device for BusLogger {

    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(#[coroutine] move || {

            loop {

                let state = BusState {
                    htcyc: self.clock.get(),
                    addr: self.bus.addr.probe(),
                    data: self.bus.data.probe(),
                    ctrl: self.bus.ctrl.probe(),
                    outs: self.bus.outs.probe(),
                    wait: self.bus.wait.probe(),
                    int: self.bus.int.probe(),
                    nmi: self.bus.nmi.probe(),
                    reset: self.bus.reset.probe(),
                    busrq: self.bus.busrq.probe(),
                };

                self.readings.borrow_mut().push(state);

                yield 1;

            }

        })

    }

}
