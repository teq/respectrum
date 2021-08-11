extern crate respectrum;

use respectrum::{bus, cpu, devs};

fn main() {

    let clock = bus::Clock::new();
    let bus = bus::CpuBus::new();
    let cpu = cpu::Cpu::new(&bus, &clock);
    let mem = devs::mem::FlatRam::new(&bus, &clock);

    let mut scheduler = bus::Scheduler::new(&clock);
    scheduler.add(&cpu);
    scheduler.add(&mem);

}
