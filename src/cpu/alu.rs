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

#[inline]
fn parity(value: u8) -> bool {
    value.count_ones() % 2 == 0
}

// 8-Bit Load Group

/// Load accumulator. Used for: LD A,I/R
pub fn ld8(input: u8, iff2: bool, in_flags: Flags) -> Flags {
    let mut flags = (Flags::from(input) & Flags::XY) | (in_flags & Flags::C);
    flags.set(Flags::P, iff2);
    flags.set(Flags::Z, input == 0);
    flags.set(Flags::S, (input as i8) < 0);
    return flags;
}

// Exchange, Block Transfer, Search Group

pub fn ldi() {}

pub fn ldd() {}

// 8-Bit Arithmetic and Logical Group

/// 8-bit add
pub fn add8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let (result, carry) = lhs.overflowing_add(rhs);
    let mut flags = Flags::from(result) & Flags::XY;
    flags.set(Flags::C, carry);
    flags.set(Flags::P, (lhs as i8).overflowing_add(rhs as i8).1);
    flags.set(Flags::H, (lhs << 4).overflowing_add(rhs << 4).1);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit sub
pub fn sub8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let (result, carry) = lhs.overflowing_sub(rhs);
    let mut flags = (Flags::from(result) & Flags::XY) | Flags::N;
    flags.set(Flags::C, carry);
    flags.set(Flags::P, (lhs as i8).overflowing_sub(rhs as i8).1);
    flags.set(Flags::H, (lhs << 4).overflowing_sub(rhs << 4).1);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit add + carry
pub fn adc8(lhs: u8, rhs: u8, flags: Flags) -> (u8, Flags) {
    add8(lhs, if flags.contains(Flags::C) { rhs + 1 } else { rhs })
}

/// 8-bit sub + carry
pub fn sbc8(lhs: u8, rhs: u8, flags: Flags) -> (u8, Flags) {
    sub8(lhs, if flags.contains(Flags::C) { rhs - 1 } else { rhs })
}

/// 8-bit logical and
pub fn and8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs & rhs;
    let mut flags = (Flags::from(result) & Flags::XY) | Flags::H;
    flags.set(Flags::P, parity(result));
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit logical or
pub fn or8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs | rhs;
    let mut flags = Flags::from(result) & Flags::XY;
    flags.set(Flags::P, parity(result));
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit logical xor
pub fn xor8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs ^ rhs;
    let mut flags = Flags::from(result) & Flags::XY;
    flags.set(Flags::P, parity(result));
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit compare
pub fn cp8(lhs: u8, rhs: u8) -> Flags {
    let (result, carry) = lhs.overflowing_sub(rhs);
    let mut flags = (Flags::from(rhs) & Flags::XY) | Flags::N;
    flags.set(Flags::C, carry);
    flags.set(Flags::P, (lhs as i8).overflowing_sub(rhs as i8).1);
    flags.set(Flags::H, (lhs << 4).overflowing_sub(rhs << 4).1);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return flags;
}

/// 8-bit increment
pub fn inc8(input: u8, in_flags: Flags) -> (u8, Flags) {
    let output = input + 1;
    let mut flags = (Flags::from(output) & Flags::XY) | (in_flags & Flags::C);
    flags.set(Flags::P, (input as i8).overflowing_add(1 as i8).1);
    flags.set(Flags::H, (input << 4).overflowing_add(1 << 4).1);
    flags.set(Flags::Z, output == 0);
    flags.set(Flags::S, (output as i8) < 0);
    return (output, flags);
}

/// 8-bit decrement
pub fn dec8(input: u8, in_flags: Flags) -> (u8, Flags) {
    let output = input - 1;
    let mut flags = (Flags::from(output) & Flags::XY) | (in_flags & Flags::C) | Flags::N;
    flags.set(Flags::P, (input as i8).overflowing_sub(1 as i8).1);
    flags.set(Flags::H, (input << 4).overflowing_sub(1 << 4).1);
    flags.set(Flags::Z, output == 0);
    flags.set(Flags::S, (output as i8) < 0);
    return (output, flags);
}

// General-Purpose Arithmetic and CPU Control Group

/// Decimal adjust (BCD)
pub fn daa8(input: u8, in_flags: Flags) -> (u8, Flags) {
    let mut correction: u8 = 0;
    if input & 0x0f > 0x09 || in_flags.contains(Flags::H) { correction |= 0x06; }
    if input & 0xf0 > 0x90 || in_flags.contains(Flags::C) { correction |= 0x60; }
    let (output, carry) = if in_flags.contains(Flags::N) {
        input.overflowing_sub(correction)
    } else {
        input.overflowing_add(correction)
    };
    let mut flags = (Flags::from(output) & Flags::XY) | (in_flags & Flags::N);
    flags.set(Flags::C, carry);
    flags.set(Flags::P, parity(output));
    flags.set(Flags::H, if in_flags.contains(Flags::N) {
        (input << 4).overflowing_sub(correction << 4).1
    } else {
        (input << 4).overflowing_add(correction << 4).1
    });
    flags.set(Flags::Z, output == 0);
    flags.set(Flags::S, (output as i8) < 0);
    return (output, flags);
}

/// Invert (one's complement)
pub fn cpl8(input: u8, in_flags: Flags) -> (u8, Flags) {
    let output = !input;
    let flags = (Flags::from(output) & Flags::XY) | (in_flags & !Flags::XY) | Flags::H | Flags::N;
    return (output, flags);
}

/// Negate (two's complement)
pub fn neg8(input: u8) -> (u8, Flags) {
    sub8(0, input)
}

/// Invert carry flag
pub fn ccf(in_flags: Flags) -> Flags {
    in_flags ^ Flags::C
}

/// Set carry flag
pub fn scf(in_flags: Flags) -> Flags {
    in_flags | Flags::C
}
