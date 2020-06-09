use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use crate::cpu::*;
use super::operation::*;

pub fn disassembler(mut addr: u16) -> impl Generator<u8, Yield=Option<Operation>> {

    move |mut byte: u8| {

        loop {

            let mut decoder = decoder();
            let mut len: u8 = 0;
            let mut bytes: [u8; 4] = [0; 4];
            let mut opcode: Option<Token> = None;
            let mut offset: Option<i8> = None;
            let mut operand: Option<OperandValue> = None;

            loop {

                bytes[len as usize] = byte;
                let state = Pin::new(&mut decoder).resume(byte);
                len += 1;

                match state {
                    GeneratorState::Yielded(token) | GeneratorState::Complete(token) => match token {
                        Token::Prefix(_) => (),
                        Token::Offset(value) => offset = Some(value),
                        Token::Operand(value) => operand = Some(value),
                        other => opcode = Some(other)
                    }
                }

                match state {
                    GeneratorState::Yielded(_) => byte = yield None,
                    GeneratorState::Complete(_) => break
                }

            };

            byte = yield Some(Operation {
                addr, len, bytes,
                opcode: opcode.unwrap(),
                offset, operand
            });

            addr += len as u16;

        }

    }

}
