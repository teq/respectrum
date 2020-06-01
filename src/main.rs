#![feature(generators, generator_trait)]
mod types;
mod cpu;

use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

use std::fs::File;
use std::io::{prelude::*, SeekFrom};

fn main() {

    let address: u16 = 0x8000;
    let offset: u16 = 0x0;
    let mut limit = 100;

    let mut file = File::open("test/zexall.bin").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.seek(SeekFrom::Start(offset as u64)).unwrap();
    file.read_to_end(&mut buffer).unwrap();

    let mut disassembler = cpu::disassembler(address + offset);

    for byte in buffer {
        if let GeneratorState::Yielded(Some(line)) = Pin::new(&mut disassembler).resume(byte) {
            println!("{}", line);
            limit -= 1;
        }
        if limit <= 0 {
            break;
        }
    }

}
