use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use crate::cpu::*;
use super::cpu_instruction::*;

/// Create generator which accepts bytes and yields disassembled CPU instructions
pub fn disassembler(mut addr: u16) -> impl Generator<u8, Yield=Option<CpuInstruction>> {

    move |mut byte: u8| {

        loop {

            let mut decoder = opcode_decoder();
            let mut len: u8 = 0;
            let mut bytes: [u8; 4] = [0; 4];
            let mut opcode: Option<Token> = None;
            let mut offset: Option<i8> = None;
            let mut operand: Option<OperandValue> = None;

            // This loop decodes each individual CPU operation (1-4 bytes)
            loop {

                bytes[len as usize] = byte;
                let state = Pin::new(&mut decoder).resume(byte);
                len += 1;

                match state {
                    GeneratorState::Yielded(result) | GeneratorState::Complete(result) => match result {
                        DecodeResult { token: Token::Prefix(_), .. } => (),
                        DecodeResult { token: Token::Offset(value), .. } => offset = Some(value),
                        DecodeResult { token: Token::Operand(value), .. } => operand = Some(value),
                        DecodeResult { token, .. } => opcode = Some(token)
                    }
                }

                match state {
                    // Decode is in progress => yield nothing
                    GeneratorState::Yielded(_) => byte = yield None,
                    // Decode completed
                    GeneratorState::Complete(_) => break
                }

            };

            // Yield disassembled CPU instruction
            byte = yield Some(CpuInstruction {
                addr, len, bytes,
                opcode: opcode.unwrap(),
                offset, operand
            });

            addr += len as u16;

        }

    }

}
