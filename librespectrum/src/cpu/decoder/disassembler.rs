use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use super::{Instruction, instruction_decoder};

pub struct DisassembledLine {
    pub address: u16,
    pub bytes: Vec<u8>,
    pub instruction: Option<Instruction>,
}

pub fn disassembler(
    base_address: u16,
    line_bytes: usize
) -> impl Generator<u8, Yield=Option<DisassembledLine>, Return=!> {

    let mut address = base_address;
    let mut bytes = Vec::with_capacity(line_bytes);
    let mut decoder = instruction_decoder();

    move |mut byte: u8| {

        loop {

            bytes.push(byte);
            let bytes_len = bytes.len() as u16;
            if let GeneratorState::Complete(instruction) = Pin::new(&mut decoder).resume(byte) {
                byte = yield Some(DisassembledLine { address, bytes, instruction: Some(instruction) });
                address += bytes_len;
                bytes = Vec::with_capacity(line_bytes);
                decoder = instruction_decoder();
            } else if bytes.len() >= line_bytes {
                byte = yield Some(DisassembledLine { address, bytes, instruction: None });
                address += bytes_len;
                bytes = Vec::with_capacity(line_bytes);
            } else {
                byte = yield None;
            }

        }

    }

}
