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
        Token::EX_AF => String::from("EX AF,AF`"),
        Token::DJNZ => format!("DJNZ ${:+}", offset.unwrap()),
        Token::JR(cond) => format!("JR {},${:+}", format_condition(cond), offset.unwrap()),
        Token::LD_RP_NN(rpair) => format!("LD {},{:X}", format_regpair(rpair), expect_word_operand(operand)),
        Token::ADD_HL_RP(rpair) => format!("ADD HL,{}", format_regpair(rpair)),
        Token::LD_AtRP_A(rpair) => format!("ADD ({}),A", format_regpair(rpair)),
        Token::LD_A_AtRP(rpair) => format!("ADD A,({})", format_regpair(rpair)),
        Token::LD_MM_HL => format!("LD ({}), HL", expect_word_operand(operand)),
        Token::LD_HL_MM => format!("LD HL, ({})", expect_word_operand(operand)),
        Token::LD_MM_A => format!("LD ({}), A", expect_word_operand(operand)),
        Token::LD_A_MM => format!("LD A, ({})", expect_word_operand(operand)),
        Token::INC_RP(rpair) => format!("INC {}", format_regpair(rpair)),
        Token::DEC_RP(rpair) => format!("DEC {}", format_regpair(rpair)),
        Token::INC_RG(reg) => format!("INC {}", format_reg(reg)),
        Token::DEC_RG(reg) => format!("DEC {}", format_reg(reg)),
        Token::LD_RG_N(reg) => format!("LD {},{:X}", format_reg(reg), expect_byte_operand(operand)),
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
        Token::JP_HL => String::from("JP (HL)"),
        Token::LD_SP_HL => String::from("LD SP,HL"),
        Token::JP(cond) => match cond {
            Condition::None => format!("JP {:X}", expect_word_operand(operand)),
            _ => format!("JP {},{:X}", format_condition(cond), expect_word_operand(operand))
        },
        Token::OUT_N_A => format!("OUT ({:X}),A", expect_byte_operand(operand)),
        Token::IN_A_N => format!("IN A,({:X})", expect_byte_operand(operand)),
        Token::EX_AtSP_HL => String::from("EX (SP),HL"),
        Token::EX_DE_HL => String::from("EX DE,HL"),
        Token::DI => String::from("DI"),
        Token::EI => String::from("EI"),
        Token::CALL(cond) => match cond {
            Condition::None => format!("CALL {:X}", expect_word_operand(operand)),
            _ => format!("CALL {},{:X}", format_condition(cond), expect_word_operand(operand))
        },
        Token::PUSH(rpair) => format!("PUSH {}", format_regpair(rpair)),
        Token::ALU_N(op) => format!("{} {}", format_alu_op(op), expect_byte_operand(operand)),
        Token::RST(value) => format!("RST {}", value),
        Token::HALT => String::from("HALT"),
        Token::LD_RG_RG(dst, src) => format!("LD {},{}", format_reg(dst), format_reg(src)),
        Token::ALU_RG(op, reg) => format!("{} {}", format_alu_op(op), format_reg(reg)),


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
        _ => unreachable!()
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
        _ => unreachable!()
    }
}

fn format_reg(reg: Reg) -> &'static str {
    match reg {
        Reg::B => "B",
        Reg::C => "C",
        Reg::D => "D",
        Reg::E => "E",
        Reg::H => "H",
        Reg::L => "L",
        Reg::AtHL => "(HL)",
        Reg::A => "A",
        Reg::R => "R",
        Reg::I => "I",
        Reg::IXH => "IXH",
        Reg::IXL => "IXL",
        Reg::IYH => "IYH",
        Reg::IYL => "IYL",
        _ => unreachable!()
    }
}

fn format_alu_op(alu_op: AluOp) -> &'static str {
    match alu_op {
        AluOp::ADD => "ADD",
        AluOp::ADC => "ADC",
        AluOp::SUB => "SUB",
        AluOp::SBC => "SBC",
        AluOp::AND => "AND",
        AluOp::XOR => "XOR",
        AluOp::OR => "OR",
        AluOp::CP => "CP",
    }
}
