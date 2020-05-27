use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use super::tokens::*;

const X_MASK: u8 = 0b11000000;
const Y_MASK: u8 = 0b00111000;
const Z_MASK: u8 = 0b00000111;
const P_MASK: u8 = 0b00110000;
const Q_MASK: u8 = 0b00001000;

const X_SHIFT: u8 = 6;
const Y_SHIFT: u8 = 3;
const Z_SHIFT: u8 = 0;
const P_SHIFT: u8 = 4;
const Q_SHIFT: u8 = 3;

/// Merge high and low u8 bytes to a u18 word
macro_rules! word {
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

        let x = (byte & X_MASK) >> X_SHIFT;
        let y = (byte & Y_MASK) >> Y_SHIFT;
        let z = (byte & Z_MASK) >> Z_SHIFT;
        let p = (byte & P_MASK) >> P_SHIFT;
        let q = (byte & Q_MASK) >> Q_SHIFT;

        match prefix {

            Some(0xed) => match x {
                1 => match z {
                    0 => {
                        if y == 6 {
                            Token::IN_AtBC
                        } else {
                            Token::IN_RG_AtBC(Reg::from(y))
                        }
                    },
                    1 => {
                        if y == 6 {
                            Token::OUT_AtBC_0
                        } else {
                            Token::OUT_AtBC_RG(Reg::from(y))
                        }
                    },
                    2 => {
                        if q == 0 {
                            Token::SBC_HL_RP(RegPair::from(p))
                        } else {
                            Token::ADC_HL_RP(RegPair::from(p))
                        }
                    },
                    3 => {
                        let low = yield if q == 0 {
                            Token::LD_AtMM_RP(RegPair::from(p))
                        } else {
                            Token::LD_RP_AtMM(RegPair::from(p))
                        };
                        let high = yield Token::Operand(low);
                        Token::Operand(high)
                    },
                    4 => Token::NEG,
                    5 => if y == 1 { Token::RETI } else { Token::RETN },
                    6 => Token::IM(IntMode::from(y)),
                    7 => match y {
                        0 => Token::LD_I_A,
                        1 => Token::LD_R_A,
                        2 => Token::LD_A_I,
                        3 => Token::LD_A_R,
                        4 => Token::RRD,
                        5 => Token::RLD,
                        _ => Token::NOP
                    },
                    _ => unreachable!()
                },
                2 => {
                    if z <= 3 && y >= 4 {
                        Token::BLI(BlockOp::from((y << 2 ) | z))
                    } else {
                        Token::NOP
                    }
                },
                _ => Token::NOP
            },

            Some(0xcb) => match x {
                0 => Token::SH(ShiftOp::from(y), Reg::from(z)),
                1 => Token::BIT(y, Reg::from(z)),
                2 => Token::RES(y, Reg::from(z)),
                3 => Token::SET(y, Reg::from(z)),
                _ => unreachable!()
            },

            _ => match byte & X_MASK & Z_MASK {

                // x=0, z=0
                0o000 => match y {
                    0 => Token::NOP,
                    1 => Token::EX_AF,
                    _ => {
                        let offset = yield match y {
                            2 => Token::DJNZ,
                            3 => Token::JR(Condition::None),
                            _ => Token::JR(Condition::from(y & 0b11))
                        };
                        Token::Offset(offset as i8)
                    }
                },
    
                // x=0, z=1
                0o001 => {
                    if q == 0 {
                        let low = yield Token::LD_RP_NN(RegPair::from(p));
                        let high = yield Token::Operand(low);
                        Token::Operand(high)
                    } else {
                        Token::ADD_HL_RP(RegPair::from(p))
                    }
                },
    
                // x=0, z=2
                0o002 => match y {
                    0 => Token::LD_RP_A(RegPair::BC),
                    1 => Token::LD_A_RP(RegPair::BC),
                    2 => Token::LD_RP_A(RegPair::DE),
                    3 => Token::LD_A_RP(RegPair::DE),
                    _ => {
                        let low = yield match y {
                            4 => Token::LD_MM_HL,
                            5 => Token::LD_HL_MM,
                            6 => Token::LD_MM_A,
                            7 => Token::LD_A_MM,
                            _ => unreachable!()
                        };
                        let high = yield Token::Operand(low);
                        Token::Operand(high)
                    }
                },
    
                // x=0, z=3
                0o003 => {
                    let rp = RegPair::from(p);
                    if q == 0 { Token::INC_RP(rp) } else { Token::DEC_RP(rp) }
                },
    
                // x=0, z=4
                0o004 => Token::INC_RG(Reg::from(y)),
    
                // x=0, z=5
                0o005 => Token::DEC_RG(Reg::from(y)),
    
                // x=0, z=6
                0o006 => {
                    let operand = yield Token::LD_RG_N(Reg::from(y));
                    Token::Operand(operand)
                },
    
                // x=0, z=7
                0o007 => match y {
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
                0o300 => Token::RET(Condition::from(y)),
    
                // x=3, z=1
                0o301 => {
                    if q == 0 {
                        let mut rp = RegPair::from(p);
                        if rp == RegPair::SPorAF { // TODO: avoid if
                            rp = RegPair::AF;
                        }
                        Token::POP(rp)
                    } else {
                        match p {
                            0 => Token::RET(Condition::None),
                            1 => Token::EXX,
                            2 => Token::JP_HL,
                            3 => Token::LD_SP_HL,
                            _ => unreachable!()
                        }
                    }
                },
    
                // x=3, z=2
                0o302 => {
                    let low = yield Token::JP(Condition::from(y));
                    let high = yield Token::Operand(low);
                    Token::Operand(high)
                },
    
                // x=3, z=3
                0o303 => match y {
                    0 => {
                        let low = yield Token::JP(Condition::None);
                        let high = yield Token::Operand(low);
                        Token::Operand(high)
                    },
                    2 | 3 => {
                        let port = yield if y == 2 { Token::OUT_N_A } else { Token::IN_A_N };
                        Token::Operand(port)
                    },
                    4 => Token::EX_AtSP_HL,
                    5 => Token::EX_DE_HL,
                    6 => Token::DI,
                    7 => Token::EI,
                    _ => unreachable!()
                },
    
                // x=3, z=4
                0o304 => {
                    let low = yield Token::CALL(Condition::from(y));
                    let high = yield Token::Operand(low);
                    Token::Operand(high)
                },
    
                // x=3, z=5
                0o305 => {
                    if q == 0 {
                        let mut rp = RegPair::from(p);
                        if rp == RegPair::SPorAF { // TODO: avoid if
                            rp = RegPair::AF;
                        }
                        Token::PUSH(rp)
                    } else {
                        let low = yield Token::CALL(Condition::None);
                        let high = yield Token::Operand(low);
                        Token::Operand(high)
                    }
                },
    
                // x=3, z=6
                0o306 => {
                    let operand = yield Token::ALU_N(AluOp::from(y));
                    Token::Operand(operand)
                },
    
                // x=3, z=7
                0o307 => Token::RST(y * 8),
    
                _ => match byte & X_MASK {
    
                    // x=1
                    1 => {
                        if (byte & Y_MASK & Z_MASK) == 0o66 {
                            Token::HALT
                        } else {
                            Token::LD_RG_RG(Reg::from(y), Reg::from(z))
                        }
                    },
    
                    // x=2
                    2 => Token::ALU_RG(AluOp::from(y), Reg::from(z)),
    
                    _ => unreachable!()
    
                }
    
            }

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
                Some(0xddcb) | Some(0xfdcb) => return,

                Some(val) if (val == 0xdd || val == 0xfd) => match byte {

                    // If DD or FD followed by DD, ED or FD we should ignore former prefix
                    0xdd | 0xed | 0xfd => byte as u16,

                    // DD or FD followed by CB gives DDCB or FDCB
                    0xcb => word!(val, byte),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_decoder_is_able_to_parse_cb_prefix() {
        let mut decoder = prefix_decoder();
        let a = Pin::new(&mut decoder).resume(0xcb);
        // assert_eq!(a, GeneratorState::Yielded(Token::Prefix(0xcb)));
        let b = Pin::new(&mut decoder).resume(0x0);
        // assert_eq!(b, GeneratorState::Complete(Token::Opcode(0)));
    }

    #[test]
    fn prefix_decoder_is_able_to_parse_ed_prefix() {
        let mut decoder = prefix_decoder();
        let bytes: Vec<u8> = vec![0xed, 0x00];
        // let tok = Pin::new(&mut decoder).resume(1);
    }

}
