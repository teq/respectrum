use std::{cell::{Cell, RefCell}, collections::HashMap, rc::Rc};
use crate::{
    devs,
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
    next_id: Cell<usize>,
    device_names: RefCell<HashMap<usize, &'static str>>,
}

impl DeviceManager {

    /// Create a new device manager with the given bus and clock
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self {
            bus,
            clock,
            next_id: Cell::new(0),
            device_names: RefCell::new(HashMap::new()),
        }
    }

    fn generate_id(&self) -> usize {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    /// Register a device name for a specific ID
    fn register_name(&self, id: usize, name: &'static str) {
        self.device_names.borrow_mut().insert(id, name);
    }

    /// Get the readable name for a device by its ID
    pub fn get_name(&self, device: &dyn Device) -> Option<&'static str> {
        self.device_names.borrow().get(&device.id()).copied()
    }

    /// Create a new CPU instance
    pub fn create_cpu(&self) -> Rc<devs::Cpu> {
        let cpu = Rc::new(devs::Cpu::new(self.generate_id(), self.bus.clone(), self.clock.clone()));
        self.register_name(cpu.id(), "Z80 CPU");
        cpu
    }

    /// Create a new Dynamic 48k memory instance
    pub fn create_dynamic_48k(&self) -> Rc<devs::mem::Dynamic48k> {
        let memory = Rc::new(devs::mem::Dynamic48k::new(self.generate_id(), self.bus.clone(), self.clock.clone()));
        self.register_name(memory.id(), "Dynamic 48K Memory");
        memory
    }

    /// Create a new bus logger instance
    pub fn create_bus_logger(&self) -> Rc<devs::BusLogger> {
        let logger = Rc::new(devs::BusLogger::new(self.generate_id(), self.bus.clone(), self.clock.clone()));
        self.register_name(logger.id(), "Bus Logger");
        logger
    }
}
