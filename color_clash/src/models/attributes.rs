use druid::Data;
use bitflags::bitflags;

use crate::palette::ZXColor;

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

impl<T> From<T> for Attributes where T: Into<u8> {
    fn from(value: T) -> Self {
        Self::from_bits_truncate(value.into())
    }
}

impl Attributes {

    pub fn get_paper(&self) -> ZXColor {
        ((*self & Attributes::PAPER_MASK).bits >> 3).into()
    }

    pub fn get_ink(&self) -> ZXColor {
        (*self & Attributes::INK_MASK).bits.into()
    }

    pub fn set_paper(&mut self, paper: impl Into<ZXColor>) {
        *self &= !Attributes::PAPER_MASK;
        *self |= ((paper.into().index() as u8 & 7) << 3).into();
    }

    pub fn set_ink(&mut self, ink: impl Into<ZXColor>) {
        *self &= !Attributes::INK_MASK;
        *self |= (ink.into().index() as u8 & 7).into();
    }

    pub fn get_paper_rgb(&self) -> [u8; 3] {
        self.get_rgb(self.get_paper())
    }

    pub fn get_ink_rgb(&self) -> [u8; 3] {
        self.get_rgb(self.get_ink())
    }

    fn get_rgb(&self, color: ZXColor) -> [u8; 3] {
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
        assert_eq!(attr.get_paper(), 7u8);
        assert_eq!(attr.get_ink(), 0u8);
        attr.set_paper(3u8);
        attr.set_ink(5u8);
        attr |= Attributes::FLASH | Attributes::BRIGHT;
        assert_eq!(attr.bits, 0b11011101);
    }

}
