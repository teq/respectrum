use std::fmt;
use super::tokens::*;

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

impl Operation {

    pub fn byte_operand(&self) -> u8 {
        if let Some(OperandValue::Byte(value)) = self.operand {
            return value;
        } else {
            panic!("Expecting byte operand");
        }
    }

    pub fn word_operand(&self) -> u16 {
        if let Some(OperandValue::Word(value)) = self.operand {
            return value;
        } else {
            panic!("Expecting word operand");
        }
    }

}

pub struct FormatOptions {
    /// _Default: false_
    pub lower_case: Option<bool>,
    /// _Default: NumberFormat::Decimal_
    pub number_format: Option<NumberFormat>,
}

pub enum NumberFormat {
    Decimal,
    HexNoPrefix,
    HexHashPrefix,
    Hex0xPrefix,
}

pub struct Formatter {
    lower_case: bool,
    number_format: NumberFormat,
}

impl Default for Formatter {
    fn default() -> Formatter {
        Formatter {
            lower_case: false,
            number_format: NumberFormat::Decimal
        }
    }
}

impl Formatter {

    pub fn format(&self, op: &Operation) -> String {
        format!(
            "{}: {:<11} | {}",
            self.format_addr(op),
            self.format_bytes(op),
            self.format_mnemonic(op)
        )
    }

    pub fn format_addr(&self, op: &Operation) -> String {
        format!("{:0>4X}", op.addr)
    }

    pub fn format_bytes(&self, op: &Operation) -> String {
        let bytes = &op.bytes[..op.len as usize];
        let hex_bytes: Vec<String> = bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect();
        hex_bytes.join(" ")
    }

    pub fn format_mnemonic(&self, op: &Operation) -> String {

        let Operation { opcode, offset, .. } = op;

        match opcode {
            Token::NOP => String::from("NOP"),
            Token::EX_AF => String::from("EX AF,AF'"),
            Token::DJNZ => format!("DJNZ ${:+}", offset.unwrap() as i16 + 2),
            Token::JR(cond) => match cond {
                Condition::None => format!("JR ${:+}", offset.unwrap() as i16 + 2),
                _ => format!("JR {},${:+}", format_condition(cond), offset.unwrap() as i16 + 2)
            },
            Token::LD_RP_NN(rpair) => format!("LD {},{}", format_regpair(rpair), op.word_operand()),
            Token::ADD_RP_RP(dst, src) => format!("ADD {},{}", format_regpair(dst), format_regpair(src)),
            Token::LD_AtRP_A(rpair) => format!("LD ({}),A", format_regpair(rpair)),
            Token::LD_A_AtRP(rpair) => format!("LD A,({})", format_regpair(rpair)),
            Token::LD_MM_RP(rpair) => format!("LD ({}),{}", op.word_operand(), format_regpair(rpair)),
            Token::LD_RP_MM(rpair) => format!("LD {},({})", format_regpair(rpair), op.word_operand()),
            Token::LD_MM_A => format!("LD ({}),A", op.word_operand()),
            Token::LD_A_MM => format!("LD A,({})", op.word_operand()),
            Token::INC_RP(rpair) => format!("INC {}", format_regpair(rpair)),
            Token::DEC_RP(rpair) => format!("DEC {}", format_regpair(rpair)),
            Token::INC_RG(reg) => format!("INC {}", format_reg(reg, offset)),
            Token::DEC_RG(reg) => format!("DEC {}", format_reg(reg, offset)),
            Token::LD_RG_N(reg) => format!("LD {},{}", format_reg(reg, offset), op.byte_operand()),
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
                Condition::None => format!("JP {}", op.word_operand()),
                _ => format!("JP {},{}", format_condition(cond), op.word_operand())
            },
            Token::OUT_N_A => format!("OUT ({}),A", op.byte_operand()),
            Token::IN_A_N => format!("IN A,({})", op.byte_operand()),
            Token::EX_AtSP_RP(rpair) => format!("EX (SP),{}", format_regpair(rpair)),
            Token::EX_DE_HL => String::from("EX DE,HL"),
            Token::DI => String::from("DI"),
            Token::EI => String::from("EI"),
            Token::CALL(cond) => match cond {
                Condition::None => format!("CALL {}", op.word_operand()),
                _ => format!("CALL {},{}", format_condition(cond), op.word_operand())
            },
            Token::PUSH(rpair) => format!("PUSH {}", format_regpair(rpair)),
            Token::ALU_N(aluop) => format_alu_op(aluop, &op.byte_operand().to_string()),
            Token::RST(value) => format!("RST {}", value),
            Token::HALT => String::from("HALT"),
            Token::LD_RG_RG(dst, src) => format!("LD {},{}", format_reg(dst, offset), format_reg(src, offset)),
            Token::ALU_RG(aluop, reg) => format_alu_op(aluop, &String::from(format_reg(reg, offset))),

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
            Token::BLOP(blop) => String::from(format_block_op(blop)),

            Token::SHOP(blop, reg) => format_shift_op(blop, reg, offset),
            Token::BIT(bit, reg) => format!("BIT {},{}", bit, format_reg(reg, offset)),
            Token::RES(bit, reg) => format!("RES {},{}", bit, format_reg(reg, offset)),
            Token::SET(bit, reg) => format!("SET {},{}", bit, format_reg(reg, offset)),

            _ => unreachable!()
        }

    }

}

fn format_condition(condition: &Condition) -> &'static str {
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

fn format_regpair(regpair: &RegPair) -> &'static str {
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

fn format_reg(reg: &Reg, maybe_offset: &Option<i8>) -> String {
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
            if *reg == Reg::AtIXd {
                format!("(IX{:+})", offset)
            } else {
                format!("(IY{:+})", offset)
            }
        }
    }
}

fn format_alu_op(alu_op: &AluOp, operand: &String) -> String {
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

fn format_int_mode(mode: &IntMode) -> &'static str {
    match mode {
        IntMode::IM0 => "0",
        IntMode::IM01 => "0/1",
        IntMode::IM1 => "1",
        IntMode::IM2 => "2",
    }
}

fn format_block_op(block_op: &BlockOp) -> &'static str {
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

fn format_shift_op(shift_op: &ShiftOp, reg: &Reg, maybe_offset: &Option<i8>) -> String {
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
