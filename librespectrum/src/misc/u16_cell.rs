use std::{
    fmt,
    cell::Cell,
    mem::ManuallyDrop,
};

#[repr(C)]
/// Stores u16 value in Cell and allows to access high & low bytes individually
pub union U16Cell {
    w: ManuallyDrop<Cell<u16>>,
    b: ManuallyDrop<WordBytes>,
}

#[repr(C)]
#[cfg(target_endian = "little")]
pub struct WordBytes {
    pub lo: Cell<u8>,
    pub hi: Cell<u8>,
}

#[repr(C)]
#[cfg(target_endian = "big")]
pub struct WordBytes {
    pub hi: Cell<u8>,
    pub lo: Cell<u8>,
}

impl Default for U16Cell {

    fn default() -> Self {
        Self { w: ManuallyDrop::new(Cell::new(0)) }
    }

}

impl fmt::Debug for U16Cell {

    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:04x}h", self.value().get() ))
    }

}

impl U16Cell {

    pub fn value(&self) -> &Cell<u16> {
        unsafe { &self.w }
    }

    pub fn bytes(&self) -> &WordBytes {
        unsafe { &self.b }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_to_update_its_value_and_individual_high_and_low_halves() {

        let u16cell: U16Cell = Default::default();
        assert_eq!(u16cell.value().get(), 0);

        u16cell.value().set(0x1234);
        assert_eq!(u16cell.value().get(), 0x1234);
        assert_eq!(u16cell.bytes().hi.get(), 0x12);
        assert_eq!(u16cell.bytes().lo.get(), 0x34);

        u16cell.bytes().hi.set(0xab);
        assert_eq!(u16cell.value().get(), 0xab34);

        u16cell.bytes().lo.set(0xcd);
        assert_eq!(u16cell.value().get(), 0xabcd);

    }

}
