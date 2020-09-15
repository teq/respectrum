use std::{fmt, cell::Cell};

#[repr(C)]
pub union Word {
    w: Cell<u16>,
    b: WordBytes,
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

impl Default for Word {

    fn default() -> Self {
        Self { w: Cell::new(0) }
    }

}

impl fmt::Debug for Word {

    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:04x}h", self.word().get() ))
    }

}

impl Word {

    pub fn word(&self) -> &Cell<u16> {
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
    fn word_should_allow_to_update_its_value_and_individual_high_and_low_halves() {

        let w: Word = Default::default();
        assert_eq!(w.word().get(), 0);

        w.word().set(0x1234);
        assert_eq!(w.word().get(), 0x1234);
        assert_eq!(w.bytes().hi.get(), 0x12);
        assert_eq!(w.bytes().lo.get(), 0x34);

        w.bytes().hi.set(0xab);
        assert_eq!(w.word().get(), 0xab34);

        w.bytes().lo.set(0xcd);
        assert_eq!(w.word().get(), 0xabcd);

    }

}
