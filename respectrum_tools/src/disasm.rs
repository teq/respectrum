#![feature(coroutines, coroutine_trait)]

extern crate librespectrum;

use clap::Parser;

use std::{
    fs,
    vec::Vec,
    pin::Pin,
    path::PathBuf,
    ops::{Coroutine, CoroutineState},
    io::{self, BufReader, BufRead, Read}
};

use librespectrum::cpu::decoder::disassembler;

/// Maximum bytes to process for each disassembled line
const LINE_BYTES: usize = 4;

/// Z80 disassembler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {

    /// Disassemble base address
    #[clap(short, long, value_name = "ADDR", default_value_t = 0)]
    base_address: u16,

    /// Binary file to disassemble
    #[clap(short, long, value_name = "FILE")]
    input_file: Option<PathBuf>,

}

fn main() {

    let args = Args::parse();

    let reader: Box<dyn BufRead> = match args.input_file {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename).unwrap()))
    };

    let mut bytes = reader.bytes();
    let mut disasm = disassembler(args.base_address, LINE_BYTES);

    while let Some(Ok(byte)) = bytes.next() {

        if let CoroutineState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
            println!(
                "{:0>4X}: {:<bytes$} | {}",
                line.address,
                line.bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect::<Vec<String>>().join(" "),
                (if let Some(instr) = line.instruction {instr.format_mnemonic()} else {String::from("...")}),
                bytes = LINE_BYTES * 3 - 1
            );
        }

    }

}
