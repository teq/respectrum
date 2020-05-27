#![feature(generators, generator_trait)]
mod types;
mod cpu;

use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use std::io;
use std::io::Read;

fn main() {

    let mut decoder = cpu::decoder::opcode_decoder();

    for it in io::stdin().bytes() {
        let byte = it.unwrap();
        println!("Byte: {:#x}", byte);
        match Pin::new(&mut decoder).resume(byte) {
            GeneratorState::Yielded(token) => println!("-- {:?}", token),
            GeneratorState::Complete(token) => {
                println!("==> {:?}\n", token);
                decoder = cpu::decoder::opcode_decoder();
            }
        }
    }

}
