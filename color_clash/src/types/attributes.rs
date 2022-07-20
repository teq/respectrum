use druid::Data;
use bitflags::bitflags;

bitflags! {
    /// Color, brightness and flash attributes
    #[derive(Data)]
    pub struct Attributes: u8 {
        const FLASH      = 0b10000000;
        const BRIGHT     = 0b01000000;
        const PAPER_MASK = 0b00111000;
        const INK_MASK   = 0b00000111;
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self::from_bits_truncate(0b00111000)
    }
}

impl Attributes {

    pub fn get_paper(&self) -> u8 {
        (*self & Attributes::PAPER_MASK).bits >> 3
    }

    pub fn get_ink(&self) -> u8 {
        (*self & Attributes::INK_MASK).bits
    }

    pub fn set_ink(&mut self, ink: u8) {
        *self &= !Attributes::INK_MASK;
        *self |= Attributes::from_bits_truncate(ink & 7);
    }

    pub fn set_paper(&mut self, paper: u8) {
        *self &= !Attributes::PAPER_MASK;
        *self |= Attributes::from_bits_truncate((paper & 7) << 3);
    }

}

pub const NORMAL_PALETTE: [[u8; 3]; 8] = [
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0xee],
    [0xee, 0x00, 0x00],
    [0xee, 0x00, 0xee],
    [0x00, 0xee, 0x00],
    [0x00, 0xee, 0xee],
    [0xee, 0xee, 0x00],
    [0xee, 0xee, 0xee],
];

pub const BRIGHT_PALETTE: [[u8; 3]; 8] = [
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0xff],
    [0xff, 0x00, 0x00],
    [0xff, 0x00, 0xff],
    [0x00, 0xff, 0x00],
    [0x00, 0xff, 0xff],
    [0xff, 0xff, 0x00],
    [0xff, 0xff, 0xff],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_to_update_individual_attributes() {

        let mut attr: Attributes = Default::default();
        assert!(!attr.contains(Attributes::FLASH | Attributes::BRIGHT));
        assert_eq!(attr.get_paper(), 7);
        assert_eq!(attr.get_ink(), 0);

        attr.set_paper(3);
        attr.set_ink(5);
        attr |= Attributes::FLASH | Attributes::BRIGHT;
        assert_eq!(attr.bits, 0b11011101);

    }

}
