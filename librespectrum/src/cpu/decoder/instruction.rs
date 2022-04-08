use std::fmt;

use crate::cpu::tokens::{
    Token, ShiftOp, BlockOp,
    IntMode, Condition, AluOp, Reg,
    RegPair, DataValue
};

/// Z80 CPU instruction
pub struct Instruction {
    pub opcode: Token,
    pub displacement: Option<i8>,
    pub data: Option<DataValue>,
}

impl fmt::Display for Instruction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.format_mnemonic().as_str())
    }
}

impl Instruction {

    pub fn expect_displacement(&self) -> i8 {
        self.displacement.expect("Expecting displacement to be defined")
    }

    pub fn expect_byte_data(&self) -> u8 {
        if let Some(DataValue::Byte(value)) = self.data {
            value
        } else {
            panic!("Expecting immediate data to be a byte");
        }
    }

    pub fn expect_word_data(&self) -> u16 {
        if let Some(DataValue::Word(value)) = self.data {
            value
        } else {
            panic!("Expecting immediate data to be a word");
        }
    }

    pub fn format_mnemonic(&self) -> String {

        match &self.opcode {

            // 8-bit Load
            Token::LD_RG_RG(dst, src) => format!("LD {},{}", self.format_reg(dst), self.format_reg(src)),
            Token::LD_RG_N(reg) => format!("LD {},{}", self.format_reg(reg), self.format_data()),
            Token::LD_A_AtRP(rpair) => format!("LD A,({})", self.format_regpair(rpair)),
            Token::LD_AtRP_A(rpair) => format!("LD ({}),A", self.format_regpair(rpair)),
            Token::LD_A_MM => format!("LD A,({})", self.format_data()),
            Token::LD_MM_A => format!("LD ({}),A", self.format_data()),

            // 16-bit Load
            Token::LD_RP_NN(rpair) => format!("LD {},{}", self.format_regpair(rpair), self.format_data()),
            Token::LD_RP_MM(rpair) => format!("LD {},({})", self.format_regpair(rpair), self.format_data()),
            Token::LD_MM_RP(rpair) => format!("LD ({}),{}", self.format_data(), self.format_regpair(rpair)),
            Token::LD_SP_RP(rpair) => format!("LD SP,{}", self.format_regpair(rpair)),
            Token::POP(rpair) => format!("POP {}", self.format_regpair(rpair)),
            Token::PUSH(rpair) => format!("PUSH {}", self.format_regpair(rpair)),

            // Exchange
            Token::EX_DE_HL => String::from("EX DE,HL"),
            Token::EX_AF => String::from("EX AF,AF'"),
            Token::EXX => String::from("EXX"),
            Token::EX_AtSP_RP(rpair) => format!("EX (SP),{}", self.format_regpair(rpair)),

            // 8-bit arithmetic and logic
            Token::ALU(op, maybe_reg) => {
                if let Some(reg) = maybe_reg {
                    self.format_alu_op(op, &self.format_reg(reg))
                } else {
                    self.format_alu_op(op, &self.format_data())
                }
            },
            Token::INC_RG(reg) => format!("INC {}", self.format_reg(reg)),
            Token::DEC_RG(reg) => format!("DEC {}", self.format_reg(reg)),

            // General-Purpose Arithmetic and CPU Control
            Token::DAA => String::from("DAA"),
            Token::CPL => String::from("CPL"),
            Token::NEG => String::from("NEG"),
            Token::CCF => String::from("CCF"),
            Token::SCF => String::from("SCF"),
            Token::NOP => String::from("NOP"),
            Token::HALT => String::from("HALT"),
            Token::DI => String::from("DI"),
            Token::EI => String::from("EI"),
            Token::IM(mode) => format!("IM {}", match mode {
                IntMode::IM0 | IntMode::IM01 => "0",
                IntMode::IM1 => "1",
                IntMode::IM2 => "2",
            }),

            // 16-Bit Arithmetic
            Token::ADD_RP_RP(dst, src) => format!("ADD {},{}", self.format_regpair(dst), self.format_regpair(src)),
            Token::ADC_HL_RP(rpair) => format!("ADC HL,{}", self.format_regpair(rpair)),
            Token::SBC_HL_RP(rpair) => format!("SBC HL,{}", self.format_regpair(rpair)),
            Token::INC_RP(rpair) => format!("INC {}", self.format_regpair(rpair)),
            Token::DEC_RP(rpair) => format!("DEC {}", self.format_regpair(rpair)),

            // Rotate and Shift
            Token::RLCA => String::from("RLCA"),
            Token::RLA => String::from("RLA"),
            Token::RRCA => String::from("RRCA"),
            Token::RRA => String::from("RRA"),
            Token::SHOP(op, reg) => match op {
                ShiftOp::RLC => format!("RLC {}", self.format_reg(reg)),
                ShiftOp::RRC => format!("RRC {}", self.format_reg(reg)),
                ShiftOp::RL => format!("RL {}", self.format_reg(reg)),
                ShiftOp::RR => format!("RR {}", self.format_reg(reg)),
                ShiftOp::SLA => format!("SLA {}", self.format_reg(reg)),
                ShiftOp::SRA => format!("SRA {}", self.format_reg(reg)),
                ShiftOp::SLL => format!("SLL {}", self.format_reg(reg)),
                ShiftOp::SRL => format!("SRL {}", self.format_reg(reg)),
            },
            Token::RLD => String::from("RLD"),
            Token::RRD => String::from("RRD"),
            Token::SHOPLD(op, reg, dst) => match op {
                ShiftOp::RLC => format!("RLC {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::RRC => format!("RRC {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::RL => format!("RL {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::RR => format!("RR {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::SLA => format!("SLA {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::SRA => format!("SRA {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::SLL => format!("SLL {},{}", self.format_reg(reg), self.format_reg(dst)),
                ShiftOp::SRL => format!("SRL {},{}", self.format_reg(reg), self.format_reg(dst)),
            },

            // Bit Set, Reset and Test
            Token::BIT(bit, reg) => format!("BIT {},{}", bit, self.format_reg(reg)),
            Token::SET(bit, reg) => format!("SET {},{}", bit, self.format_reg(reg)),
            Token::SETLD(bit, reg, dst) => format!(
                "SET {},{},{}",
                bit, self.format_reg(reg), self.format_reg(dst)
            ),
            Token::RES(bit, reg) => format!("RES {},{}", bit, self.format_reg(reg)),
            Token::RESLD(bit, reg, dst) => format!(
                "RES {},{},{}",
                bit, self.format_reg(reg), self.format_reg(dst)
            ),

            // Jump, Call and Return
            Token::JP(cond) => match cond {
                Condition::None => format!("JP {}", self.format_data()),
                _ => format!("JP {},{}", self.format_condition(cond), self.format_data())
            },
            Token::JP_RP(rpair) => format!("JP ({})", self.format_regpair(rpair)),
            Token::JR(cond) => match cond {
                Condition::None => format!("JR {}", self.format_addr_displacement()),
                _ => format!("JR {},{}", self.format_condition(cond), self.format_addr_displacement())
            },
            Token::DJNZ => format!("DJNZ {}", self.format_addr_displacement()),
            Token::CALL(cond) => match cond {
                Condition::None => format!("CALL {}", self.format_data()),
                _ => format!("CALL {},{}", self.format_condition(cond), self.format_data())
            },
            Token::RET(cond) => match cond {
                Condition::None => String::from("RET"),
                _ => format!("RET {}", self.format_condition(cond))
            },
            Token::RETI => String::from("RETI"),
            Token::RETN => String::from("RETN"),
            Token::RST(value) => format!("RST {}", self.format_byte(value * 8)),

            // IO
            Token::IN_A_N => format!("IN A,({})", self.format_data()),
            Token::OUT_N_A => format!("OUT ({}),A", self.format_data()),
            Token::IN_RG_AtBC(reg) => format!("IN {},(C)", self.format_reg(reg)),
            Token::OUT_AtBC_RG(reg) => format!("OUT (C),{}", self.format_reg(reg)),
            Token::IN_AtBC => String::from("IN (C)"),
            Token::OUT_AtBC_0 => String::from("OUT (C),0"),

            // Block transfer, search and IO
            Token::BLOP(op) => String::from(match op {
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

            other => unreachable!("{:?}", other)

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
            other => unreachable!("{:?}", other)
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
            other => unreachable!("{:?}", other)
        }
    }

    fn format_reg(&self, reg: &Reg) -> String {
        match reg {
            Reg::B => String::from("B"),
            Reg::C => String::from("C"),
            Reg::D => String::from("D"),
            Reg::E => String::from("E"),
            Reg::H => String::from("H"),
            Reg::L => String::from("L"),
            Reg::AtHL => String::from("(HL)"),
            Reg::A => String::from("A"),
            Reg::I => String::from("I"),
            Reg::R => String::from("R"),
            Reg::IXH => String::from("IXH"),
            Reg::IXL => String::from("IXL"),
            Reg::IYH => String::from("IYH"),
            Reg::IYL => String::from("IYL"),
            Reg::AtIX => {
                let displacement = self.displacement.unwrap();
                format!("(IX{})", self.format_number_with_sign(displacement as i32))
            },
            Reg::AtIY => {
                let displacement = self.displacement.unwrap();
                format!("(IY{})", self.format_number_with_sign(displacement as i32))
            },
            other => unreachable!("{:?}", other)
        }
    }

    fn format_alu_op(&self, op: &AluOp, operand: &String) -> String {
        match op {
            AluOp::ADD => format!("ADD A,{}", operand),
            AluOp::ADC => format!("ADC A,{}", operand),
            AluOp::SUB => format!("SUB {}", operand),
            AluOp::SBC => format!("SBC A,{}", operand),
            AluOp::AND => format!("AND {}", operand),
            AluOp::XOR => format!("XOR {}", operand),
            AluOp::OR => format!("OR {}", operand),
            AluOp::CP => format!("CP {}", operand),
        }
    }

    fn format_data(&self) -> String {
        match self.data {
            Some(DataValue::Byte(byte)) => self.format_byte(byte),
            Some(DataValue::Word(word)) => self.format_word(word),
            _ => panic!("Expecting immediate data")
        }
    }

    fn format_addr_displacement(&self) -> String {
        let displacement = self.displacement.unwrap() as i32 + 2;
        // let addr = (self.addr as i32 + displacement) as u16;
        // format!("{}", self.format_word(addr))
        format!("${}", self.format_number_with_sign(displacement))
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

}
