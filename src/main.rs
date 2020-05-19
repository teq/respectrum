mod memory;
mod cpu;
mod display;

use memory::Memory;
use cpu::Cpu;
use display::Display;
use std::mem;

fn main() {

    let mem = Memory::init();
    let cpu = Cpu::init();
    let display = Display::init();

    let al = mem::align_of_val(&cpu);
    let sz = mem::size_of_val(&cpu);
    println!("Value: {:#?}, align: {}, size: {}", cpu, al, sz);

}
