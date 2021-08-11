#![feature(generators, generator_trait)]
#![feature(never_type)]
#![feature(trait_alias)]
#![feature(untagged_unions)]

#[macro_use]
extern crate bitflags;

pub mod bus;
pub mod cpu;
pub mod devs;
pub mod misc;
pub mod tools;

fn main() {

    let clock = bus::Clock::new();
    let bus = bus::CpuBus::new();
    let cpu = cpu::Cpu::new(&bus, &clock);
    let mem = devs::mem::FlatRam::new(&bus, &clock);

    let mut scheduler = bus::Scheduler::new(&clock);
    scheduler.add(&cpu);
    scheduler.add(&mem);

}
