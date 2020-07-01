#![allow(dead_code)]

/// Decoded CPU token
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Token {

    Prefix(u16),
    Offset(i8),
    Operand(OperandValue),

    // Register, memory, IO instructions
    LD_RG_RG(Reg, Reg),
    LD_AtRP_A(RegPair),
    LD_A_AtRP(RegPair),
    LD_MM_RP(RegPair),
    LD_RP_MM(RegPair),
    LD_MM_A,
    LD_A_MM,
    LD_SP_RP(RegPair),
    LD_RG_N(Reg),
    LD_RP_NN(RegPair),
    EX_AF,
    EXX,
    EX_DE_HL,
    OUT_N_A,
    IN_A_N,
    OUT_AtBC_RG(Reg),
    IN_RG_AtBC(Reg),
    OUT_AtBC_0, // undocumented
    IN_AtBC, // undocumented

    // Jumps/calls & stack
    DJNZ,
    RST(u8),
    JP(Condition),
    JP_RP(RegPair),
    JR(Condition),
    CALL(Condition),
    RET(Condition),
    POP(RegPair),
    PUSH(RegPair),
    EX_AtSP_RP(RegPair),

    // Arithmentic, shifts, bit ops
    ADD_RP_RP(RegPair, RegPair),
    INC_RG(Reg),
    DEC_RG(Reg),
    INC_RP(RegPair),
    DEC_RP(RegPair),
    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    ALU_N(AluOp),
    ALU_RG(AluOp, Reg),
    SBC_HL_RP(RegPair),
    ADC_HL_RP(RegPair),
    NEG,
    RRD,
    RLD,
    SHOP(ShiftOp, Reg),
    LDSH(Reg, ShiftOp, Reg), // undocumented
    BIT(u8, Reg),
    RES(u8, Reg),
    LDRES(Reg, u8, Reg),  // undocumented
    SET(u8, Reg),
    LDSET(Reg, u8, Reg),  // undocumented

    // Interrupts, misc
    NOP,
    HALT,
    DI,
    EI,
    IM(IntMode),
    RETN,
    RETI,

    // Block instructions
    BLOP(BlockOp),

}

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    /// Actual opcode or opcode prefix
    Opcode,
    /// Offset (or displacement) byte
    Offset,
    /// Operand (immediate data) byte
    Operand
}

/// Opcode operand value
#[derive(Debug, Clone, Copy)]
pub enum OperandValue {
    Byte(u8),
    Word(u16)
}

/// 8-bit register
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg {
    B = 0, C, D, E, H, L, AtHL, A, // DO NOT reorder
    F, I, R, IXH, IXL, IYH, IYL, AtIX, AtIY,
}

impl From<u8> for Reg {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b111) }
    }
}

/// 16-bit register pair
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RegPair {
    BC = 0, DE, HL, SPorAF, // DO NOT reorder
    SP, AF, PC, IR, IX, IY
}

impl From<u8> for RegPair {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b11) }
    }
}

impl RegPair {
    pub fn prefer_sp(self) -> RegPair {
        return if self == RegPair::SPorAF { RegPair::SP } else { self };
    }
    pub fn prefer_af(self) -> RegPair {
        return if self == RegPair::SPorAF { RegPair::AF } else { self };
    }
}

/// Condition
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Condition {
    NZ = 0, Z, NC, C, PO, PE, P, M, // DO NOT reorder
    None
}

impl From<u8> for Condition {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b111) }
    }
}

/// ALU operation
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AluOp {
    ADD = 0, ADC, SUB, SBC, AND, XOR, OR, CP, // DO NOT reorder
}

impl From<u8> for AluOp {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b111) }
    }
}

/// Shift operation
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ShiftOp {
    RLC = 0, RRC, RL, RR, SLA, SRA, SLL, SRL, // DO NOT reorder
}

impl From<u8> for ShiftOp {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b111) }
    }
}

/// Interrupt mode
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IntMode {
    IM0 = 0, IM01, IM1, IM2, // DO NOT reorder
}

impl From<u8> for IntMode {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b11) }
    }
}

/// Block operation
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockOp { // DO NOT reorder
    LDI = 0, CPI,  INI,  OUTI,
    LDD,     CPD,  IND,  OUTD,
    LDIR,    CPIR, INIR, OTIR,
    LDDR,    CPDR, INDR, OTDR,
}

impl From<u8> for BlockOp {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b1111) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_from_returns_register_corresponding_to_opcode_bits() {
        assert_eq!(Reg::B, Reg::from(0));
        assert_eq!(Reg::C, Reg::from(1));
        assert_eq!(Reg::D, Reg::from(2));
        assert_eq!(Reg::E, Reg::from(3));
        assert_eq!(Reg::H, Reg::from(4));
        assert_eq!(Reg::L, Reg::from(5));
        assert_eq!(Reg::AtHL, Reg::from(6));
        assert_eq!(Reg::A, Reg::from(7));
    }

    #[test]
    fn reg_pair_from_returns_register_pair_corresponding_to_opcode_bits() {
        assert_eq!(RegPair::BC,     RegPair::from(0));
        assert_eq!(RegPair::DE,     RegPair::from(1));
        assert_eq!(RegPair::HL,     RegPair::from(2));
        assert_eq!(RegPair::SPorAF, RegPair::from(3));
    }

    #[test]
    fn condition_from_returns_condition_corresponding_to_opcode_bits() {
        assert_eq!(Condition::NZ, Condition::from(0));
        assert_eq!(Condition::Z,  Condition::from(1));
        assert_eq!(Condition::NC, Condition::from(2));
        assert_eq!(Condition::C,  Condition::from(3));
        assert_eq!(Condition::PO, Condition::from(4));
        assert_eq!(Condition::PE, Condition::from(5));
        assert_eq!(Condition::P,  Condition::from(6));
        assert_eq!(Condition::M,  Condition::from(7));
    }

    #[test]
    fn alu_op_from_returns_alu_operation_corresponding_to_opcode_bits() {
        assert_eq!(AluOp::ADD, AluOp::from(0));
        assert_eq!(AluOp::ADC, AluOp::from(1));
        assert_eq!(AluOp::SUB, AluOp::from(2));
        assert_eq!(AluOp::SBC, AluOp::from(3));
        assert_eq!(AluOp::AND, AluOp::from(4));
        assert_eq!(AluOp::XOR, AluOp::from(5));
        assert_eq!(AluOp::OR,  AluOp::from(6));
        assert_eq!(AluOp::CP,  AluOp::from(7));
    }

    #[test]
    fn shift_op_from_returns_shift_operation_corresponding_to_opcode_bits() {
        assert_eq!(ShiftOp::RLC, ShiftOp::from(0));
        assert_eq!(ShiftOp::RRC, ShiftOp::from(1));
        assert_eq!(ShiftOp::RL,  ShiftOp::from(2));
        assert_eq!(ShiftOp::RR,  ShiftOp::from(3));
        assert_eq!(ShiftOp::SLA, ShiftOp::from(4));
        assert_eq!(ShiftOp::SRA, ShiftOp::from(5));
        assert_eq!(ShiftOp::SLL, ShiftOp::from(6));
        assert_eq!(ShiftOp::SRL, ShiftOp::from(7));
    }

    #[test]
    fn int_mode_from_returns_int_mode_corresponding_to_opcode_bits() {
        assert_eq!(IntMode::IM0,  IntMode::from(0));
        assert_eq!(IntMode::IM01, IntMode::from(1));
        assert_eq!(IntMode::IM1,  IntMode::from(2));
        assert_eq!(IntMode::IM2,  IntMode::from(3));
    }

    #[test]
    fn block_op_from_returns_block_operation_corresponding_to_opcode_bits() {
        assert_eq!(BlockOp::LDI,  BlockOp::from(0x0));
        assert_eq!(BlockOp::CPI,  BlockOp::from(0x1));
        assert_eq!(BlockOp::INI,  BlockOp::from(0x2));
        assert_eq!(BlockOp::OUTI, BlockOp::from(0x3));
        assert_eq!(BlockOp::LDD,  BlockOp::from(0x4));
        assert_eq!(BlockOp::CPD,  BlockOp::from(0x5));
        assert_eq!(BlockOp::IND,  BlockOp::from(0x6));
        assert_eq!(BlockOp::OUTD, BlockOp::from(0x7));
        assert_eq!(BlockOp::LDIR, BlockOp::from(0x8));
        assert_eq!(BlockOp::CPIR, BlockOp::from(0x9));
        assert_eq!(BlockOp::INIR, BlockOp::from(0xa));
        assert_eq!(BlockOp::OTIR, BlockOp::from(0xb));
        assert_eq!(BlockOp::LDDR, BlockOp::from(0xc));
        assert_eq!(BlockOp::CPDR, BlockOp::from(0xd));
        assert_eq!(BlockOp::INDR, BlockOp::from(0xe));
        assert_eq!(BlockOp::OTDR, BlockOp::from(0xf));
    }

}
