use std::fmt;

use crate::cpu::tokens::{Token, DataValue};
use super::InstructionFormatter;

/// Disassembled Z80 instruction
pub struct CpuInstruction {
    pub addr: u16,
    pub len: u8,
    pub bytes: [u8; 4],
    pub opcode: Token,
    pub displacement: Option<i8>,
    pub data: Option<DataValue>
}

impl fmt::Display for CpuInstruction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instr_formatter: InstructionFormatter = Default::default();
        formatter.write_str(instr_formatter.format(self).as_str())
    }
}
