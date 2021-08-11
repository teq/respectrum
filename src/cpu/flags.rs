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
