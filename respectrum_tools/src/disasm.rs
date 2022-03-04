#![feature(generators, generator_trait)]

extern crate librespectrum;

use librespectrum::tools;

use std::{
    pin::Pin,
    fs::File,
    io::{Read, Seek, SeekFrom},
    ops::{Generator, GeneratorState},
};

fn main() {

    let address: u16 = 0x0000;
    let offset: u16 = 0x0;
    let mut limit = 100;

    let mut file = File::open("roms/48.rom").unwrap();
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
