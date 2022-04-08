use std::{cell::Cell, ops::{Index, Deref}};

pub trait Memory: Index<u16, Output = Cell<u8>> + Deref<Target = Vec<Cell<u8>>> {
    fn writable(&self, addr: u16) -> bool;
}
