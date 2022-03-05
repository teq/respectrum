use std::{cell::Cell, ops::Index};

mod dynamic_48k;
pub use dynamic_48k::Dynamic48k;

pub trait Memory: Index<u16, Output = Cell<u8>> {
    fn writable(&self, addr: u16) -> bool;
}
