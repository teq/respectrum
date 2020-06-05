use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use std::fmt;
use super::tokens::*;
use super::decoder::*;

pub fn disassembler(mut address: u16) -> impl Generator<u8, Yield=Option<Line>> {

    move |mut byte: u8| {

        loop {

            let mut decoder = opcode_decoder();
            let mut bytes: Vec<u8> = Vec::with_capacity(4);
            let mut prefix: Option<u16> = None;
            let mut opcode: Option<Token> = None;
            let mut offset: Option<i8> = None;
            let mut operand: Option<OperandValue> = None;

            let mnemonic = loop {

                let mut process_token = |token: Token| {
                    match token {
                        Token::Prefix(value) => prefix = Some(value),
                        Token::Offset(value) => offset = Some(value),
                        Token::Operand(value) => operand = Some(value),
                        other => opcode = Some(other)
                    }
                };

                bytes.push(byte);
                match Pin::new(&mut decoder).resume(byte) {
                    GeneratorState::Yielded(token) => {
                        process_token(token);
                        byte = yield None;
                    },
                    GeneratorState::Complete(token) => {
                        process_token(token);
                        break format_mnemonic(opcode.unwrap(), offset, operand);
                    }
                }

            };

            let step = bytes.len() as u16;
            byte = yield Some(Line { address, bytes, mnemonic });
            address += step;

        }

    }

}

#[derive(Default, Debug)]
pub struct Line {
    pub address: u16,
    pub bytes: Vec<u8>,
    pub mnemonic: String
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>4X}: {:<11} | {}", self.address, format_bytes(&self.bytes), self.mnemonic)
    }
}

fn format_bytes(bytes: &Vec<u8>) -> String {
    let hex_bytes: Vec<String> = bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect();
    hex_bytes.join(" ")
}

fn format_mnemonic(opcode: Token, offset: Option<i8>, operand: Option<OperandValue>) -> String {

    match opcode {
        Token::NOP => String::from("NOP"),
        Token::EX_AF => String::from("EX AF,AF'"),
        Token::DJNZ => format!("DJNZ ${:+}", offset.unwrap() + 2),
        Token::JR(cond) => match cond {
            Condition::None => format!("JR ${:+}", offset.unwrap() + 2),
            _ => format!("JR {},${:+}", format_condition(cond), offset.unwrap() + 2)
        },
        Token::LD_RP_NN(rpair) => format!("LD {},{}", format_regpair(rpair), expect_word_operand(operand)),
        Token::ADD_RP_RP(dst, src) => format!("ADD {},{}", format_regpair(dst), format_regpair(src)),
        Token::LD_AtRP_A(rpair) => format!("LD ({}),A", format_regpair(rpair)),
        Token::LD_A_AtRP(rpair) => format!("LD A,({})", format_regpair(rpair)),
        Token::LD_MM_RP(rpair) => format!("LD ({}),{}", expect_word_operand(operand), format_regpair(rpair)),
        Token::LD_RP_MM(rpair) => format!("LD {},({})", format_regpair(rpair), expect_word_operand(operand)),
        Token::LD_MM_A => format!("LD ({}),A", expect_word_operand(operand)),
        Token::LD_A_MM => format!("LD A,({})", expect_word_operand(operand)),
        Token::INC_RP(rpair) => format!("INC {}", format_regpair(rpair)),
        Token::DEC_RP(rpair) => format!("DEC {}", format_regpair(rpair)),
        Token::INC_RG(reg) => format!("INC {}", format_reg(reg, offset)),
        Token::DEC_RG(reg) => format!("DEC {}", format_reg(reg, offset)),
        Token::LD_RG_N(reg) => format!("LD {},{}", format_reg(reg, offset), expect_byte_operand(operand)),
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
            _ => format!("RET {}", format_condition(cond))
        },
        Token::POP(rpair) => format!("POP {}", format_regpair(rpair)),
        Token::EXX => String::from("EXX"),
        Token::JP_RP(rpair) => format!("JP ({})", format_regpair(rpair)),
        Token::LD_SP_RP(rpair) => format!("LD SP,{}", format_regpair(rpair)),
        Token::JP(cond) => match cond {
            Condition::None => format!("JP {}", expect_word_operand(operand)),
            _ => format!("JP {},{}", format_condition(cond), expect_word_operand(operand))
        },
        Token::OUT_N_A => format!("OUT ({}),A", expect_byte_operand(operand)),
        Token::IN_A_N => format!("IN A,({})", expect_byte_operand(operand)),
        Token::EX_AtSP_RP(rpair) => format!("EX (SP),{}", format_regpair(rpair)),
        Token::EX_DE_HL => String::from("EX DE,HL"),
        Token::DI => String::from("DI"),
        Token::EI => String::from("EI"),
        Token::CALL(cond) => match cond {
            Condition::None => format!("CALL {}", expect_word_operand(operand)),
            _ => format!("CALL {},{}", format_condition(cond), expect_word_operand(operand))
        },
        Token::PUSH(rpair) => format!("PUSH {}", format_regpair(rpair)),
        Token::ALU_N(op) => format_alu_op(op, expect_byte_operand(operand).to_string()),
        Token::RST(value) => format!("RST {}", value),
        Token::HALT => String::from("HALT"),
        Token::LD_RG_RG(dst, src) => format!("LD {},{}", format_reg(dst, offset), format_reg(src, offset)),
        Token::ALU_RG(op, reg) => format_alu_op(op, String::from(format_reg(reg, offset))),

        Token::IN_RG_AtBC(reg) => format!("IN {},(C)", format_reg(reg, offset)),
        Token::IN_AtBC => String::from("IN (C)"),
        Token::OUT_AtBC_RG(reg) => format!("OUT (C),{}", format_reg(reg, offset)),
        Token::OUT_AtBC_0 => String::from("OUT (C),0"),
        Token::SBC_HL_RP(rpair) => format!("SBC HL,{}", format_regpair(rpair)),
        Token::ADC_HL_RP(rpair) => format!("ADC HL,{}", format_regpair(rpair)),
        Token::NEG => String::from("NEG"),
        Token::RETN => String::from("RETN"),
        Token::RETI => String::from("RETI"),
        Token::IM(mode) => format!("IM {}", format_int_mode(mode)),
        Token::RRD => String::from("RRD"),
        Token::RLD => String::from("RLD"),
        Token::BLI(op) => String::from(format_block_op(op)),

        Token::SH(op, reg) => format_shift_op(op, reg, offset),
        Token::BIT(bit, reg) => format!("BIT {},{}", bit, format_reg(reg, offset)),
        Token::RES(bit, reg) => format!("RES {},{}", bit, format_reg(reg, offset)),
        Token::SET(bit, reg) => format!("SET {},{}", bit, format_reg(reg, offset)),

        _ => String::from("???")
    }

}

fn expect_byte_operand(operand: Option<OperandValue>) -> u8 {
    if let Some(OperandValue::Byte(value)) = operand {
        return value;
    } else {
        panic!("Expecting byte operand");
    }
}

fn expect_word_operand(operand: Option<OperandValue>) -> u16 {
    if let Some(OperandValue::Word(value)) = operand {
        return value;
    } else {
        panic!("Expecting word operand");
    }
}

fn format_condition(condition: Condition) -> &'static str {
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

fn format_regpair(regpair: RegPair) -> &'static str {
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

fn format_reg(reg: Reg, maybe_offset: Option<i8>) -> String {
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
        Reg::AtIXd | Reg::AtIYd => {
            let offset = maybe_offset.expect("Expecting offset");
            if reg == Reg::AtIXd {
                format!("(IX{:+})", offset)
            } else {
                format!("(IY{:+})", offset)
            }
        }
    }
}

fn format_alu_op(alu_op: AluOp, operand: String) -> String {
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

fn format_int_mode(mode: IntMode) -> &'static str {
    match mode {
        IntMode::IM0 => "0",
        IntMode::IM01 => "0/1",
        IntMode::IM1 => "1",
        IntMode::IM2 => "2",
    }
}

fn format_block_op(block_op: BlockOp) -> &'static str {
    match block_op {
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
    }
}

fn format_shift_op(shift_op: ShiftOp, reg: Reg, maybe_offset: Option<i8>) -> String {
    match shift_op {
        ShiftOp::RLC => format!("RLC {}", format_reg(reg, maybe_offset)),
        ShiftOp::RRC => format!("RRC {}", format_reg(reg, maybe_offset)),
        ShiftOp::RL => format!("RL {}", format_reg(reg, maybe_offset)),
        ShiftOp::RR => format!("RR {}", format_reg(reg, maybe_offset)),
        ShiftOp::SLA => format!("SLA {}", format_reg(reg, maybe_offset)),
        ShiftOp::SRA => format!("SRA {}", format_reg(reg, maybe_offset)),
        ShiftOp::SLL => format!("SLL {}", format_reg(reg, maybe_offset)),
        ShiftOp::SRL => format!("SRL {}", format_reg(reg, maybe_offset)),
    }
}
