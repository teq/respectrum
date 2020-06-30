use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use crate::{
    mkword,
    cpu::tokens::*
};

fn get_x(byte: u8) -> u8 { (byte & 0b11000000) >> 6 }
fn get_y(byte: u8) -> u8 { (byte & 0b00111000) >> 3 }
fn get_z(byte: u8) -> u8 {  byte & 0b00000111       }
fn get_p(byte: u8) -> u8 { (byte & 0b00110000) >> 4 }
fn get_q(byte: u8) -> u8 { (byte & 0b00001000) >> 3 }

/// Result of decoding a byte into a token
#[derive(Debug, Clone, Copy)]
pub struct DecodeResult {
    /// Decoded token
    pub token: Token,
    /// Expected type for the next token.
    /// During emulation it allows to decide
    /// which M-cycle to use to process next byte
    pub upnext: TokenType,
}

/// Create generator which accepts bytes, yields decoded
/// tokens and returns when entire instruction sequence is decoded
pub fn opcode_decoder() -> impl Generator<u8, Yield=DecodeResult, Return=DecodeResult> {

    |mut byte: u8| {

        let mut prefix: Option<u16> = None;

        {
            // Z80 opcode may have multiple (possibly) duplicate and overriding each other prefixes.
            // Here we try to interpret incoming bytes as prefix bytes until we reach actual opcode
            // or offset (aka displacement) byte
            let mut decoder = prefix_decoder();
            while let GeneratorState::Yielded(result) = Pin::new(&mut decoder).resume(byte) {
                if let DecodeResult { token: Token::Prefix(code), .. } = result {
                    prefix = Some(code);
                }
                // Re-yield current prefix token and advance to the next byte
                byte = yield result;
            }
        }

        // When DD/FD prefix is set it alters (HL), H, L to (IX/IY+d), IXH/IYH, IXL/IYL resp.
        let alt_reg = move |reg: Reg| match prefix {
            None => reg,
            Some(other) => match other & 0xff {
                0xdd => match reg {
                    Reg::H => Reg::IXH,
                    Reg::L => Reg::IXL,
                    Reg::AtHL => Reg::AtIX,
                    _ => reg
                },
                0xfd => match reg {
                    Reg::H => Reg::IYH,
                    Reg::L => Reg::IYL,
                    Reg::AtHL => Reg::AtIY,
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

            Some(0xed) => match (get_x(byte), get_y(byte), get_z(byte)) {
                (1, 6, 0) => DecodeResult { token: Token::IN_AtBC, upnext: TokenType::Opcode },
                (1, y, 0) => DecodeResult { token: Token::IN_RG_AtBC(Reg::from(y)), upnext: TokenType::Opcode },
                (1, 6, 1) => DecodeResult { token: Token::OUT_AtBC_0, upnext: TokenType::Opcode },
                (1, y, 1) => DecodeResult { token: Token::OUT_AtBC_RG(Reg::from(y)), upnext: TokenType::Opcode },
                (1, _, 2) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    DecodeResult {
                        token: if q == 0 {
                            Token::SBC_HL_RP(RegPair::from(p).prefer_sp())
                        } else {
                            Token::ADC_HL_RP(RegPair::from(p).prefer_sp())
                        },
                        upnext: TokenType::Opcode
                    }
                },
                (1, _, 3) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    let low_operand_byte = yield DecodeResult {
                        token: if q == 0 {
                            Token::LD_MM_RP(RegPair::from(p).prefer_sp())
                        } else {
                            Token::LD_RP_MM(RegPair::from(p).prefer_sp())
                        },
                        upnext: TokenType::Operand
                    };
                    let high_operand_byte = yield DecodeResult {
                        token: Token::Operand(OperandValue::Byte(low_operand_byte)),
                        upnext: TokenType::Operand
                    };
                    DecodeResult {
                        token: Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte))),
                        upnext: TokenType::Opcode
                    }
                },
                (1, _, 4) => DecodeResult { token: Token::NEG, upnext: TokenType::Opcode },
                (1, 1, 5) => DecodeResult { token: Token::RETI, upnext: TokenType::Opcode },
                (1, _, 5) => DecodeResult { token: Token::RETN, upnext: TokenType::Opcode },
                (1, y, 6) => DecodeResult { token: Token::IM(IntMode::from(y)), upnext: TokenType::Opcode },
                (1, 0, 7) => DecodeResult { token: Token::LD_RG_RG(Reg::I, Reg::A), upnext: TokenType::Opcode },
                (1, 1, 7) => DecodeResult { token: Token::LD_RG_RG(Reg::R, Reg::A), upnext: TokenType::Opcode },
                (1, 2, 7) => DecodeResult { token: Token::LD_RG_RG(Reg::A, Reg::I), upnext: TokenType::Opcode },
                (1, 3, 7) => DecodeResult { token: Token::LD_RG_RG(Reg::A, Reg::R), upnext: TokenType::Opcode },
                (1, 4, 7) => DecodeResult { token: Token::RRD, upnext: TokenType::Opcode },
                (1, 5, 7) => DecodeResult { token: Token::RLD, upnext: TokenType::Opcode },
                (1, _, 7) => DecodeResult { token: Token::NOP, upnext: TokenType::Opcode },
                (1, _, _) => unreachable!(),
                (2, y, z) if z <= 3 && y >= 4 => DecodeResult {
                    token: Token::BLOP(BlockOp::from((y << 2 ) | z)),
                    upnext: TokenType::Opcode
                },
                (2, _, _) => DecodeResult { token: Token::NOP, upnext: TokenType::Opcode },
                (_, _, _) => DecodeResult { token: Token::NOP, upnext: TokenType::Opcode }
            },

            Some(0xcb) => {
                let (y, z) = (get_y(byte), get_z(byte));
                match get_x(byte) {
                    0 => DecodeResult { token: Token::SHOP(ShiftOp::from(y), Reg::from(z)), upnext: TokenType::Opcode },
                    1 => DecodeResult { token: Token::BIT(y, Reg::from(z)), upnext: TokenType::Opcode },
                    2 => DecodeResult { token: Token::RES(y, Reg::from(z)), upnext: TokenType::Opcode },
                    3 => DecodeResult { token: Token::SET(y, Reg::from(z)), upnext: TokenType::Opcode },
                    _ => unreachable!()
                }
            },

            Some(0xcbdd) | Some(0xcbfd) => {
                // First byte after DDCB/FDCB is an offset and then opcode
                byte = yield DecodeResult { token: Token::Offset(byte as i8), upnext: TokenType::Opcode };
                DecodeResult {
                    token: match (get_x(byte), get_y(byte), get_z(byte)) {
                        (0, y, 6) => Token::SHOP(ShiftOp::from(y), alt_reg(Reg::AtHL)),
                        (0, y, z) => Token::LDSH(alt_reg(Reg::from(z)), ShiftOp::from(y), alt_reg(Reg::AtHL)),
                        (1, y, _) => Token::BIT(y, alt_reg(Reg::AtHL)),
                        (2, y, 6) => Token::RES(y, alt_reg(Reg::AtHL)),
                        (2, y, z) => Token::LDRES(alt_reg(Reg::from(z)), y, alt_reg(Reg::AtHL)),
                        (3, y, 6) => Token::SET(y, alt_reg(Reg::AtHL)),
                        (3, y, z) => Token::LDSET(alt_reg(Reg::from(z)), y, alt_reg(Reg::AtHL)),
                        (_, _, _) => unreachable!()
                    },
                    upnext: TokenType::Opcode
                }
            },

            Some(0xdd) | Some(0xfd) | None  => match (get_x(byte), get_y(byte), get_z(byte)) {

                // x=0, z=0
                (0, 0, 0) => DecodeResult { token: Token::NOP, upnext: TokenType::Opcode },
                (0, 1, 0) => DecodeResult { token: Token::EX_AF, upnext: TokenType::Opcode },
                (0, y, 0) => {
                    let offset_byte = yield DecodeResult {
                        token: match y {
                            2 => Token::DJNZ,
                            3 => Token::JR(Condition::None),
                            _ => Token::JR(Condition::from(y & 0b11))
                        },
                        upnext: TokenType::Offset
                    };
                    DecodeResult { token: Token::Offset(offset_byte as i8), upnext: TokenType::Opcode }
                },

                // x=0, z=1
                (0, _, 1) => {
                    let p = get_p(byte);
                    DecodeResult {
                        token: if get_q(byte) == 0 {
                            let low_operand_byte = yield DecodeResult {
                                token: Token::LD_RP_NN(alt_rpair(RegPair::from(p).prefer_sp())),
                                upnext: TokenType::Operand
                            };
                            let high_operand_byte = yield DecodeResult {
                                token: Token::Operand(OperandValue::Byte(low_operand_byte)),
                                upnext: TokenType::Operand
                            };
                            Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte)))
                        } else {
                            Token::ADD_RP_RP(alt_rpair(RegPair::HL), alt_rpair(RegPair::from(p).prefer_sp()))
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=2
                (0, 0, 2) => DecodeResult { token: Token::LD_AtRP_A(RegPair::BC), upnext: TokenType::Opcode },
                (0, 1, 2) => DecodeResult { token: Token::LD_A_AtRP(RegPair::BC), upnext: TokenType::Opcode },
                (0, 2, 2) => DecodeResult { token: Token::LD_AtRP_A(RegPair::DE), upnext: TokenType::Opcode },
                (0, 3, 2) => DecodeResult { token: Token::LD_A_AtRP(RegPair::DE), upnext: TokenType::Opcode },
                (0, y, 2) => {
                    let low_operand_byte = yield DecodeResult {
                        token: match y {
                            4 => Token::LD_MM_RP(alt_rpair(RegPair::HL)),
                            5 => Token::LD_RP_MM(alt_rpair(RegPair::HL)),
                            6 => Token::LD_MM_A,
                            7 => Token::LD_A_MM,
                            _ => unreachable!()
                        },
                        upnext: TokenType::Operand
                    };
                    let high_operand_byte = yield DecodeResult {
                        token: Token::Operand(OperandValue::Byte(low_operand_byte)),
                        upnext: TokenType::Operand
                    };
                    DecodeResult {
                        token: Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte))),
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=3
                (0, _, 3) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    let rp = alt_rpair(RegPair::from(p).prefer_sp());
                    DecodeResult {
                        token: if q == 0 { Token::INC_RP(rp) } else { Token::DEC_RP(rp) },
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=4,5
                (0, y, z @ (4 | 5)) => {
                    let opcode_token = if z == 4 {
                        Token::INC_RG(alt_reg(Reg::from(y)))
                    } else {
                        Token::DEC_RG(alt_reg(Reg::from(y)))
                    };
                    DecodeResult {
                        token: match prefix {
                            None => opcode_token,
                            Some(_) => {
                                let offset_byte = yield DecodeResult {
                                    token: opcode_token,
                                    upnext: TokenType::Offset
                                };
                                Token::Offset(offset_byte as i8)
                            }
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=6
                (0, y, 6) => {
                    match prefix {
                        None => {
                            let operand_byte = yield DecodeResult {
                                token: Token::LD_RG_N(alt_reg(Reg::from(y))),
                                upnext: TokenType::Operand
                            };
                            DecodeResult {
                                token: Token::Operand(OperandValue::Byte(operand_byte)),
                                upnext: TokenType::Opcode
                            }
                        },
                        Some(_) => {
                            let offset_byte = yield DecodeResult {
                                token: Token::LD_RG_N(alt_reg(Reg::from(y))),
                                upnext: TokenType::Offset
                            };
                            let operand_byte = yield DecodeResult {
                                token: Token::Offset(offset_byte as i8),
                                upnext: TokenType::Operand
                            };
                            DecodeResult {
                                token: Token::Operand(OperandValue::Byte(operand_byte)),
                                upnext: TokenType::Opcode
                            }
                        }
                    }
                },

                // x=0, z=7
                (0, 0, 7) => DecodeResult { token: Token::RLCA, upnext: TokenType::Opcode },
                (0, 1, 7) => DecodeResult { token: Token::RRCA, upnext: TokenType::Opcode },
                (0, 2, 7) => DecodeResult { token: Token::RLA, upnext: TokenType::Opcode },
                (0, 3, 7) => DecodeResult { token: Token::RRA, upnext: TokenType::Opcode },
                (0, 4, 7) => DecodeResult { token: Token::DAA, upnext: TokenType::Opcode },
                (0, 5, 7) => DecodeResult { token: Token::CPL, upnext: TokenType::Opcode },
                (0, 6, 7) => DecodeResult { token: Token::SCF, upnext: TokenType::Opcode },
                (0, 7, 7) => DecodeResult { token: Token::CCF, upnext: TokenType::Opcode },
                (0, _, 7) => unreachable!(),

                // x=1,2
                (x @ (1 | 2), y, z) => {
                    let opcode_token = if x == 1 {
                        let dst_reg = Reg::from(y);
                        let src_reg = Reg::from(z);
                        if dst_reg == Reg::AtHL && src_reg == Reg::AtHL {
                            Token::HALT // exception
                        } else if dst_reg == Reg::AtHL {
                            Token::LD_RG_RG(alt_reg(dst_reg), src_reg)
                        } else if src_reg == Reg::AtHL {
                            Token::LD_RG_RG(dst_reg, alt_reg(src_reg))
                        } else {
                            Token::LD_RG_RG(alt_reg(dst_reg), alt_reg(src_reg))
                        }
                    } else {
                        Token::ALU_RG(AluOp::from(y), alt_reg(Reg::from(z)))
                    };
                    match prefix {
                        None => DecodeResult {
                            token: opcode_token,
                            upnext: TokenType::Opcode
                        },
                        Some(_) => {
                            let offset_byte = yield DecodeResult {
                                token: opcode_token,
                                upnext: TokenType::Offset
                            };
                            DecodeResult {
                                token: Token::Offset(offset_byte as i8),
                                upnext: TokenType::Opcode
                            }
                        }
                    }
                },

                // x=3, z=0
                (3, y, 0) => DecodeResult {
                    token: Token::RET(Condition::from(y)),
                    upnext: TokenType::Opcode
                },

                // x=3, z=1
                (3, _, 1) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    DecodeResult {
                        token: if q == 0 {
                            Token::POP(alt_rpair(RegPair::from(p).prefer_af()))
                        } else {
                            match p {
                                0 => Token::RET(Condition::None),
                                1 => Token::EXX,
                                2 => Token::JP_RP(alt_rpair(RegPair::HL)),
                                3 => Token::LD_SP_RP(alt_rpair(RegPair::HL)),
                                _ => unreachable!()
                            }
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=3, z=2 &
                // x=3, y=0, z=3
                (3, y, z @ 2) | (3, y @ 0, z @ 3) => {
                    let low_operand_byte = yield DecodeResult {
                        token: Token::JP(if z == 2 { Condition::from(y) } else { Condition::None }),
                        upnext: TokenType::Operand
                    };
                    let high_operand_byte = yield DecodeResult {
                        token: Token::Operand(OperandValue::Byte(low_operand_byte)),
                        upnext: TokenType::Operand
                    };
                    DecodeResult {
                        token: Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte))),
                        upnext: TokenType::Opcode
                    }
                },

                // x=3, z=3
                (3, y @ (2 | 3), 3) => {
                    let port_byte = yield DecodeResult {
                        token: if y == 2 { Token::OUT_N_A } else { Token::IN_A_N },
                        upnext: TokenType::Operand
                    };
                    DecodeResult {
                        token: Token::Operand(OperandValue::Byte(port_byte)),
                        upnext: TokenType::Opcode
                    }
                },
                (3, 4, 3) => DecodeResult {
                    token: Token::EX_AtSP_RP(alt_rpair(RegPair::HL)),
                    upnext: TokenType::Opcode
                },
                (3, 5, 3) => DecodeResult { token: Token::EX_DE_HL, upnext: TokenType::Opcode },
                (3, 6, 3) => DecodeResult { token: Token::DI, upnext: TokenType::Opcode },
                (3, 7, 3) => DecodeResult { token: Token::EI, upnext: TokenType::Opcode },
                (3, _, 3) => unreachable!(),

                // x=3, z=4,5
                (3, y, z @ (4 | 5)) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    if z == 5 && q == 0 {
                        DecodeResult {
                            token: Token::PUSH(alt_rpair(RegPair::from(p).prefer_af())),
                            upnext: TokenType::Opcode
                        }
                    } else {
                        let low_operand_byte = yield DecodeResult {
                            token: Token::CALL(if z == 4 { Condition::from(y) } else { Condition::None }),
                            upnext: TokenType::Operand
                        };
                        let high_operand_byte = yield DecodeResult {
                            token: Token::Operand(OperandValue::Byte(low_operand_byte)),
                            upnext: TokenType::Operand
                        };
                        DecodeResult {
                            token: Token::Operand(OperandValue::Word(mkword!(high_operand_byte, low_operand_byte))),
                            upnext: TokenType::Opcode
                        }
                    }
                },

                // x=3, z=6
                (3, y, 6) => {
                    let operand_byte = yield DecodeResult {
                        token: Token::ALU_N(AluOp::from(y)),
                        upnext: TokenType::Operand
                    };
                    DecodeResult {
                        token: Token::Operand(OperandValue::Byte(operand_byte)),
                        upnext: TokenType::Opcode
                    }
                },

                // x=3, z=7
                (3, y, 7) => DecodeResult { token: Token::RST(y), upnext: TokenType::Opcode },

                (_, _, _) => unreachable!()

            },

            _ => unreachable!()

        }

    }

}

/// Create generator which accepts bytes, yields decoded
/// prefix tokens and completes on first non-prefix token
pub fn prefix_decoder() -> impl Generator<u8, Yield=DecodeResult> {

    |mut byte: u8| {

        let mut current_prefix: Option<u16> = None;

        loop {

            let (prefix, upnext) = match current_prefix {

                // CB & ED are always followed by opcode byte
                Some(0xcb) | Some(0xed) => return,

                // DDCB & FDCB are always followed by offset (aka displacement) byte
                Some(0xcbdd) | Some(0xcbfd) => return,

                Some(val) if (val == 0xdd || val == 0xfd) => match byte {

                    // If DD or FD followed by DD, ED or FD we should ignore former prefix
                    0xed | 0xdd | 0xfd => (byte as u16, TokenType::Opcode),

                    // DD or FD followed by CB gives DDCB or FDCB
                    0xcb => (mkword!(byte, val), TokenType::Offset),

                    // Otherwise it is followed by opcode
                    _ => return

                },

                _ => match byte {

                    // All prefixes start with CB, ED, DD or FD
                    0xcb | 0xed | 0xdd | 0xfd => (byte as u16, TokenType::Opcode),

                    // Otherwise it is an opcode
                    _ => return

                }

            };

            // Yield prefix token and advance to the next byte
            byte = yield DecodeResult { token: Token::Prefix(prefix), upnext };

            current_prefix = Some(prefix);

        }

    }

}
