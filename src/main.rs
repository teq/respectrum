mod types;
mod bus;
mod cpu;

use bus::Memory;
use cpu::Cpu;

fn main() {

    let mem = Memory::init();
    let cpu = Cpu::init();

    let al = std::mem::align_of_val(&cpu);
    let sz = std::mem::size_of_val(&cpu);
    println!("Value: {:#?}, align: {}, size: {}", cpu, al, sz);

}
