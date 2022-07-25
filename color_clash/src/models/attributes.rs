use druid::Data;
use bitflags::bitflags;

const NORMAL_PALETTE: [[u8; 3]; 8] = [
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0xee],
    [0xee, 0x00, 0x00],
    [0xee, 0x00, 0xee],
    [0x00, 0xee, 0x00],
    [0x00, 0xee, 0xee],
    [0xee, 0xee, 0x00],
    [0xee, 0xee, 0xee],
];

const BRIGHT_PALETTE: [[u8; 3]; 8] = [
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0xff],
    [0xff, 0x00, 0x00],
    [0xff, 0x00, 0xff],
    [0x00, 0xff, 0x00],
    [0x00, 0xff, 0xff],
    [0xff, 0xff, 0x00],
    [0xff, 0xff, 0xff],
];

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
        0b00111000.into()
    }
}

impl From<&u8> for Attributes {
    fn from(byte: &u8) -> Self {
        (*byte).into()
    }
}

impl From<u8> for Attributes {
    fn from(byte: u8) -> Self {
        Self::from_bits_truncate(byte)
    }
}


impl Attributes {

    pub fn get_paper(&self) -> u8 {
        (*self & Attributes::PAPER_MASK).bits >> 3
    }

    pub fn get_ink(&self) -> u8 {
        (*self & Attributes::INK_MASK).bits
    }

    pub fn set_paper(&mut self, paper: u8) {
        *self &= !Attributes::PAPER_MASK;
        *self |= ((paper & 7) << 3).into();
    }

    pub fn set_ink(&mut self, ink: u8) {
        *self &= !Attributes::INK_MASK;
        *self |= (ink & 7).into();
    }

    fn get_palette(&self) -> [[u8; 3]; 8] {
        if self.contains(Attributes::BRIGHT) { BRIGHT_PALETTE } else { NORMAL_PALETTE }
    }

    pub fn get_paper_rgb(&self) -> [u8; 3] {
        self.get_palette()[self.get_paper() as usize]
    }

    pub fn get_ink_rgb(&self) -> [u8; 3] {
        let palette = if self.contains(Attributes::BRIGHT) { BRIGHT_PALETTE } else { NORMAL_PALETTE };
        self.get_palette()[self.get_ink() as usize]
    }

}

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
