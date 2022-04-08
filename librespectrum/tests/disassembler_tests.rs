#![feature(generators, generator_trait)]

extern crate librespectrum;

use std::{
    fs::File,
    pin::Pin,
    io::{self, BufRead},
    ops::{Generator, GeneratorState}
};

use librespectrum::cpu::decoder::instruction_decoder;

#[test]
fn disassembler_recognizes_all_z80_opcodes() {

    // File with reference listing
    let file = File::open("tests/misc/opcodes.lst").unwrap();
    let mut lines = io::BufReader::new(file).lines().enumerate();

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

        let mut bytes_iter = parts.next().unwrap().trim().split(" ")
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .enumerate().peekable();
        let expected_mnemonic = parts.next().unwrap().trim();

        let mut decoder = instruction_decoder();

        // Feed bytes to disassembler and observe results
        while let Some((byte_num, byte)) = bytes_iter.next() {

            let result = Pin::new(&mut decoder).resume(byte);

            if bytes_iter.peek().is_some() {

                // Some bytes left in current opcode, disassembler should yield nothing
                if let GeneratorState::Complete(instruction) = result {
                    report_failure(format!(
                        "Unexpected output when parsing byte number {}: {}",
                        byte_num, instruction
                    ));
                }

            } else {

                // It's the last byte for current opcode, disassembler should yield a line
                if let GeneratorState::Complete(instruction) = result {

                    let formatted_mnemonic = instruction.format_mnemonic();
                    if formatted_mnemonic != expected_mnemonic  {
                        report_failure(format!(
                            "Wrong mnemonic. Expecting: {}, got: {}",
                            expected_mnemonic, formatted_mnemonic
                        ));
                    }

                } else {
                    report_failure(String::from("No output on last opcode byte"));
                }

            }

        }

    }

}
