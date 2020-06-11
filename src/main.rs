#![feature(generators, generator_trait)]
#![feature(or_patterns)]

pub mod bus;
pub mod cpu;
pub mod tools;
pub mod types;

use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use std::fs::File;
use std::io::{prelude::*, SeekFrom};

fn main() {

    let address: u16 = 0x8000;
    let offset: u16 = 0x0;
    let mut limit = 100;

    let mut file = File::open("tests/exerciser/zexall.bin").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.seek(SeekFrom::Start(offset as u64)).unwrap();
    file.read_to_end(&mut buffer).unwrap();

    let mut disassembler = tools::disassembler(address + offset);

    for byte in buffer {
        if let GeneratorState::Yielded(Some(op)) = Pin::new(&mut disassembler).resume(byte) {
            println!("{}", op);
            limit -= 1;
        }
        if limit <= 0 {
            break;
        }
    }

}
