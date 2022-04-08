use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use crate::{
    mkword,
    cpu::tokens::{
        Token, TokenType, DataValue, Reg, RegPair,
        IntMode, BlockOp, ShiftOp, AluOp, Condition
    },
};

use super::Instruction;

/// Result of decoding a byte into a token
pub struct TokenDecodeResult {
    /// Decoded token
    pub token: Token,
    /// Expected type for the next token
    pub upnext: TokenType,
}

/// Create generator which accepts bytes, yields decoded
/// tokens and returns cpu instruction when entire instruction sequence is decoded
pub fn instruction_decoder() -> impl Generator<u8, Yield=TokenDecodeResult, Return=Instruction> {

    let mut tdecoder = token_decoder();
    let mut displacement: Option<i8> = None;
    let mut data: Option<DataValue> = None;
    let mut opcode: Option<Token> = None;

    move |mut byte: u8| {

        loop {

            let state = Pin::new(&mut tdecoder).resume(byte);

            let (complete, result) = match state {
                GeneratorState::Yielded(result) => (false, result),
                GeneratorState::Complete(result) => (true, result),
            };

            match result.token {
                Token::Prefix(_) => (),
                Token::Displacement(value) => displacement = Some(value),
                Token::Data(value) => data = Some(value),
                token => opcode = Some(token)
            }

            if complete {
                return Instruction {
                    opcode: opcode.expect("Expecting opcode to be defined"),
                    displacement,
                    data,
                };
            } else {
                byte = yield result;
            }

        }

    }

}

fn get_x(byte: u8) -> u8 { (byte & 0b11000000) >> 6 }
fn get_y(byte: u8) -> u8 { (byte & 0b00111000) >> 3 }
fn get_z(byte: u8) -> u8 {  byte & 0b00000111       }
fn get_p(byte: u8) -> u8 { (byte & 0b00110000) >> 4 }
fn get_q(byte: u8) -> u8 { (byte & 0b00001000) >> 3 }

/// Create generator which accepts bytes, yields decoded
/// tokens and returns when entire instruction sequence is decoded
fn token_decoder() -> impl Generator<u8, Yield=TokenDecodeResult, Return=TokenDecodeResult> {

    |mut byte: u8| {

        let mut prefix: Option<u16> = None;

        {
            // Z80 opcode may have multiple (possibly) duplicate and overriding each other prefixes.
            // Here we try to interpret incoming bytes as prefix bytes until we reach actual opcode
            // or displacement byte
            let mut pdecoder = prefix_decoder();
            while let GeneratorState::Yielded(result) = Pin::new(&mut pdecoder).resume(byte) {
                if let TokenDecodeResult { token: Token::Prefix(code), .. } = result {
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
                (1, 6, 0) => TokenDecodeResult { token: Token::IN_AtBC, upnext: TokenType::Opcode },
                (1, y, 0) => TokenDecodeResult { token: Token::IN_RG_AtBC(Reg::from(y)), upnext: TokenType::Opcode },
                (1, 6, 1) => TokenDecodeResult { token: Token::OUT_AtBC_0, upnext: TokenType::Opcode },
                (1, y, 1) => TokenDecodeResult { token: Token::OUT_AtBC_RG(Reg::from(y)), upnext: TokenType::Opcode },
                (1, _, 2) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    TokenDecodeResult {
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
                    let low_data_byte = yield TokenDecodeResult {
                        token: if q == 0 {
                            Token::LD_MM_RP(RegPair::from(p).prefer_sp())
                        } else {
                            Token::LD_RP_MM(RegPair::from(p).prefer_sp())
                        },
                        upnext: TokenType::Data
                    };
                    let high_data_byte = yield TokenDecodeResult {
                        token: Token::Data(DataValue::Byte(low_data_byte)),
                        upnext: TokenType::Data
                    };
                    TokenDecodeResult {
                        token: Token::Data(DataValue::Word(mkword!(high_data_byte, low_data_byte))),
                        upnext: TokenType::Opcode
                    }
                },
                (1, _, 4) => TokenDecodeResult { token: Token::NEG, upnext: TokenType::Opcode },
                (1, 1, 5) => TokenDecodeResult { token: Token::RETI, upnext: TokenType::Opcode },
                (1, _, 5) => TokenDecodeResult { token: Token::RETN, upnext: TokenType::Opcode },
                (1, y, 6) => TokenDecodeResult { token: Token::IM(IntMode::from(y)), upnext: TokenType::Opcode },
                (1, 0, 7) => TokenDecodeResult { token: Token::LD_RG_RG(Reg::I, Reg::A), upnext: TokenType::Opcode },
                (1, 1, 7) => TokenDecodeResult { token: Token::LD_RG_RG(Reg::R, Reg::A), upnext: TokenType::Opcode },
                (1, 2, 7) => TokenDecodeResult { token: Token::LD_RG_RG(Reg::A, Reg::I), upnext: TokenType::Opcode },
                (1, 3, 7) => TokenDecodeResult { token: Token::LD_RG_RG(Reg::A, Reg::R), upnext: TokenType::Opcode },
                (1, 4, 7) => TokenDecodeResult { token: Token::RRD, upnext: TokenType::Opcode },
                (1, 5, 7) => TokenDecodeResult { token: Token::RLD, upnext: TokenType::Opcode },
                (1, _, 7) => TokenDecodeResult { token: Token::NOP, upnext: TokenType::Opcode },
                (1, _, _) => unreachable!(),
                (2, y, z) if z <= 3 && y >= 4 => TokenDecodeResult {
                    token: Token::BLOP(BlockOp::from((y << 2 ) | z)),
                    upnext: TokenType::Opcode
                },
                (2, _, _) => TokenDecodeResult { token: Token::NOP, upnext: TokenType::Opcode },
                (_, _, _) => TokenDecodeResult { token: Token::NOP, upnext: TokenType::Opcode }
            },

            Some(0xcb) => {
                let (y, z) = (get_y(byte), get_z(byte));
                match get_x(byte) {
                    0 => TokenDecodeResult { token: Token::SHOP(ShiftOp::from(y), Reg::from(z)), upnext: TokenType::Opcode },
                    1 => TokenDecodeResult { token: Token::BIT(y, Reg::from(z)), upnext: TokenType::Opcode },
                    2 => TokenDecodeResult { token: Token::RES(y, Reg::from(z)), upnext: TokenType::Opcode },
                    3 => TokenDecodeResult { token: Token::SET(y, Reg::from(z)), upnext: TokenType::Opcode },
                    _ => unreachable!()
                }
            },

            Some(0xcbdd) | Some(0xcbfd) => {
                // First byte after DDCB/FDCB is a displacement byte and then opcode follows
                byte = yield TokenDecodeResult { token: Token::Displacement(byte as i8), upnext: TokenType::Opcode };
                TokenDecodeResult {
                    token: match (get_x(byte), get_y(byte), get_z(byte)) {
                        (0, y, 6) => Token::SHOP(ShiftOp::from(y), alt_reg(Reg::AtHL)),
                        (0, y, z) => Token::SHOPLD(ShiftOp::from(y), alt_reg(Reg::AtHL), Reg::from(z)),
                        (1, y, _) => Token::BIT(y, alt_reg(Reg::AtHL)),
                        (2, y, 6) => Token::RES(y, alt_reg(Reg::AtHL)),
                        (2, y, z) => Token::RESLD(y, alt_reg(Reg::AtHL), Reg::from(z)),
                        (3, y, 6) => Token::SET(y, alt_reg(Reg::AtHL)),
                        (3, y, z) => Token::SETLD(y, alt_reg(Reg::AtHL), Reg::from(z)),
                        (_, _, _) => unreachable!()
                    },
                    upnext: TokenType::Opcode
                }
            },

            Some(0xdd) | Some(0xfd) | None  => match (get_x(byte), get_y(byte), get_z(byte)) {

                // x=0, z=0
                (0, 0, 0) => TokenDecodeResult { token: Token::NOP, upnext: TokenType::Opcode },
                (0, 1, 0) => TokenDecodeResult { token: Token::EX_AF, upnext: TokenType::Opcode },
                (0, y, 0) => {
                    let displacement_byte = yield TokenDecodeResult {
                        token: match y {
                            2 => Token::DJNZ,
                            3 => Token::JR(Condition::None),
                            _ => Token::JR(Condition::from(y & 0b11))
                        },
                        upnext: TokenType::Displacement
                    };
                    TokenDecodeResult { token: Token::Displacement(displacement_byte as i8), upnext: TokenType::Opcode }
                },

                // x=0, z=1
                (0, _, 1) => {
                    let p = get_p(byte);
                    TokenDecodeResult {
                        token: if get_q(byte) == 0 {
                            let low_data_byte = yield TokenDecodeResult {
                                token: Token::LD_RP_NN(alt_rpair(RegPair::from(p).prefer_sp())),
                                upnext: TokenType::Data
                            };
                            let high_data_byte = yield TokenDecodeResult {
                                token: Token::Data(DataValue::Byte(low_data_byte)),
                                upnext: TokenType::Data
                            };
                            Token::Data(DataValue::Word(mkword!(high_data_byte, low_data_byte)))
                        } else {
                            Token::ADD_RP_RP(alt_rpair(RegPair::HL), alt_rpair(RegPair::from(p).prefer_sp()))
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=2
                (0, 0, 2) => TokenDecodeResult { token: Token::LD_AtRP_A(RegPair::BC), upnext: TokenType::Opcode },
                (0, 1, 2) => TokenDecodeResult { token: Token::LD_A_AtRP(RegPair::BC), upnext: TokenType::Opcode },
                (0, 2, 2) => TokenDecodeResult { token: Token::LD_AtRP_A(RegPair::DE), upnext: TokenType::Opcode },
                (0, 3, 2) => TokenDecodeResult { token: Token::LD_A_AtRP(RegPair::DE), upnext: TokenType::Opcode },
                (0, y, 2) => {
                    let low_data_byte = yield TokenDecodeResult {
                        token: match y {
                            4 => Token::LD_MM_RP(alt_rpair(RegPair::HL)),
                            5 => Token::LD_RP_MM(alt_rpair(RegPair::HL)),
                            6 => Token::LD_MM_A,
                            7 => Token::LD_A_MM,
                            _ => unreachable!()
                        },
                        upnext: TokenType::Data
                    };
                    let high_data_byte = yield TokenDecodeResult {
                        token: Token::Data(DataValue::Byte(low_data_byte)),
                        upnext: TokenType::Data
                    };
                    TokenDecodeResult {
                        token: Token::Data(DataValue::Word(mkword!(high_data_byte, low_data_byte))),
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=3
                (0, _, 3) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    let rp = alt_rpair(RegPair::from(p).prefer_sp());
                    TokenDecodeResult {
                        token: if q == 0 { Token::INC_RP(rp) } else { Token::DEC_RP(rp) },
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=4,5
                (0, y, z @ (4 | 5)) => {
                    let reg = Reg::from(y);
                    let opcode_token = if z == 4 {
                        Token::INC_RG(alt_reg(reg))
                    } else {
                        Token::DEC_RG(alt_reg(reg))
                    };
                    TokenDecodeResult {
                        token: if prefix.is_some() && reg == Reg::AtHL {
                            let displacement_byte = yield TokenDecodeResult {
                                token: opcode_token,
                                upnext: TokenType::Displacement
                            };
                            Token::Displacement(displacement_byte as i8)
                        } else {
                            opcode_token
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=6
                (0, y, 6) => {
                    let reg = Reg::from(y);
                    let opcode_token = Token::LD_RG_N(alt_reg(reg));
                    let data_byte = yield TokenDecodeResult {
                        token: if prefix.is_some() && reg == Reg::AtHL {
                            let displacement_byte = yield TokenDecodeResult {
                                token: opcode_token,
                                upnext: TokenType::Displacement
                            };
                            Token::Displacement(displacement_byte as i8)
                        } else {
                            opcode_token
                        },
                        upnext: TokenType::Data
                    };
                    TokenDecodeResult {
                        token: Token::Data(DataValue::Byte(data_byte)),
                        upnext: TokenType::Opcode
                    }
                },

                // x=0, z=7
                (0, 0, 7) => TokenDecodeResult { token: Token::RLCA, upnext: TokenType::Opcode },
                (0, 1, 7) => TokenDecodeResult { token: Token::RRCA, upnext: TokenType::Opcode },
                (0, 2, 7) => TokenDecodeResult { token: Token::RLA, upnext: TokenType::Opcode },
                (0, 3, 7) => TokenDecodeResult { token: Token::RRA, upnext: TokenType::Opcode },
                (0, 4, 7) => TokenDecodeResult { token: Token::DAA, upnext: TokenType::Opcode },
                (0, 5, 7) => TokenDecodeResult { token: Token::CPL, upnext: TokenType::Opcode },
                (0, 6, 7) => TokenDecodeResult { token: Token::SCF, upnext: TokenType::Opcode },
                (0, 7, 7) => TokenDecodeResult { token: Token::CCF, upnext: TokenType::Opcode },
                (0, _, 7) => unreachable!(),

                // x=1
                (1, y, z) => {
                    let dst_reg = Reg::from(y);
                    let src_reg = Reg::from(z);
                    let opcode_token = if dst_reg == Reg::AtHL && src_reg == Reg::AtHL {
                        Token::HALT // exception
                    } else if dst_reg == Reg::AtHL {
                        Token::LD_RG_RG(alt_reg(dst_reg), src_reg)
                    } else if src_reg == Reg::AtHL {
                        Token::LD_RG_RG(dst_reg, alt_reg(src_reg))
                    } else {
                        Token::LD_RG_RG(alt_reg(dst_reg), alt_reg(src_reg))
                    };
                    TokenDecodeResult {
                        token: if prefix.is_some() && (dst_reg == Reg::AtHL || src_reg == Reg::AtHL) {
                            let displacement_byte = yield TokenDecodeResult {
                                token: opcode_token,
                                upnext: TokenType::Displacement
                            };
                            Token::Displacement(displacement_byte as i8)
                        } else {
                            opcode_token
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=2
                (2, y, z) => {
                    let reg = Reg::from(z);
                    let opcode_token = Token::ALU(AluOp::from(y), Some(alt_reg(reg)));
                    TokenDecodeResult {
                        token: if prefix.is_some() && reg == Reg::AtHL {
                            let displacement_byte = yield TokenDecodeResult {
                                token: opcode_token,
                                upnext: TokenType::Displacement
                            };
                            Token::Displacement(displacement_byte as i8)
                        } else {
                            opcode_token
                        },
                        upnext: TokenType::Opcode
                    }
                },

                // x=3, z=0
                (3, y, 0) => TokenDecodeResult {
                    token: Token::RET(Condition::from(y)),
                    upnext: TokenType::Opcode
                },

                // x=3, z=1
                (3, _, 1) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    TokenDecodeResult {
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
                    let low_data_byte = yield TokenDecodeResult {
                        token: Token::JP(if z == 2 { Condition::from(y) } else { Condition::None }),
                        upnext: TokenType::Data
                    };
                    let high_data_byte = yield TokenDecodeResult {
                        token: Token::Data(DataValue::Byte(low_data_byte)),
                        upnext: TokenType::Data
                    };
                    TokenDecodeResult {
                        token: Token::Data(DataValue::Word(mkword!(high_data_byte, low_data_byte))),
                        upnext: TokenType::Opcode
                    }
                },

                // x=3, z=3
                (3, y @ (2 | 3), 3) => {
                    let port_byte = yield TokenDecodeResult {
                        token: if y == 2 { Token::OUT_N_A } else { Token::IN_A_N },
                        upnext: TokenType::Data
                    };
                    TokenDecodeResult {
                        token: Token::Data(DataValue::Byte(port_byte)),
                        upnext: TokenType::Opcode
                    }
                },
                (3, 4, 3) => TokenDecodeResult {
                    token: Token::EX_AtSP_RP(alt_rpair(RegPair::HL)),
                    upnext: TokenType::Opcode
                },
                (3, 5, 3) => TokenDecodeResult { token: Token::EX_DE_HL, upnext: TokenType::Opcode },
                (3, 6, 3) => TokenDecodeResult { token: Token::DI, upnext: TokenType::Opcode },
                (3, 7, 3) => TokenDecodeResult { token: Token::EI, upnext: TokenType::Opcode },
                (3, _, 3) => unreachable!(),

                // x=3, z=4,5
                (3, y, z @ (4 | 5)) => {
                    let (p, q) = (get_p(byte), get_q(byte));
                    if z == 5 && q == 0 {
                        TokenDecodeResult {
                            token: Token::PUSH(alt_rpair(RegPair::from(p).prefer_af())),
                            upnext: TokenType::Opcode
                        }
                    } else {
                        let low_data_byte = yield TokenDecodeResult {
                            token: Token::CALL(if z == 4 { Condition::from(y) } else { Condition::None }),
                            upnext: TokenType::Data
                        };
                        let high_data_byte = yield TokenDecodeResult {
                            token: Token::Data(DataValue::Byte(low_data_byte)),
                            upnext: TokenType::Data
                        };
                        TokenDecodeResult {
                            token: Token::Data(DataValue::Word(mkword!(high_data_byte, low_data_byte))),
                            upnext: TokenType::Opcode
                        }
                    }
                },

                // x=3, z=6
                (3, y, 6) => {
                    let data_byte = yield TokenDecodeResult {
                        token: Token::ALU(AluOp::from(y), None),
                        upnext: TokenType::Data
                    };
                    TokenDecodeResult {
                        token: Token::Data(DataValue::Byte(data_byte)),
                        upnext: TokenType::Opcode
                    }
                },

                // x=3, z=7
                (3, y, 7) => TokenDecodeResult { token: Token::RST(y), upnext: TokenType::Opcode },

                (_, _, _) => unreachable!()

            },

            _ => unreachable!()

        }

    }

}

/// Create generator which accepts bytes, yields decoded
/// prefix tokens and completes on first non-prefix token
fn prefix_decoder() -> impl Generator<u8, Yield=TokenDecodeResult> {

    |mut byte: u8| {

        let mut current_prefix: Option<u16> = None;

        loop {

            let (prefix, upnext) = match current_prefix {

                // CB & ED are always followed by opcode byte
                Some(0xcb) | Some(0xed) => return,

                // DDCB & FDCB are always followed by displacement byte
                Some(0xcbdd) | Some(0xcbfd) => return,

                Some(val @ (0xdd | 0xfd)) => match byte {

                    // If DD or FD followed by DD, ED or FD we should ignore former prefix
                    0xed | 0xdd | 0xfd => (byte as u16, TokenType::Opcode),

                    // DD or FD followed by CB gives DDCB or FDCB
                    0xcb => (mkword!(byte, val), TokenType::Displacement),

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
            byte = yield TokenDecodeResult { token: Token::Prefix(prefix), upnext };

            current_prefix = Some(prefix);

        }

    }

}
