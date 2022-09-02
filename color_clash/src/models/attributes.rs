use druid::Data;
use bitflags::bitflags;

use crate::palette::Color;

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

    pub fn get_paper(&self) -> Color {
        ((*self & Attributes::PAPER_MASK).bits >> 3).into()
    }

    pub fn get_ink(&self) -> Color {
        (*self & Attributes::INK_MASK).bits.into()
    }

    pub fn set_paper(&mut self, paper: impl Into<Color>) {
        *self &= !Attributes::PAPER_MASK;
        *self |= ((paper.into().index() as u8 & 7) << 3).into();
    }

    pub fn set_ink(&mut self, ink: impl Into<Color>) {
        *self &= !Attributes::INK_MASK;
        *self |= (ink.into().index() as u8 & 7).into();
    }

    pub fn get_paper_rgb(&self) -> [u8; 3] {
        let paper = self.get_paper();
        self.get_rgb(paper)
    }

    pub fn get_ink_rgb(&self) -> [u8; 3] {
        let ink = self.get_ink();
        self.get_rgb(ink)
    }

    fn get_rgb(&self, color: Color) -> [u8; 3] {
        if self.contains(Attributes::BRIGHT) {
            color.rgb_bright()
        } else {
            color.rgb_dim()
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_to_update_individual_attributes() {

        let mut attr: Attributes = Default::default();
        assert!(!attr.contains(Attributes::FLASH | Attributes::BRIGHT));
        assert_eq!(attr.get_paper(), 7usize);
        assert_eq!(attr.get_ink(), 0usize);

        attr.set_paper(3usize);
        attr.set_ink(5usize);
        attr |= Attributes::FLASH | Attributes::BRIGHT;
        assert_eq!(attr.bits, 0b11011101);

    }

}
