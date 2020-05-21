use std::{fs, ops};

const PAGE_SIZE: u16 = 0x4000;

pub struct Memory {

    /// Holds all memory pages, both ROM and RAM
    pages: Vec<Vec<u8>>,

    /// Maps memory windows to particular memory pages
    windows: [usize; 4]

}

impl ops::Index<u16> for Memory {
    type Output = u8;
    fn index(&self, addr: u16) -> &Self::Output {
        let window_page = self.windows[(addr / PAGE_SIZE) as usize];
        let page = &self.pages[window_page];
        &page[(addr % PAGE_SIZE) as usize]
    }
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

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asd() {

    }

}
