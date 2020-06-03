#![feature(generators, generator_trait)]

extern crate respectrum;

use std::fs::File;
use std::io::{self, BufRead};
use std::pin::Pin;
use std::ops::{Generator, GeneratorState};
use respectrum::cpu;

#[test]
fn disassembler_recognizes_all_z80_opcodes() {

    // File with reference listing
    let file = File::open("tests/misc/disassm.lst").unwrap();
    let mut lines = io::BufReader::new(file).lines().enumerate();

    // Disassembler to test
    let mut disassembler = cpu::disassembler(0);

    // Iterate over listing lines
    while let Some((line_num, Ok(line))) = lines.next() {

        let report_failure = |details: String| {
            panic!("Failure at line {}:\n{}\nDetails: {}", line_num, line, details);
        };

        // Line body excluding possible comments after ";"
        let body = if let Some(index) = line.find(";") { &line[..index] } else { &line[..] };

        if body.is_empty() { continue; }

        // Split address, opcode bytes and parsed disassembled mnemonic
        let mut parts = body.split(|c| c == ':' || c == '|');

        let address = u16::from_str_radix(parts.next().unwrap().trim(), 16).unwrap();
        let bytes = parts.next().unwrap().trim().split(" ").map(|s| u8::from_str_radix(s, 16).unwrap());
        let mnemonic = parts.next().unwrap().trim();

        let mut bytes_iter = bytes.enumerate().peekable();

        // Feed bytes to disassembler and observe results
        while let Some((byte_num, byte)) = bytes_iter.next() {

            if let GeneratorState::Yielded(maybe_line) = Pin::new(&mut disassembler).resume(byte) {

                if bytes_iter.peek().is_some() {

                    // Some bytes left in current opcode, disassembler should yield nothing
                    if maybe_line.is_some() {
                        report_failure(format!(
                            "Unexpected output when parsing byte number {}: {:?}",
                            byte_num, maybe_line
                        ));
                    }

                } else {

                    // It is a last byte for current opcode, disassembler should yield a line
                    if let Some(line) = maybe_line {

                        if address != line.address {
                            report_failure(format!(
                                "Wrong address. Expecting: {}, got: {}",
                                address, line.address
                            ));
                        }

                        // if bytes != line.bytes {
                        //     report_failure(format!(
                        //         "Opcode bytes do not match. Expecting: {}, got: {}",
                        //         address, line.address
                        //     ));
                        // }

                        if mnemonic != line.mnemonic {
                            report_failure(format!(
                                "Wrong mnemonic. Expecting: {}, got: {}",
                                mnemonic, line.mnemonic
                            ));
                        }

                    } else {
                        report_failure(String::from("No output on last opcode byte"));
                    }

                }

            }

        }

    }

}
