#![feature(generators, generator_trait)]

extern crate respectrum;

use std::{
    fs::File,
    pin::Pin,
    io::{self, BufRead},
    ops::{Generator, GeneratorState}
};

use respectrum::tools;

#[test]
fn disassembler_recognizes_all_z80_opcodes() {

    // File with reference listing
    let file = File::open("tests/misc/opcodes.lst").unwrap();
    let mut lines = io::BufReader::new(file).lines().enumerate();

    let mut current_addr: u16 = 0;

    // Disassembler to test
    let mut disassembler = tools::disassembler(current_addr);

    // Instruction formatter
    let formatter: tools::InstructionFormatter = Default::default();

    // Iterate over listing lines
    while let Some((line_idx, Ok(line))) = lines.next() {

        let report_failure = |details: String| {
            panic!("Failure at line {}:\n{}\nDetails: {}", line_idx + 1, line, details);
        };

        // Line body excluding possible comments after ";"
        let body = if let Some(index) = line.find(";") { &line[..index] } else { &line[..] };

        if body.is_empty() { continue; }

        // Split opcode bytes and parsed disassembled mnemonic
        let mut parts = body.split('|');

        let expected_bytes = parts.next().unwrap().trim();
        let expected_mnemonic = parts.next().unwrap().trim();

        let mut bytes_iter = expected_bytes.split(" ")
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .enumerate().peekable();

        // Feed bytes to disassembler and observe results
        while let Some((byte_num, byte)) = bytes_iter.next() {

            if let GeneratorState::Yielded(maybe_op) = Pin::new(&mut disassembler).resume(byte) {

                if bytes_iter.peek().is_some() {

                    // Some bytes left in current opcode, disassembler should yield nothing
                    if let Some(op) = maybe_op {
                        report_failure(format!(
                            "Unexpected output when parsing byte number {}: {}",
                            byte_num, op
                        ));
                    }

                } else {

                    // It is a last byte for current opcode, disassembler should yield a line
                    if let Some(op) = maybe_op {

                        if current_addr != op.addr {
                            report_failure(format!(
                                "Wrong address. Expecting: {}, got: {}",
                                current_addr, op.addr
                            ));
                        }

                        let formatted_bytes = formatter.format_bytes(&op);
                        if expected_bytes != formatted_bytes {
                            report_failure(format!(
                                "Opcode bytes do not match. Expecting: {}, got: {}",
                                expected_bytes, formatted_bytes
                            ));
                        }

                        let formatted_mnemonic = formatter.format_mnemonic(&op);
                        if expected_mnemonic != formatted_mnemonic {
                            report_failure(format!(
                                "Wrong mnemonic. Expecting: {}, got: {}",
                                expected_mnemonic, formatted_mnemonic
                            ));
                        }

                        current_addr += op.len as u16;

                    } else {
                        report_failure(String::from("No output on last opcode byte"));
                    }

                }

            }

        }

    }

}
