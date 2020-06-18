use std::{
    fs,
    rc::Rc,
    ops::{Index, IndexMut}
};

use crate::bus::{Device, CpuBus, clock::Task};

const ROM_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0xc000;

/// Plain 48k memory
pub struct Memory {
    memory: Vec<u8>,
    // bus: Rc<CpuBus>,
}

impl Index<u16> for Memory {
    type Output = u8;
    fn index(&self, addr: u16) -> &Self::Output {
        &self.memory[addr as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        if (addr as usize) < ROM_SIZE {
            static mut TMP: u8 = 0; // TODO: try to avoid this static variable hack
            return unsafe { &mut TMP };
        } else {
            return &mut self.memory[addr as usize]
        }
    }
}

// impl Device for Memory {

//     fn run<'a>(&'a self) -> Box<dyn Task + 'a> {
//         Box::new(move || {
//             loop {
//                 yield self.bus.clock.rising(4);
//             }
//         })
//     }

// }

impl Memory {

    // Initialize new memory instance
    pub fn new() -> Memory {
        let mut memory: Vec<u8> = fs::read("./roms/48.rom").unwrap();
        assert_eq!(memory.len(), ROM_SIZE);
        memory.append(&mut vec![0; RAM_SIZE]);
        Memory { memory }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_new_memory_instance() {
        let mem = Memory::new();
        assert_eq!(mem.memory.len(), ROM_SIZE + RAM_SIZE);
    }

    #[test]
    fn rom_should_allow_reads() {
        let mem = Memory::new();
        assert_eq!(mem[0x0000], 0xf3);
        assert_eq!(mem[0x3fff], 0x3c);
    }

    #[test]
    fn rom_should_ignore_writes() {
        let mut mem = Memory::new();
        assert_eq!(mem[0x0000], 0xf3);
        mem[0x0000] = 0x00;
        assert_eq!(mem[0x0000], 0xf3);
    }

    #[test]
    fn ram_should_allow_reads() {
        let mem = Memory::new();
        assert_eq!(mem[0x4000], 0x00);
        assert_eq!(mem[0xffff], 0x00);

    }

    #[test]
    fn ram_should_allow_writes() {
        let mut mem = Memory::new();
        assert_eq!(mem[0x4000], 0x00);
        mem[0x4000] = 0xff;
        assert_eq!(mem[0x4000], 0xff);
    }

}
