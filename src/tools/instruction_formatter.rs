use crate::cpu::tokens::{Token, ShiftOp, BlockOp, IntMode, Condition, AluOp, Reg, RegPair, DataValue};
use super::CpuInstruction;

#[derive(Default)]
pub struct InstructionFormatOptions {
    /// Convert relative addresses to absolute. **Default: `false`**
    pub calc_rel_addrs: Option<bool>,
}

/// Z80 instruction formatter
#[derive(Default)]
pub struct InstructionFormatter {
    options: InstructionFormatOptions
}

impl InstructionFormatter {

    /// Create new formatter instance
    pub fn new(options: InstructionFormatOptions) -> InstructionFormatter {
        InstructionFormatter { options }
    }

    /// Format all fields
    pub fn format(&self, inst: &CpuInstruction) -> String {
        format!(
            "{}: {:<11} | {}",
            self.format_addr(inst),
            self.format_bytes(inst),
            self.format_mnemonic(inst)
        )
    }

    /// Format address field
    pub fn format_addr(&self, inst: &CpuInstruction) -> String {
        format!("{:0>4X}", inst.addr)
    }

    /// Format operation bytes
    pub fn format_bytes(&self, inst: &CpuInstruction) -> String {
        let bytes = &inst.bytes[..inst.len as usize];
        let strings: Vec<String> = bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect();
        strings.join(" ")
    }

    pub fn format_mnemonic(&self, inst: &CpuInstruction) -> String {

        match &inst.opcode {

            // 8-bit Load
            Token::LD_RG_RG(dst, src) => format!("LD {},{}", self.format_reg(dst, inst), self.format_reg(src, inst)),
            Token::LD_RG_N(reg) => format!("LD {},{}", self.format_reg(reg, inst), self.format_data(inst)),
            Token::LD_A_AtRP(rpair) => format!("LD A,({})", self.format_regpair(rpair)),
            Token::LD_AtRP_A(rpair) => format!("LD ({}),A", self.format_regpair(rpair)),
            Token::LD_A_MM => format!("LD A,({})", self.format_data(inst)),
            Token::LD_MM_A => format!("LD ({}),A", self.format_data(inst)),

            // 16-bit Load
            Token::LD_RP_NN(rpair) => format!("LD {},{}", self.format_regpair(rpair), self.format_data(inst)),
            Token::LD_RP_MM(rpair) => format!("LD {},({})", self.format_regpair(rpair), self.format_data(inst)),
            Token::LD_MM_RP(rpair) => format!("LD ({}),{}", self.format_data(inst), self.format_regpair(rpair)),
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
                    self.format_alu_op(op, &self.format_reg(reg, inst))
                } else {
                    self.format_alu_op(op, &self.format_data(inst))
                }
            },
            Token::INC_RG(reg) => format!("INC {}", self.format_reg(reg, inst)),
            Token::DEC_RG(reg) => format!("DEC {}", self.format_reg(reg, inst)),

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
                ShiftOp::RLC => format!("RLC {}", self.format_reg(reg, inst)),
                ShiftOp::RRC => format!("RRC {}", self.format_reg(reg, inst)),
                ShiftOp::RL => format!("RL {}", self.format_reg(reg, inst)),
                ShiftOp::RR => format!("RR {}", self.format_reg(reg, inst)),
                ShiftOp::SLA => format!("SLA {}", self.format_reg(reg, inst)),
                ShiftOp::SRA => format!("SRA {}", self.format_reg(reg, inst)),
                ShiftOp::SLL => format!("SLL {}", self.format_reg(reg, inst)),
                ShiftOp::SRL => format!("SRL {}", self.format_reg(reg, inst)),
            },
            Token::RLD => String::from("RLD"),
            Token::RRD => String::from("RRD"),
            Token::SHOPLD(op, reg, dst) => match op {
                ShiftOp::RLC => format!("RLC {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::RRC => format!("RRC {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::RL => format!("RL {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::RR => format!("RR {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::SLA => format!("SLA {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::SRA => format!("SRA {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::SLL => format!("SLL {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
                ShiftOp::SRL => format!("SRL {},{}", self.format_reg(reg, inst), self.format_reg(dst, inst)),
            },

            // Bit Set, Reset and Test
            Token::BIT(bit, reg) => format!("BIT {},{}", bit, self.format_reg(reg, inst)),
            Token::SET(bit, reg) => format!("SET {},{}", bit, self.format_reg(reg, inst)),
            Token::SETLD(bit, reg, dst) => format!(
                "SET {},{},{}",
                bit, self.format_reg(reg, inst), self.format_reg(dst, inst)
            ),
            Token::RES(bit, reg) => format!("RES {},{}", bit, self.format_reg(reg, inst)),
            Token::RESLD(bit, reg, dst) => format!(
                "RES {},{},{}",
                bit, self.format_reg(reg, inst), self.format_reg(dst, inst)
            ),

            // Jump, Call and Return
            Token::JP(cond) => match cond {
                Condition::None => format!("JP {}", self.format_data(inst)),
                _ => format!("JP {},{}", self.format_condition(cond), self.format_data(inst))
            },
            Token::JP_RP(rpair) => format!("JP ({})", self.format_regpair(rpair)),
            Token::JR(cond) => match cond {
                Condition::None => format!("JR {}", self.format_addr_displacement(inst)),
                _ => format!("JR {},{}", self.format_condition(cond), self.format_addr_displacement(inst))
            },
            Token::DJNZ => format!("DJNZ {}", self.format_addr_displacement(inst)),
            Token::CALL(cond) => match cond {
                Condition::None => format!("CALL {}", self.format_data(inst)),
                _ => format!("CALL {},{}", self.format_condition(cond), self.format_data(inst))
            },
            Token::RET(cond) => match cond {
                Condition::None => String::from("RET"),
                _ => format!("RET {}", self.format_condition(cond))
            },
            Token::RETI => String::from("RETI"),
            Token::RETN => String::from("RETN"),
            Token::RST(value) => format!("RST {}", self.format_byte(value * 8)),

            // IO
            Token::IN_A_N => format!("IN A,({})", self.format_data(inst)),
            Token::OUT_N_A => format!("OUT ({}),A", self.format_data(inst)),
            Token::IN_RG_AtBC(reg) => format!("IN {},(C)", self.format_reg(reg, inst)),
            Token::OUT_AtBC_RG(reg) => format!("OUT (C),{}", self.format_reg(reg, inst)),
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

    fn format_reg(&self, reg: &Reg, inst: &CpuInstruction) -> String {
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
                let displacement = inst.displacement.unwrap();
                format!("(IX{})", self.format_number_with_sign(displacement as i32))
            },
            Reg::AtIY => {
                let displacement = inst.displacement.unwrap();
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

    fn format_data(&self, inst: &CpuInstruction) -> String {
        match inst.data {
            Some(DataValue::Byte(byte)) => self.format_byte(byte),
            Some(DataValue::Word(word)) => self.format_word(word),
            _ => panic!("Expecting immediate data")
        }
    }

    fn format_addr_displacement(&self, inst: &CpuInstruction) -> String {
        let displacement = inst.displacement.unwrap() as i32 + 2;
        if self.calc_rel_addrs() {
            let addr = (inst.addr as i32 + displacement) as u16;
            format!("{}", self.format_word(addr))
        } else {
            format!("${}", self.format_number_with_sign(displacement))
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
