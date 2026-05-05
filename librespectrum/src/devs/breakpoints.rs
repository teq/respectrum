
use std::cell::{Cell, RefCell};

use crate::{core::Identifier, devs::CpuState};

pub enum BreakCondition {
    CpuStateMatch(Box<dyn Fn(&CpuState) -> bool>),
    BeforeOpcodeRead(Option<u16>),
    AfterMemoryAccess(u16),
    AfterMemoryAccessRange(u16, u16),
    AfterMemoryWrite(u16),
    AfterMemoryRead(u16),
}

pub struct BreakpointEntry {
    pub id: Identifier,
    pub condition: BreakCondition,
    pub once: bool,
}

#[derive(Default)]
pub struct BreakpointManager {
    breakpoints: RefCell<Vec<BreakpointEntry>>,
    next_id: Cell<Identifier>
}

impl BreakpointManager {

    fn generate_id(&self) -> Identifier {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    pub fn once(&self, breakpoint: BreakCondition) -> Identifier {
        self.add(breakpoint, true)
    }

    pub fn add(&self, condition: BreakCondition, once: bool) -> Identifier {
        let id = self.generate_id();
        self.breakpoints.borrow_mut().push(BreakpointEntry { id, condition, once });
        id
    }

    pub fn remove(&self, id: Identifier) {
        self.breakpoints.borrow_mut().retain(|entry| entry.id != id);
    }

    fn match_and_maybe_remove<F>(&self, predicate: F) -> Option<Identifier> where F: Fn(&BreakpointEntry) -> bool {
        let matched = self.breakpoints.borrow().iter()
            .rfind(|entry| predicate(entry))
            .map(|entry| (entry.id, entry.once));

        matched.map(|(id, once)| {
            if once {
                self.remove(id);
            }
            id
        })
    }

    pub fn hits_cpu_state_match(&self, state: &CpuState) -> Option<Identifier> {
        self.match_and_maybe_remove(|entry| {
            matches!(&entry.condition, BreakCondition::CpuStateMatch(condition) if condition(state))
        })
    }

    pub fn hits_before_opcode_read(&self, address: u16) -> Option<Identifier> {
        self.match_and_maybe_remove(|entry| {
            matches!(&entry.condition, BreakCondition::BeforeOpcodeRead(None))
            || matches!(&entry.condition, BreakCondition::BeforeOpcodeRead(Some(opcode_address)) if *opcode_address == address)
        })
    }

    pub fn hits_after_memory_read(&self, address: u16) -> Option<Identifier> {
        self.match_and_maybe_remove(|entry| {
            matches!(&entry.condition, BreakCondition::AfterMemoryAccess(access_address) if *access_address == address)
            || matches!(&entry.condition, BreakCondition::AfterMemoryAccessRange(start, end) if *start <= address && address <= *end)
            || matches!(&entry.condition, BreakCondition::AfterMemoryRead(read_address) if *read_address == address)
        })
    }

    pub fn hits_after_memory_write(&self, address: u16) -> Option<Identifier> {
        self.match_and_maybe_remove(|entry| {
            matches!(&entry.condition, BreakCondition::AfterMemoryAccess(access_address) if *access_address == address)
            || matches!(&entry.condition, BreakCondition::AfterMemoryAccessRange(start, end) if *start <= address && address <= *end)
            || matches!(&entry.condition, BreakCondition::AfterMemoryWrite(write_address) if *write_address == address)
        })
    }

}
