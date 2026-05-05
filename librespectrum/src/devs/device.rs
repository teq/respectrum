use std::{cell::{Cell, RefCell}, collections::HashMap, rc::Rc};

use crate::{
    core::{Clock, CpuBus, Identifiable, Identifier, NoReturnTask},
    devs::{BreakpointManager, BusLogger, Cpu, mem::Static48k},
};

pub trait Device: Identifiable {
    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a>;
}

/// Device manager for creating and managing devices in the system
pub struct DeviceManager {
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    breakpoint_manager: Rc<BreakpointManager>,
    next_id: Cell<Identifier>,
    device_names: RefCell<HashMap<Identifier, &'static str>>,
}

impl DeviceManager {

    /// Create a new device manager with the given bus and clock
    pub fn new(bus: &Rc<CpuBus>, clock: &Rc<Clock>, breakpoint_manager: &Rc<BreakpointManager>) -> Self {
        Self {
            bus: Rc::clone(bus),
            clock: Rc::clone(clock),
            breakpoint_manager: Rc::clone(breakpoint_manager),
            next_id: Cell::new(0),
            device_names: RefCell::new(HashMap::new()),
        }
    }

    fn generate_id(&self) -> Identifier {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    /// Register a device name for a specific ID
    fn register_name(&self, id: Identifier, name: &'static str) {
        self.device_names.borrow_mut().insert(id, name);
    }

    /// Get the readable name for a device by its ID or device reference
    pub fn get_name<T: Identifiable>(&self, identifiable: T) -> Option<&'static str> {
        self.device_names.borrow().get(&identifiable.id()).copied()
    }

    /// Create a new CPU instance
    pub fn create_cpu(&self) -> Rc<Cpu> {
        let cpu = Rc::new(Cpu::new(self.generate_id(), &self.bus, &self.clock, &self.breakpoint_manager));
        self.register_name(cpu.id(), "Z80 CPU");
        cpu
    }

    /// Create a new 48k memory instance
    pub fn create_48k_memory(&self) -> Rc<Static48k> {
        let memory = Rc::new(Static48k::new(self.generate_id(), &self.bus, &self.clock, &self.breakpoint_manager));
        self.register_name(memory.id(), "Static 48K Memory");
        memory
    }

    /// Create a new bus logger instance
    pub fn create_bus_logger(&self) -> Rc<BusLogger> {
        let logger = Rc::new(BusLogger::new(self.generate_id(), &self.bus, &self.clock));
        self.register_name(logger.id(), "Bus Logger");
        logger
    }
}
