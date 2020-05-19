use std::fs;

const PAGE_SIZE: u16 = 0x4000;

pub struct Memory {

    /// Holds all memory pages, both ROM and RAM
    pages: Vec<Vec<u8>>,

    /// Maps memory windows to particular memory pages
    windows: [usize; 4]

}

impl Memory {

    // Initialize memory subsystem
    pub fn init() -> Memory {

        let rom = fs::read("./roms/48.rom").unwrap();

        Memory {
            pages: vec![
                rom,
                vec![0; PAGE_SIZE as usize],
                vec![0; PAGE_SIZE as usize],
                vec![0; PAGE_SIZE as usize]
            ],
            windows: [1, 2, 3, 4]
        }

    }

    /// Read byte from memory
    pub fn read_byte(&self, addr: u16) -> u8 {
        let window_page = self.windows[(addr / PAGE_SIZE) as usize];
        let page = &self.pages[window_page];
        page[(addr % PAGE_SIZE) as usize]
    }

    /// Write byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let window_page = self.windows[(addr / PAGE_SIZE) as usize];
        let page = &mut self.pages[window_page];
        page[(addr % PAGE_SIZE) as usize] = value;
    }

}
