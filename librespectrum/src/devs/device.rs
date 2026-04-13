use std::rc::Rc;
use crate::{
    bus::{NoReturnTask, CpuBus, Clock},
    misc::Identifiable,
};

pub trait Device: Identifiable {
    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a>;
}

/// Device manager for creating and managing devices in the system
pub struct DeviceManager {
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
}

impl DeviceManager {
    /// Create a new device manager with the given bus and clock
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self { bus, clock }
    }

    /// Create a new CPU instance
    pub fn create_cpu(&self) -> Rc<crate::devs::Cpu> {
        Rc::new(crate::devs::Cpu::new(self.bus.clone(), self.clock.clone()))
    }

    /// Create a new Dynamic 48k memory instance
    pub fn create_dynamic_48k(&self) -> Rc<crate::devs::mem::Dynamic48k> {
        Rc::new(crate::devs::mem::Dynamic48k::new(self.bus.clone(), self.clock.clone()))
    }

    /// Create a new bus logger instance
    pub fn create_bus_logger(&self) -> Rc<crate::devs::BusLogger> {
        Rc::new(crate::devs::BusLogger::new(self.bus.clone(), self.clock.clone()))
    }
}
