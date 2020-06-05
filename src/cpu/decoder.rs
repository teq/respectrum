use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use super::tokens::*;

fn get_x(byte: u8) -> u8 { (byte & 0b11000000) >> 6 }
fn get_y(byte: u8) -> u8 { (byte & 0b00111000) >> 3 }
fn get_z(byte: u8) -> u8 {  byte & 0b00000111       }
fn get_p(byte: u8) -> u8 { (byte & 0b00110000) >> 4 }
fn get_q(byte: u8) -> u8 { (byte & 0b00001000) >> 3 }

/// Merge high and low u8 bytes to a u16 word
macro_rules! mkword {
    ($high: expr, $low: expr) => { (($high as u16) << 8) | $low as u16 }
}

/// Yields decoded tokens and returns complete opcode token when it is decoded
pub fn opcode_decoder() -> impl Generator<u8, Yield=Token, Return=Token> {

    |mut byte: u8| {

        let mut prefix: Option<u16> = None;

        {
            // Z80 opcode may have multiple (possibly) duplicate and overriding each other prefixes.
            // Here we try to interpret incoming bytes as prefix bytes until we reach actual opcode
            // or offset (aka displacement) byte
            let mut decoder = prefix_decoder();
            while let GeneratorState::Yielded(token) = Pin::new(&mut decoder).resume(byte) {
                if let Token::Prefix(code) = token {
                    prefix = Some(code);
                }
                // Re-yield current prefix token and advance to the next byte
                byte = yield token;
            }
        }

        // When DD/FD prefix is set it alters (HL), H, L to (IX/IY+d), IXH/IYH, IXL/IYL resp.
        let alt_reg = move |reg: Reg| match prefix {
            None => reg,
            Some(other) => match other & 0xff {
                0xdd => match reg {
                    Reg::H => Reg::IXH,
                    Reg::L => Reg::IXL,
                    Reg::AtHL => Reg::AtIXd,
                    _ => reg
                },
                0xfd => match reg {
                    Reg::H => Reg::IYH,
                    Reg::L => Reg::IYL,
                    Reg::AtHL => Reg::AtIYd,
                    _ => reg
                },
                _ => reg
            }
        };

        // When DD/FD prefix is set it alters HL to IX/IY
        let alt_rpair = move |rpair: RegPair| match prefix {
            None => rpair,
            Some(other) => match other & 0xff {
                0xdd => if rpair == RegPair::HL { RegPair::IX } else { rpair },
                0xfd => if rpair == RegPair::HL { RegPair::IY } else { rpair },
                _ => rpair
            }
        };

        match prefix {

            Some(0xed) => match get_x(byte) {
                1 => match get_z(byte) {
                    0 => {
                        let y = get_y(byte);
                        if y == 6 {
                            Token::IN_AtBC
                        } else {
                            Token::IN_RG_AtBC(Reg::from(y))
                        }
                    },
                    1 => {
                        let y = get_y(byte);
                        if y == 6 {
                            Token::OUT_AtBC_0
                        } else {
                            Token::OUT_AtBC_RG(Reg::from(y))
                        }
                    },
                    2 => {
                        let (p, q) = (get_p(byte), get_q(byte));
                        if q == 0 {
                            Token::SBC_HL_RP(RegPair::from(p).prefer_sp())
                        } else {
                            Token::ADC_HL_RP(RegPair::from(p).prefer_sp())
                        }
                    },
                    3 => {
                        let (p, q) = (get_p(byte), get_q(byte));
                        let low_operand_byte = yield if q == 0 {
                            Token::LD_MM_RP(RegPair::from(p).prefer_sp())
                        } else {
                            Token::LD_RP_MM(RegPair::from(p).prefer_sp())
                        };
                        let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                        Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                    },
                    4 => Token::NEG,
                    5 => if get_y(byte) == 1 { Token::RETI } else { Token::RETN },
                    6 => Token::IM(IntMode::from(get_y(byte))),
                    7 => match get_y(byte) {
                        0 => Token::LD_RG_RG(Reg::I, Reg::A),
                        1 => Token::LD_RG_RG(Reg::R, Reg::A),
                        2 => Token::LD_RG_RG(Reg::A, Reg::I),
                        3 => Token::LD_RG_RG(Reg::A, Reg::R),
                        4 => Token::RRD,
                        5 => Token::RLD,
                        _ => Token::NOP
                    },
                    _ => unreachable!()
                },
                2 => {
                    let (z, y) = (get_z(byte), get_y(byte));
                    if z <= 3 && y >= 4 {
                        Token::BLI(BlockOp::from((y << 2 ) | z))
                    } else {
                        Token::NOP
                    }
                },
                _ => Token::NOP
            },

            Some(0xcb) => {
                let (y, z) = (get_y(byte), get_z(byte));
                match get_x(byte) {
                    0 => Token::SH(ShiftOp::from(y), Reg::from(z)),
                    1 => Token::BIT(y, Reg::from(z)),
                    2 => Token::RES(y, Reg::from(z)),
                    3 => Token::SET(y, Reg::from(z)),
                    _ => unreachable!()
                }
            },

            Some(0xcbdd) | Some(0xcbfd) => {
                byte = yield Token::Offset(byte as i8); // first byte after prefix is an offset
                let (y, z) = (get_y(byte), get_z(byte));
                match get_x(byte) {
                    0 => {
                        if z == 6 {
                            Token::SH(ShiftOp::from(y), alt_reg(Reg::AtHL))
                        } else {
                            Token::LDSH(alt_reg(Reg::from(z)), ShiftOp::from(y), alt_reg(Reg::AtHL))
                        }
                    },
                    1 => Token::BIT(y, alt_reg(Reg::AtHL)),
                    2 => {
                        if z == 6 {
                            Token::RES(y, alt_reg(Reg::AtHL))
                        } else {
                            Token::LDRES(alt_reg(Reg::from(z)), y, alt_reg(Reg::AtHL))
                        }
                    },
                    3 => {
                        if z == 6 {
                            Token::SET(y, alt_reg(Reg::AtHL))
                        } else {
                            Token::LDSET(alt_reg(Reg::from(z)), y, alt_reg(Reg::AtHL))
                        }
                    },
                    _ => unreachable!()
                }
            },

            Some(0xdd) | Some(0xfd) | None  => match byte & 0o307 { // mask y and z

                // x=0, z=0
                0o000 => match get_y(byte) {
                    0 => Token::NOP,
                    1 => Token::EX_AF,
                    y => {
                        let offset_byte = yield match y {
                            2 => Token::DJNZ,
                            3 => Token::JR(Condition::None),
                            _ => Token::JR(Condition::from(y & 0b11))
                        };
                        Token::Offset(offset_byte as i8)
                    }
                },

                // x=0, z=1
                0o001 => {
                    let p = get_p(byte);
                    if get_q(byte) == 0 {
                        let low_operand_byte = yield Token::LD_RP_NN(alt_rpair(RegPair::from(p).prefer_sp()));
                        let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                        Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                    } else {
                        Token::ADD_RP_RP(alt_rpair(RegPair::HL), alt_rpair(RegPair::from(p).prefer_sp()))
                    }
                },

                // x=0, z=2
                0o002 => match get_y(byte) {
                    0 => Token::LD_AtRP_A(RegPair::BC),
                    1 => Token::LD_A_AtRP(RegPair::BC),
                    2 => Token::LD_AtRP_A(RegPair::DE),
                    3 => Token::LD_A_AtRP(RegPair::DE),
                    y => {
                        let low_operand_byte = yield match y {
                            4 => Token::LD_MM_RP(alt_rpair(RegPair::HL)),
                            5 => Token::LD_RP_MM(alt_rpair(RegPair::HL)),
                            6 => Token::LD_MM_A,
                            7 => Token::LD_A_MM,
                            _ => unreachable!()
                        };
                        let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                        Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                    }
                },

                // x=0, z=3
                0o003 => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    let rp = alt_rpair(RegPair::from(p).prefer_sp());
                    if q == 0 { Token::INC_RP(rp) } else { Token::DEC_RP(rp) }
                },

                // x=0, z=4,5
                0o004 | 0o005 => {
                    let (y, z) = (get_y(byte), get_z(byte));
                    let opcode_token = if z == 4 {
                        Token::INC_RG(alt_reg(Reg::from(y)))
                    } else {
                        Token::DEC_RG(alt_reg(Reg::from(y)))
                    };
                    match prefix {
                        None => opcode_token,
                        Some(_) => {
                            let offset_byte = yield opcode_token;
                            Token::Offset(offset_byte as i8)
                        }
                    }
                },

                // x=0, z=6
                0o006 => {
                    byte = yield Token::LD_RG_N(alt_reg(Reg::from(get_y(byte))));
                    match prefix {
                        None => Token::Operand(OperandValue::Byte(byte)),
                        Some(_) => {
                            let operand_byte = yield Token::Offset(byte as i8);
                            Token::Operand(OperandValue::Byte(operand_byte))
                        }
                    }
                },

                // x=0, z=7
                0o007 => match get_y(byte) {
                    0 => Token::RLCA,
                    1 => Token::RRCA,
                    2 => Token::RLA,
                    3 => Token::RRA,
                    4 => Token::DAA,
                    5 => Token::CPL,
                    6 => Token::SCF,
                    7 => Token::CCF,
                    _ => unreachable!()
                },

                // x=3, z=0
                0o300 => Token::RET(Condition::from(get_y(byte))),

                // x=3, z=1
                0o301 => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    if q == 0 {
                        Token::POP(alt_rpair(RegPair::from(p).prefer_af()))
                    } else {
                        match p {
                            0 => Token::RET(Condition::None),
                            1 => Token::EXX,
                            2 => Token::JP_RP(alt_rpair(RegPair::HL)),
                            3 => Token::LD_SP_RP(alt_rpair(RegPair::HL)),
                            _ => unreachable!()
                        }
                    }
                },

                // x=3, z=2
                0o302 => {
                    let low_operand_byte = yield Token::JP(Condition::from(get_y(byte)));
                    let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                    Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                },

                // x=3, z=3
                0o303 => match get_y(byte) {
                    0 => {
                        let low_operand_byte = yield Token::JP(Condition::None);
                        let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                        Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                    },
                    y @ 2 | y @ 3 => {
                        let port_byte = yield if y == 2 { Token::OUT_N_A } else { Token::IN_A_N };
                        Token::Operand(OperandValue::Byte(port_byte))
                    },
                    4 => Token::EX_AtSP_RP(alt_rpair(RegPair::HL)),
                    5 => Token::EX_DE_HL,
                    6 => Token::DI,
                    7 => Token::EI,
                    _ => unreachable!()
                },

                // x=3, z=4
                0o304 => {
                    let low_operand_byte = yield Token::CALL(Condition::from(get_y(byte)));
                    let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                    Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                },

                // x=3, z=5
                0o305 => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    if q == 0 {
                        Token::PUSH(alt_rpair(RegPair::from(p).prefer_af()))
                    } else {
                        let low_operand_byte = yield Token::CALL(Condition::None);
                        let high_operand_byte = yield Token::Operand(OperandValue::Byte(low_operand_byte));
                        Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                    }
                },

                // x=3, z=6
                0o306 => {
                    let operand_byte = yield Token::ALU_N(AluOp::from(get_y(byte)));
                    Token::Operand(OperandValue::Byte(operand_byte))
                },

                // x=3, z=7
                0o307 => Token::RST(get_y(byte) * 8),

                _ => match get_x(byte) {

                    1 => {
                        let dst_reg = Reg::from(get_y(byte));
                        let src_reg = Reg::from(get_z(byte));
                        let opcode_token = if dst_reg == Reg::AtHL && src_reg == Reg::AtHL {
                            Token::HALT // exception
                        } else if dst_reg == Reg::AtHL {
                            Token::LD_RG_RG(alt_reg(dst_reg), src_reg)
                        } else if src_reg == Reg::AtHL {
                            Token::LD_RG_RG(dst_reg, alt_reg(src_reg))
                        } else {
                            Token::LD_RG_RG(alt_reg(dst_reg), alt_reg(src_reg))
                        };
                        match prefix {
                            None => opcode_token,
                            Some(_) => Token::Offset((yield opcode_token) as i8)
                        }
                    },

                    2 => {
                        let opcode_token = Token::ALU_RG(
                            AluOp::from(get_y(byte)), alt_reg(Reg::from(get_z(byte)))
                        );
                        match prefix {
                            None => opcode_token,
                            Some(_) => Token::Offset((yield opcode_token) as i8)
                        }
                    },

                    _ => unreachable!()

                }

            },

            _ => unreachable!()

        }

    }

}

/// Yields decoded prefix tokens and completes on first non-prefix token
pub fn prefix_decoder() -> impl Generator<u8, Yield=Token> {

    |mut byte: u8| {

        let mut current_prefix: Option<u16> = None;

        loop {

            let next_prefix = match current_prefix {

                // CB & ED are always followed by opcode byte
                Some(0xcb) | Some(0xed) => return,

                // DDCB & FDCB are always followed by offset (aka displacement) byte
                Some(0xcbdd) | Some(0xcbfd) => return,

                Some(val) if (val == 0xdd || val == 0xfd) => match byte {

                    // If DD or FD followed by DD, ED or FD we should ignore former prefix
                    0xdd | 0xed | 0xfd => byte as u16,

                    // DD or FD followed by CB gives DDCB or FDCB
                    0xcb => mkword!(byte, val),

                    // Otherwise it is followed by opcode
                    _ => return

                },

                _ => match byte {

                    // All prefixes start with CB, ED, DD or FD
                    0xcb | 0xed | 0xdd | 0xfd => byte as u16,

                    // Otherwise it is an opcode
                    _ => return

                }

            };

            // Yield prefix token and advance to the next byte
            byte = yield Token::Prefix(next_prefix);

            current_prefix = Some(next_prefix);

        }

    }

}
