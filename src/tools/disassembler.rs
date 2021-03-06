use std::ops::Generator;

use crate::cpu::InstructionDecoder;
use super::CpuInstruction;

/// Create generator which accepts bytes and yields disassembled CPU instructions
pub fn disassembler(mut addr: u16) -> impl Generator<u8, Yield=Option<CpuInstruction>> {

    move |mut byte: u8| {

        // Loop through CPU instructions
        loop {

            let mut decoder = InstructionDecoder::new();
            let mut len: u8 = 0;
            let mut bytes: [u8; 4] = [0; 4];

            // Decode instruction (1-4 bytes)
            while {
                bytes[len as usize] = byte; len += 1;
                decoder.decode(byte)
            } {
                // Instruction decode is in progress => yield nothing
                byte = yield None;
            }

            // Yield disassembled CPU instruction
            byte = yield Some(CpuInstruction {
                addr, len, bytes,
                opcode: decoder.expect_opcode(),
                displacement: decoder.displacement(),
                data: decoder.data()
            });

            addr += len as u16;

        }

    }

}
