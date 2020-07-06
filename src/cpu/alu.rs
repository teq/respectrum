use crate::{
    cpu::*,
};

bitflags! {
    /// CPU flags
    pub struct Flags : u8 {
        const NONE = 0;
        /// Carry flag
        const C = 1 << 0;
        /// Add / Subtract flag
        const N = 1 << 1;
        /// Parity / Overflow flag
        const P = 1 << 2;
        /// A copy of bit 3 of the result
        const X = 1 << 3;
        /// Half Carry flag
        const H = 1 << 4;
        /// A copy of bit 5 of the result
        const Y = 1 << 5;
        /// Zero flag
        const Z = 1 << 6;
        /// Sign flag
        const S = 1 << 7;
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        unsafe { Flags::from_bits_unchecked(value) }
    }
}

/// 8-bit add
pub fn add8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let (_, overflow) = (lhs as i8).overflowing_add(rhs as i8);
    let (_, hcarry) = (lhs << 4).overflowing_add(rhs << 4);
    let (result, carry) = lhs.overflowing_add(rhs);
    let mut flags = Flags::from(result & 0x28); // mask 3 & 5 bits
    flags.set(Flags::C, carry);
    flags.set(Flags::P, overflow);
    flags.set(Flags::H, hcarry);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit sub
pub fn sub8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let (_, overflow) = (lhs as i8).overflowing_sub(rhs as i8);
    let (_, hcarry) = (lhs << 4).overflowing_sub(rhs << 4);
    let (result, carry) = lhs.overflowing_sub(rhs);
    let mut flags = Flags::from(result & 0x28); // mask 3 & 5 bits
    flags.set(Flags::C, carry);
    flags.set(Flags::P, overflow);
    flags.set(Flags::H, hcarry);
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
    let mut flags = Flags::from(result & 0x28 | 0x10); // mask 3 & 5 bits, set 4 bit
    flags.set(Flags::P, result.count_ones() % 2 == 0);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit logical or
pub fn or8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs | rhs;
    let mut flags = Flags::from(result & 0x28); // mask 3 & 5 bits
    flags.set(Flags::P, result.count_ones() % 2 == 0);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit logical xor
pub fn xor8(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs ^ rhs;
    let mut flags = Flags::from(result & 0x28); // mask 3 & 5 bits
    flags.set(Flags::P, result.count_ones() % 2 == 0);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit compare
pub fn cp8(lhs: u8, rhs: u8) -> Flags {
    let (_, overflow) = (lhs as i8).overflowing_sub(rhs as i8);
    let (_, hcarry) = (lhs << 4).overflowing_sub(rhs << 4);
    let (result, carry) = lhs.overflowing_sub(rhs);
    let mut flags = Flags::from(rhs & 0x28 | 0x2); // mask 3 & 5 bits, set 1 bit
    flags.set(Flags::C, carry);
    flags.set(Flags::P, overflow);
    flags.set(Flags::H, hcarry);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return flags;
}

/// 8-bit increment
pub fn inc8(val: u8, flags: Flags) -> (u8, Flags) {
    let (_, overflow) = (val as i8).overflowing_add(1 as i8);
    let (_, hcarry) = (val << 4).overflowing_add(1 << 4);
    let result = val + 1;
    let mut flags = Flags::from(flags.bits() & 0x28); // mask 3 & 5 bits
    flags.set(Flags::P, overflow);
    flags.set(Flags::H, hcarry);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}

/// 8-bit decrement
pub fn dec8(val: u8, flags: Flags) -> (u8, Flags) {
    let (_, overflow) = (val as i8).overflowing_sub(1 as i8);
    let (_, hcarry) = (val << 4).overflowing_sub(1 << 4);
    let result = val - 1;
    let mut flags = Flags::from(flags.bits() & 0x28); // mask 3 & 5 bits
    flags.set(Flags::P, overflow);
    flags.set(Flags::H, hcarry);
    flags.set(Flags::Z, result == 0);
    flags.set(Flags::S, (result as i8) < 0);
    return (result, flags);
}
