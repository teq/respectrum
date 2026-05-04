
use std::cell::RefCell;

use super::{CpuState};

pub enum Breakpoint {
    BeforeOpcodeRead { once: bool, condition: Option<Box<dyn Fn(&CpuState) -> bool>> },
    Access(u16), // Break on access to given address
    AccessRange(u16, u16), // Break on access to given address range (inclusive)
    Write(u16), // Break on write to given address
    Read(u16), // Break on read from given address
}

#[derive(Default)]
pub struct BreakpointManager {
    breakpoints: RefCell<Vec<Breakpoint>>
}

impl BreakpointManager {

    pub fn push(&self, breakpoint: Breakpoint) {
        self.breakpoints.borrow_mut().push(breakpoint);
    }

    pub fn clear(&self) {
        self.breakpoints.borrow_mut().clear();
    }

    pub fn on_before_opcode_read(&self, state: &CpuState) -> bool {
        todo!();
    }

}
