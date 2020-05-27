/// Decoded CPU token
#[derive(Debug)]
pub enum Token {

    Prefix(u16),
    Offset(i8),
    Operand(u8),

    // no prefix

    NOP,
    EX_AF,
    DJNZ,
    JR(Condition),
    LD_RP_NN(RegPair),
    ADD_HL_RP(RegPair),
    LD_RP_A(RegPair),
    LD_A_RP(RegPair),
    LD_MM_HL,
    LD_HL_MM,
    LD_MM_A,
    LD_A_MM,
    INC_RP(RegPair),
    DEC_RP(RegPair),
    INC_RG(Reg),
    DEC_RG(Reg),
    LD_RG_N(Reg),
    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    RET(Condition),
    POP(RegPair),
    EXX,
    JP_HL,
    LD_SP_HL,
    JP(Condition),
    OUT_N_A,
    IN_A_N,
    EX_AtSP_HL,
    EX_DE_HL,
    DI,
    EI,
    CALL(Condition),
    PUSH(RegPair),
    ALU_N(AluOp),
    RST(u8),
    HALT,
    LD_RG_RG(Reg, Reg),
    ALU_RG(AluOp, Reg),

    // ED prefix

    IN_RG_AtBC(Reg),
    IN_AtBC,
    OUT_AtBC_RG(Reg),
    OUT_AtBC_0,
    SBC_HL_RP(RegPair),
    ADC_HL_RP(RegPair),
    LD_AtMM_RP(RegPair),
    LD_RP_AtMM(RegPair),
    NEG,
    RETN,
    RETI,
    IM(IntMode),
    LD_I_A,
    LD_R_A,
    LD_A_I,
    LD_A_R,
    RRD,
    RLD,
    BLI(BlockOp),

    // CB prefix

    SH(ShiftOp, Reg),
    BIT(u8, Reg),
    RES(u8, Reg),
    SET(u8, Reg),

}

/// 8-bit register
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg {
    B = 0, C, D, E, H, L, AtHL, A, // DO NOT reorder
    R, I, IXH, IXL, IYH, IYL,
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
    SP, AF, IX, IY,
}

impl From<u8> for RegPair {
    fn from(code: u8) -> Self {
        unsafe { std::mem::transmute(code & 0b11) }
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
