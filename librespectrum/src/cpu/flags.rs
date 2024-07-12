bitflags! {
    /// CPU flags
    pub struct Flags : u8 {
        /// No flags set
        const NONE = 0;
        /// Carry flag
        const C = 1 << 0;
        /// Add / Subtract flag
        const N = 1 << 1;
        /// Parity / Overflow flag
        const P = 1 << 2;
        /// Bit 3 of the result
        const X = 1 << 3;
        /// Half Carry flag
        const H = 1 << 4;
        /// Bit 5 of the result
        const Y = 1 << 5;
        /// Zero flag
        const Z = 1 << 6;
        /// Sign flag
        const S = 1 << 7;
        /// Bits 3 & 5 of the result
        const XY = Self::X.bits | Self::Y.bits;
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        unsafe { Flags::from_bits_unchecked(value) }
    }
}

impl Flags {
    pub fn set_zs_flags(&mut self, value: u8) -> &mut Self {
        self.set(Flags::Z, value == 0);
        self.set(Flags::S, (value as i8) < 0);
        self
    }

    pub fn set_parity_flag(&mut self, value: u8) -> &mut Self {
        let mut value = value;
        value ^= value >> 4;
        value ^= value >> 2;
        value ^= value >> 1;
        self.set(Flags::P, value & 1 != 0);
        self
    }
}
