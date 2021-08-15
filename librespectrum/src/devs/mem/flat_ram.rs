use crate::bus::{Clock, CpuBus, Device, NoReturnTask};

/// Plain 48k memory
pub struct FlatRam<'a> {
    pub bus: &'a CpuBus,
    pub clock: &'a Clock,
    pub memory: Vec<u8>,
}

impl<'a> FlatRam<'a> {

    // Create new memory instance
    pub fn new(bus: &'a CpuBus, clock: &'a Clock) -> Self {
        Self {
            bus, clock,
            memory: Default::default()
        }
    }

}

impl<'a> Device<'a> for FlatRam<'a> {

    fn run(&'a self) -> Box<dyn NoReturnTask + 'a> {
        Box::new(move || {
            loop {
                yield self.clock.rising(4);
            }
        })
    }

}
