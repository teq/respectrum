use std::fmt;

use crate::cpu::*;

/// Disassembled Z80 operation
pub struct Operation {
    pub addr: u16,
    pub len: u8,
    pub bytes: [u8; 4],
    pub opcode: Token,
    pub offset: Option<i8>,
    pub operand: Option<OperandValue>
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatter: Formatter = Default::default();
        f.write_str(formatter.format(self).as_str())
    }
}

#[derive(Default)]
pub struct FormatOptions {
    /// Convert relative addresses to absolute. **Default: `false`**
    pub calc_rel_addrs: Option<bool>,
}

/// Z80 operation formatter
#[derive(Default)]
pub struct Formatter {
    options: FormatOptions
}

impl Formatter {

    /// Create new formatter instance
    pub fn new(options: FormatOptions) -> Formatter {
        Formatter { options }
    }

    /// Format all fields
    pub fn format(&self, op: &Operation) -> String {
        format!(
            "{}: {:<11} | {}",
            self.format_addr(op),
            self.format_bytes(op),
            self.format_mnemonic(op)
        )
    }

    /// Format address field
    pub fn format_addr(&self, op: &Operation) -> String {
        format!("{:0>4X}", op.addr)
    }

    /// Format operation bytes
    pub fn format_bytes(&self, op: &Operation) -> String {
        let bytes = &op.bytes[..op.len as usize];
        let strings: Vec<String> = bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect();
        strings.join(" ")
    }

    pub fn format_mnemonic(&self, op: &Operation) -> String {

        match &op.opcode {

            Token::NOP => String::from("NOP"),
            Token::EX_AF => String::from("EX AF,AF'"),
            Token::DJNZ => format!("DJNZ {}", self.format_addr_offset(op)),
            Token::JR(cond) => match cond {
                Condition::None => format!("JR {}", self.format_addr_offset(op)),
                _ => format!("JR {},{}", self.format_condition(cond), self.format_addr_offset(op))
            },
            Token::LD_RP_NN(rpair) => format!("LD {},{}", self.format_regpair(rpair), self.format_operand(op)),
            Token::ADD_RP_RP(dst, src) => format!("ADD {},{}", self.format_regpair(dst), self.format_regpair(src)),
            Token::LD_AtRP_A(rpair) => format!("LD ({}),A", self.format_regpair(rpair)),
            Token::LD_A_AtRP(rpair) => format!("LD A,({})", self.format_regpair(rpair)),
            Token::LD_MM_RP(rpair) => format!("LD ({}),{}", self.format_operand(op), self.format_regpair(rpair)),
            Token::LD_RP_MM(rpair) => format!("LD {},({})", self.format_regpair(rpair), self.format_operand(op)),
            Token::LD_MM_A => format!("LD ({}),A", self.format_operand(op)),
            Token::LD_A_MM => format!("LD A,({})", self.format_operand(op)),
            Token::INC_RP(rpair) => format!("INC {}", self.format_regpair(rpair)),
            Token::DEC_RP(rpair) => format!("DEC {}", self.format_regpair(rpair)),
            Token::INC_RG(reg) => format!("INC {}", self.format_reg(reg, op)),
            Token::DEC_RG(reg) => format!("DEC {}", self.format_reg(reg, op)),
            Token::LD_RG_N(reg) => format!("LD {},{}", self.format_reg(reg, op), self.format_operand(op)),
            Token::RLCA => String::from("RLCA"),
            Token::RRCA => String::from("RRCA"),
            Token::RLA => String::from("RLA"),
            Token::RRA => String::from("RRA"),
            Token::DAA => String::from("DAA"),
            Token::CPL => String::from("CPL"),
            Token::SCF => String::from("SCF"),
            Token::CCF => String::from("CCF"),
            Token::RET(cond) => match cond {
                Condition::None => String::from("RET"),
                _ => format!("RET {}", self.format_condition(cond))
            },
            Token::POP(rpair) => format!("POP {}", self.format_regpair(rpair)),
            Token::EXX => String::from("EXX"),
            Token::JP_RP(rpair) => format!("JP ({})", self.format_regpair(rpair)),
            Token::LD_SP_RP(rpair) => format!("LD SP,{}", self.format_regpair(rpair)),
            Token::JP(cond) => match cond {
                Condition::None => format!("JP {}", self.format_operand(op)),
                _ => format!("JP {},{}", self.format_condition(cond), self.format_operand(op))
            },
            Token::OUT_N_A => format!("OUT ({}),A", self.format_operand(op)),
            Token::IN_A_N => format!("IN A,({})", self.format_operand(op)),
            Token::EX_AtSP_RP(rpair) => format!("EX (SP),{}", self.format_regpair(rpair)),
            Token::EX_DE_HL => String::from("EX DE,HL"),
            Token::DI => String::from("DI"),
            Token::EI => String::from("EI"),
            Token::CALL(cond) => match cond {
                Condition::None => format!("CALL {}", self.format_operand(op)),
                _ => format!("CALL {},{}", self.format_condition(cond), self.format_operand(op))
            },
            Token::PUSH(rpair) => format!("PUSH {}", self.format_regpair(rpair)),
            Token::ALU_N(alu_op) => self.format_alu_op(alu_op, &self.format_operand(op)),
            Token::RST(value) => format!("RST {}", self.format_byte(value * 8)),
            Token::HALT => String::from("HALT"),
            Token::LD_RG_RG(dst, src) => format!("LD {},{}", self.format_reg(dst, op), self.format_reg(src, op)),
            Token::ALU_RG(alu_op, reg) => self.format_alu_op(alu_op, &self.format_reg(reg, op)),
            Token::IN_RG_AtBC(reg) => format!("IN {},(C)", self.format_reg(reg, op)),
            Token::IN_AtBC => String::from("IN (C)"),
            Token::OUT_AtBC_RG(reg) => format!("OUT (C),{}", self.format_reg(reg, op)),
            Token::OUT_AtBC_0 => String::from("OUT (C),0"),
            Token::SBC_HL_RP(rpair) => format!("SBC HL,{}", self.format_regpair(rpair)),
            Token::ADC_HL_RP(rpair) => format!("ADC HL,{}", self.format_regpair(rpair)),
            Token::NEG => String::from("NEG"),
            Token::RETN => String::from("RETN"),
            Token::RETI => String::from("RETI"),
            Token::IM(mode) => format!("IM {}", match mode {
                IntMode::IM0 => "0",
                IntMode::IM01 => "0/1",
                IntMode::IM1 => "1",
                IntMode::IM2 => "2",
            }),
            Token::RRD => String::from("RRD"),
            Token::RLD => String::from("RLD"),
            Token::BLOP(block_op) => String::from(match block_op {
                BlockOp::LDI => "LDI",
                BlockOp::CPI => "CPI",
                BlockOp::INI => "INI",
                BlockOp::OUTI => "OUTI",
                BlockOp::LDD => "LDD",
                BlockOp::CPD => "CPD",
                BlockOp::IND => "IND",
                BlockOp::OUTD => "OUTD",
                BlockOp::LDIR => "LDIR",
                BlockOp::CPIR => "CPIR",
                BlockOp::INIR => "INIR",
                BlockOp::OTIR => "OTIR",
                BlockOp::LDDR => "LDDR",
                BlockOp::CPDR => "CPDR",
                BlockOp::INDR => "INDR",
                BlockOp::OTDR => "OTDR",
            }),
            Token::SHOP(shift_op, reg) => match shift_op {
                ShiftOp::RLC => format!("RLC {}", self.format_reg(reg, op)),
                ShiftOp::RRC => format!("RRC {}", self.format_reg(reg, op)),
                ShiftOp::RL => format!("RL {}", self.format_reg(reg, op)),
                ShiftOp::RR => format!("RR {}", self.format_reg(reg, op)),
                ShiftOp::SLA => format!("SLA {}", self.format_reg(reg, op)),
                ShiftOp::SRA => format!("SRA {}", self.format_reg(reg, op)),
                ShiftOp::SLL => format!("SLL {}", self.format_reg(reg, op)),
                ShiftOp::SRL => format!("SRL {}", self.format_reg(reg, op)),
            },
            Token::BIT(bit, reg) => format!("BIT {},{}", bit, self.format_reg(reg, op)),
            Token::RES(bit, reg) => format!("RES {},{}", bit, self.format_reg(reg, op)),
            Token::SET(bit, reg) => format!("SET {},{}", bit, self.format_reg(reg, op)),

            _ => unreachable!()

        }

    }

    fn format_condition(&self, condition: &Condition) -> &'static str {
        match condition {
            Condition::NZ => "NZ",
            Condition::Z => "Z",
            Condition::NC => "NC",
            Condition::C => "C",
            Condition::PO => "PO",
            Condition::PE => "PE",
            Condition::P => "P",
            Condition::M => "M",
            _ => unreachable!("{:?}", condition)
        }
    }

    fn format_regpair(&self, regpair: &RegPair) -> &'static str {
        match regpair {
            RegPair::BC => "BC",
            RegPair::DE => "DE",
            RegPair::HL => "HL",
            RegPair::SP => "SP",
            RegPair::AF => "AF",
            RegPair::IX => "IX",
            RegPair::IY => "IY",
            _ => unreachable!("{:?}", regpair)
        }
    }

    fn format_reg(&self, reg: &Reg, op: &Operation) -> String {
        match reg {
            Reg::B => String::from("B"),
            Reg::C => String::from("C"),
            Reg::D => String::from("D"),
            Reg::E => String::from("E"),
            Reg::H => String::from("H"),
            Reg::L => String::from("L"),
            Reg::AtHL => String::from("(HL)"),
            Reg::A => String::from("A"),
            Reg::R => String::from("R"),
            Reg::I => String::from("I"),
            Reg::IXH => String::from("IXH"),
            Reg::IXL => String::from("IXL"),
            Reg::IYH => String::from("IYH"),
            Reg::IYL => String::from("IYL"),
            Reg::AtIXd => {
                let offset = op.offset.unwrap() as i32;
                format!("(IX{})", self.format_number_with_sign(offset))
            },
            Reg::AtIYd => {
                let offset = op.offset.unwrap() as i32;
                format!("(IY{})", self.format_number_with_sign(offset))
            },
        }
    }

    fn format_alu_op(&self, alu_op: &AluOp, operand: &String) -> String {
        match alu_op {
            AluOp::ADD => format!("ADD A,{}", operand),
            AluOp::ADC => format!("ADC A,{}", operand),
            AluOp::SUB => format!("SUB A,{}", operand),
            AluOp::SBC => format!("SBC A,{}", operand),
            AluOp::AND => format!("AND {}", operand),
            AluOp::XOR => format!("XOR {}", operand),
            AluOp::OR => format!("OR {}", operand),
            AluOp::CP => format!("CP {}", operand),
        }
    }

    fn format_operand(&self, op: &Operation) -> String {
        match op.operand {
            Some(OperandValue::Byte(byte)) => self.format_byte(byte),
            Some(OperandValue::Word(word)) => self.format_word(word),
            _ => panic!("Expecting operand")
        }
    }

    fn format_addr_offset(&self, op: &Operation) -> String {
        let offset = op.offset.unwrap() as i32 + 2;
        if self.calc_rel_addrs() {
            let addr = (op.addr as i32 + offset) as u16;
            format!("{}", self.format_word(addr))
        } else {
            format!("${}", self.format_number_with_sign(offset))
        }
    }

    fn format_byte(&self, byte: u8) -> String {
        format!("{:X}h", byte)
    }

    fn format_word(&self, word: u16) -> String {
        format!("{:X}h", word)
    }

    fn format_number_with_sign(&self, num: i32) -> String {
        format!("{}{:X}h", (if num < 0 { "-" } else { "+" }), num.abs())
    }

    fn calc_rel_addrs(&self) -> bool {
        self.options.calc_rel_addrs.unwrap_or(false)
    }

}
