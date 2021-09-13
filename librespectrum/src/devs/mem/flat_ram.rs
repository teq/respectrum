use std::rc::Rc;

use crate::bus::{Clock, CpuBus, Device, NoReturnTask};

/// Plain 48k memory
pub struct FlatRam {
    pub bus: Rc<CpuBus>,
    pub clock: Rc<Clock>,
    pub memory: Vec<u8>,
}

impl FlatRam {

    // Create new memory instance
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self {
            bus, clock,
            memory: Default::default()
        }
    }

}

impl Device for FlatRam {

    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {
        Box::new(move || {
            loop {
                yield self.clock.rising(4);
            }
        })
    }

}
