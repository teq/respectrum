use std::ops::{Index, IndexMut};

pub struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(addr_bits: u8, page_bits: u8) -> Self {
        assert!(addr_bits <= 16);
        assert!(addr_bits + page_bits <= 24);
        Self {
            memory: vec![0; usize::pow(2, addr_bits as u32 + page_bits as u32)]
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}
