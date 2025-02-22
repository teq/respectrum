use std::{
    rc::Rc,
    pin::Pin,
    cell::Cell,
    ops::{Coroutine, CoroutineState},
};

use crate::{
    mkword, spword, yield_from,
    bus::{NoReturnTask, Clock, CpuBus, Task, Ctrl},
    cpu::{
        tokens::{Token, TokenType, Reg, RegPair, BlockOp, AluOp, ShiftOp, IntMode, Condition},
        decoder::{instruction_decoder, Instruction},
        Flags,
    },
    misc::{U16Cell, Identifiable},
};

use super::Device;

/// Z80 CPU
#[derive(Default)]
pub struct Cpu {
    pub af: U16Cell,
    pub bc: U16Cell,
    pub de: U16Cell,
    pub hl: U16Cell,
    pub alt_af: U16Cell,
    pub alt_bc: U16Cell,
    pub alt_de: U16Cell,
    pub alt_hl: U16Cell,
    pub ix: U16Cell,
    pub iy: U16Cell,
    pub sp: U16Cell,
    pub pc: U16Cell,
    pub ir: U16Cell,
    pub iff1: Cell<bool>,
    pub iff2: Cell<bool>,
    pub im: Cell<IntMode>,
    pub int: Cell<bool>,
    pub nmi: Cell<bool>,
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
}

impl Identifiable for Cpu {
    fn id(&self) -> u32 { 1 }
}

impl Device for Cpu {

    /// Run CPU device task
    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(#[coroutine] move || {

            self.bus.m1.drive(self, false);
            self.bus.busak.drive(self, false);
            self.bus.halt.drive(self, false);

            // Instruction loop
            loop {

                let mut decoder = instruction_decoder();
                let mut upnext = TokenType::Opcode;

                // Instruction decode loop
                let instruction = loop {

                    let pc = self.rp(RegPair::PC).get();

                    // Process possible interrupts
                    if self.nmi.get() || self.int.get() {
                        self.bus.halt.drive(self, false);
                    }

                    if self.nmi.get() {
                        self.nmi.set(false);
                        self.iff1.set(false);
                        unimplemented!();
                    } else if self.int.get() && self.iff1.get() {
                        self.int.set(false);
                        self.iff1.set(false);
                        self.iff2.set(false);
                        unimplemented!();
                    }

                    // Read the next byte using appropriate M-cycle
                    let byte: u8 = match upnext {
                        TokenType::Opcode => yield_from!(self.opcode_read(pc)),
                        TokenType::Displacement | TokenType::Data => yield_from!(self.memory_read(pc))
                    };

                    match Pin::new(&mut decoder).resume(byte) {
                        CoroutineState::Yielded(result) => upnext = result.upnext,
                        CoroutineState::Complete(instruction) => break instruction
                    }

                    self.rp(RegPair::PC).set(pc.wrapping_add(1));
                };

                // Process instruction
                match instruction.opcode {

                    // 8-bit Load

                    Token::LD_RG_RG(dst @ (Reg::AtIX | Reg::AtIY), src) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.idx_addr(dst, instruction.displacement.unwrap());
                        yield_from!(self.memory_write(addr, self.rg(src).get()));
                    },
                    Token::LD_RG_RG(dst, src @ (Reg::AtIX | Reg::AtIY)) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.idx_addr(src, instruction.displacement.unwrap());
                        self.rg(dst).set(yield_from!(self.memory_read(addr)));
                    },
                    Token::LD_RG_RG(Reg::AtHL, src) => {
                        let addr = self.rp(RegPair::HL).get();
                        yield_from!(self.memory_write(addr, self.rg(src).get()));
                    },
                    Token::LD_RG_RG(dst, Reg::AtHL) => {
                        let addr = self.rp(RegPair::HL).get();
                        self.rg(dst).set(yield_from!(self.memory_read(addr)))
                    },
                    Token::LD_RG_RG(dst @ (Reg::I | Reg::R), Reg::A) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        self.rg(dst).set(self.rg(Reg::A).get());
                    },
                    Token::LD_RG_RG(Reg::A, src @ (Reg::I | Reg::R)) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        let value = self.rg(src).get();
                        self.rg(Reg::A).set(value);
                        let mut flags = (self.get_flags() & Flags::C) | (Flags::from(value) & Flags::XY);
                        flags.set_zs_flags_u8(value);
                        flags.set(Flags::P, self.iff2.get());
                        self.set_flags(flags);
                    },
                    Token::LD_RG_RG(dst, src) => {
                        self.rg(dst).set(self.rg(src).get());
                    },
                    Token::LD_RG_N(Reg::AtHL) => {
                        let addr = self.rp(RegPair::HL).get();
                        yield_from!(self.memory_write(addr, instruction.expect_byte_data()));
                    },
                    Token::LD_RG_N(reg) => {
                        self.rg(reg).set(instruction.expect_byte_data());
                    },
                    Token::LD_A_AtRP(rpair) => {
                        let addr = self.rp(rpair).get();
                        self.rg(Reg::A).set(yield_from!(self.memory_read(addr)));
                    },
                    Token::LD_AtRP_A(rpair) => {
                        let addr = self.rp(rpair).get();
                        yield_from!(self.memory_write(addr, self.rg(Reg::A).get()));
                    },
                    Token::LD_A_MM => {
                        let addr = instruction.expect_word_data();
                        self.rg(Reg::A).set(yield_from!(self.memory_read(addr)));
                    },
                    Token::LD_MM_A => {
                        let addr = instruction.expect_word_data();
                        yield_from!(self.memory_write(addr, self.rg(Reg::A).get()));
                    },

                    // 16-bit Load

                    Token::LD_RP_NN(rpair) => {
                        self.rp(rpair).set(instruction.expect_word_data());
                    },
                    Token::LD_RP_MM(rpair) => {
                        let addr = instruction.expect_word_data();
                        let lo = yield_from!(self.memory_read(addr));
                        let hi = yield_from!(self.memory_read(addr + 1));
                        self.rp(rpair).set(mkword!(hi, lo));
                    },
                    Token::LD_MM_RP(rpair) => {
                        let addr = instruction.expect_word_data();
                        let (hi, lo) = spword!(self.rp(rpair).get());
                        yield_from!(self.memory_write(addr, lo));
                        yield_from!(self.memory_write(addr + 1, hi));
                    },
                    Token::LD_SP_RP(rpair) => {
                        yield self.clock.rising(2); // complement M1 to 6 t-cycles
                        self.rp(RegPair::SP).set(self.rp(rpair).get());
                    },
                    Token::POP(rpair) => {
                        self.rp(rpair).set(yield_from!(self.stack_pop()));
                    },
                    Token::PUSH(rpair) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        yield_from!(self.stack_push(self.rp(rpair).get()));
                    },

                    // Exchange

                    Token::EX_DE_HL => self.swap_hlde(),
                    Token::EX_AF => self.swap_acc(),
                    Token::EXX => self.swap_regfile(),
                    Token::EX_AtSP_RP(rpair) => {
                        let addr = self.rp(RegPair::SP).get();
                        let rd_lo = yield_from!(self.memory_read(addr));
                        let rd_hi = yield_from!(self.memory_read(addr + 1));
                        yield self.clock.rising(1);
                        let (wr_hi, wr_lo) = spword!(self.rp(rpair).get());
                        self.rp(rpair).set(mkword!(rd_hi, rd_lo));
                        yield_from!(self.memory_write(addr + 1, wr_hi));
                        yield_from!(self.memory_write(addr, wr_lo));
                        yield self.clock.rising(2);
                    },

                    // Block transfer, search group

                    Token::BLOP(BlockOp::LDI)  => yield_from!(self.memory_copy_block( 1, false)),
                    Token::BLOP(BlockOp::LDD)  => yield_from!(self.memory_copy_block(-1, false)),
                    Token::BLOP(BlockOp::LDIR) => yield_from!(self.memory_copy_block( 1, true)),
                    Token::BLOP(BlockOp::LDDR) => yield_from!(self.memory_copy_block(-1, true)),
                    Token::BLOP(BlockOp::CPI)  => yield_from!(self.memory_cmp_block ( 1, false)),
                    Token::BLOP(BlockOp::CPD)  => yield_from!(self.memory_cmp_block (-1, false)),
                    Token::BLOP(BlockOp::CPIR) => yield_from!(self.memory_cmp_block ( 1, true)),
                    Token::BLOP(BlockOp::CPDR) => yield_from!(self.memory_cmp_block (-1, true)),

                    // 8-bit arithmetic and logic

                    Token::ALU(op, maybe_reg) => {
                        let lhs = self.rg(Reg::A).get();
                        let rhs = if let Some(reg) = maybe_reg {
                            self.rg(reg).get()
                        } else {
                            instruction.expect_byte_data()
                        };
                        let mut flags = self.get_flags() & Flags::C;
                        let result = match op {
                            AluOp::ADD | AluOp::ADC => {
                                let (result, carry) = lhs.carrying_add(rhs, op == AluOp::ADC && flags.contains(Flags::C));
                                flags.set(Flags::C, carry);
                                flags.set(Flags::P, (lhs as i8).overflowing_add(rhs as i8).1);
                                flags.set(Flags::H, (lhs << 4).overflowing_add(rhs << 4).1);
                                result
                            },
                            AluOp::CP | AluOp::SUB | AluOp::SBC => {
                                flags.insert(Flags::N);
                                let (result, carry) = lhs.borrowing_sub(rhs, op == AluOp::SBC && flags.contains(Flags::C));
                                flags.set(Flags::C, carry);
                                flags.set(Flags::P, (lhs as i8).overflowing_sub(rhs as i8).1);
                                flags.set(Flags::H, (lhs << 4).overflowing_sub(rhs << 4).1);
                                result
                            },
                            AluOp::AND | AluOp::XOR | AluOp::OR => {
                                let result = match op {
                                    AluOp::AND => lhs & rhs,
                                    AluOp::XOR => lhs ^ rhs,
                                    AluOp::OR => lhs | rhs,
                                    _ => unreachable!()
                                };
                                flags.set_parity_flag(result);
                                result
                            },
                        };
                        flags |= Flags::from(result) & Flags::XY;
                        flags.set_zs_flags_u8(result);
                        self.set_flags(flags);
                        if op != AluOp::CP {
                            self.rg(Reg::A).set(result);
                        }
                    },

                    Token::INC_RG(reg) | Token::DEC_RG(reg) => {
                        let value = self.rg(reg).get();
                        let mut flags = self.get_flags() & Flags::C;
                        let result = if let Token::INC_RG(..) = instruction.opcode {
                            flags.set(Flags::P, (value as i8).overflowing_add(1 as i8).1);
                            flags.set(Flags::H, (value << 4).overflowing_add(1 << 4).1);
                            value.wrapping_add(1)
                        } else {
                            flags.insert(Flags::N);
                            flags.set(Flags::P, (value as i8).overflowing_sub(1 as i8).1);
                            flags.set(Flags::H, (value << 4).overflowing_sub(1 << 4).1);
                            value.wrapping_sub(1)
                        };
                        flags |= Flags::from(result) & Flags::XY;
                        flags.set_zs_flags_u8(result);
                        self.set_flags(flags);
                        self.rg(Reg::A).set(result);
                    },

                    // General-Purpose Arithmetic and CPU Control

                    Token::DAA => {
                        let value = self.rg(Reg::A).get();
                        let mut flags = self.get_flags();
                        let mut correction: u8 = 0;
                        if value & 0x0f > 0x09 || flags.contains(Flags::H) { correction |= 0x06; }
                        if value & 0xf0 > 0x90 || flags.contains(Flags::C) { correction |= 0x60; }
                        let (result, carry) = if flags.contains(Flags::N) {
                            value.overflowing_sub(correction)
                        } else {
                            value.overflowing_add(correction)
                        };
                        flags.set(Flags::C, carry);
                        flags.set(Flags::H, if flags.contains(Flags::N) {
                            (value << 4).overflowing_sub(correction << 4).1
                        } else {
                            (value << 4).overflowing_add(correction << 4).1
                        });
                        flags |= Flags::from(result) & Flags::XY;
                        flags.set_zs_flags_u8(result);
                        flags.set_parity_flag(result);
                        self.set_flags(flags);
                        self.rg(Reg::A).set(result);
                    },
                    Token::CPL => {
                        let result = !self.rg(Reg::A).get();
                        self.rg(Reg::A).set(result);
                        self.set_flags(
                            (self.get_flags() & !Flags::XY) |
                            (Flags::from(result) & Flags::XY) |
                            Flags::H | Flags::N
                        );
                    },
                    Token::NEG => {
                        let value = self.rg(Reg::A).get();
                        let (result, carry) = (0 as u8).overflowing_sub(value);
                        let mut flags = Flags::N | (Flags::from(result) & Flags::XY);
                        flags.set(Flags::C, carry);
                        flags.set(Flags::P, (0 as i8).overflowing_sub(value as i8).1);
                        flags.set(Flags::H, (0 as u8).overflowing_sub(value << 4).1);
                        flags.set_zs_flags_u8(result);
                        self.set_flags(flags);
                        self.rg(Reg::A).set(value);
                    },
                    Token::CCF => {
                        self.set_flags(self.get_flags() ^ Flags::C);
                    },
                    Token::SCF => {
                        self.set_flags(self.get_flags() | Flags::C);
                    },
                    Token::NOP => {},
                    Token::HALT => {
                        self.bus.halt.drive(self, true);
                        self.rp(RegPair::PC).update(|pc| pc.wrapping_sub(1)); // rewind PC 1 byte back
                    },
                    Token::DI => {
                        self.iff1.set(false);
                        self.iff2.set(false);
                    },
                    Token::EI => {
                        self.iff1.set(true);
                        self.iff2.set(true);
                    },
                    Token::IM(mode) => {
                        self.im.set(mode);
                    },

                    // 16-Bit Arithmetic

                    Token::ADD_RP_RP(dst, src) => {
                        yield self.clock.rising(7); // Last 2 M-cycles = 4+3 t-cycles
                        let lhs = self.rp(dst).get();
                        let rhs = self.rp(src).get();
                        let (result, carry) = lhs.overflowing_add(rhs);
                        let mut flags = self.get_flags() & !Flags::N;
                        flags.set(Flags::C, carry);
                        flags.set(Flags::H, (lhs << 4).overflowing_add(rhs << 4).1);
                        self.rp(dst).set(result);
                        self.set_flags(flags);
                    },
                    Token::ADC_HL_RP(rpair) => {
                        yield self.clock.rising(7); // Last 2 M-cycles = 4+3 t-cycles
                        let lhs = self.rp(RegPair::HL).get();
                        let rhs = self.rp(rpair).get();
                        let mut flags = Flags::NONE;
                        let (result, carry) = lhs.carrying_add(rhs, flags.contains(Flags::C));
                        flags.set_zs_flags_u16(result);
                        flags.set(Flags::C, carry);
                        flags.set(Flags::P, (lhs as i16).overflowing_add(rhs as i16).1);
                        flags.set(Flags::H, (lhs << 4).overflowing_add(rhs << 4).1);
                        self.rp(RegPair::HL).set(result);
                        self.set_flags(flags);
                    },
                    Token::SBC_HL_RP(rpair) => {
                        yield self.clock.rising(7); // Last 2 M-cycles = 4+3 t-cycles
                        let lhs = self.rp(RegPair::HL).get();
                        let rhs = self.rp(rpair).get();
                        let mut flags = Flags::NONE;
                        let (result, carry) = lhs.borrowing_sub(rhs, flags.contains(Flags::C));
                        flags.set_zs_flags_u16(result);
                        flags.set(Flags::C, carry);
                        flags.set(Flags::P, (lhs as i16).overflowing_sub(rhs as i16).1);
                        flags.set(Flags::H, (lhs << 4).overflowing_sub(rhs << 4).1);
                        self.rp(RegPair::HL).set(result);
                        self.set_flags(flags);
                    },
                    Token::INC_RP(rpair) => {
                        yield self.clock.rising(2); // complement M-cycle to 6 t-cycles
                        self.rp(rpair).update(|rp| rp.wrapping_add(1));
                    },
                    Token::DEC_RP(rpair) => {
                        yield self.clock.rising(2); // complement M-cycle to 6 t-cycles
                        self.rp(rpair).update(|rp| rp.wrapping_sub(1));
                    },

                    // Rotate and Shift

                    Token::SHOP(op, reg, maybe_dst) => {
                        let val = yield_from!(self.read_register(reg, instruction.displacement));

                        let mut flags = self.get_flags() & !(Flags::H | Flags::N);

                        let result = match op {
                            ShiftOp::RLC => {
                                let val = val.rotate_left(1);
                                flags.set(Flags::C, val & 0x1 != 0);
                                val
                            },
                            ShiftOp::RRC => {
                                let val = val.rotate_right(1);
                                flags.set(Flags::C, val & 0x80 != 0);
                                val
                            },
                            ShiftOp::RL => {
                                let mut val = val.rotate_left(1);
                                let carry = val & 0x1 != 0; val &= !0x1;
                                if flags.contains(Flags::C) { val |= 0x1; }
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::RR => {
                                let mut val = val.rotate_right(1);
                                let carry = val & 0x80 != 0; val &= !0x80;
                                if flags.contains(Flags::C) { val |= 0x80; }
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::SLA => {
                                let mut val = val.rotate_left(1);
                                let carry = val & 0x1 != 0; val &= !0x1;
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::SRA => {
                                let mut val = val.rotate_right(1);
                                let carry = val & 0x80 != 0; val &= !0x80;
                                val |= (val | 0x40) << 1;
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::SLL => {
                                let mut val = val.rotate_left(1);
                                let carry = val & 0x1 != 0; val |= 0x1;
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::SRL => {
                                let mut val = val.rotate_right(1);
                                let carry = val & 0x80 != 0; val &= !0x80;
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::RLCA => {
                                let val = val.rotate_left(1);
                                flags.set(Flags::C, val & 0x1 != 0);
                                val
                            },
                            ShiftOp::RRCA => {
                                let val = val.rotate_right(1);
                                flags.set(Flags::C, val & 0x80 != 0);
                                val
                            },
                            ShiftOp::RLA => {
                                let mut val = val.rotate_left(1);
                                let carry = val & 0x1 != 0; val &= !0x1;
                                if flags.contains(Flags::C) { val |= 0x1; }
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::RRA => {
                                let mut val = val.rotate_right(1);
                                let carry = val & 0x80 != 0; val &= !0x80;
                                if flags.contains(Flags::C) { val |= 0x80; }
                                flags.set(Flags::C, carry);
                                val
                            },
                            ShiftOp::RLD => {
                                yield self.clock.rising(3); // M4
                                let acc = self.rg(Reg::A).get();
                                self.rg(Reg::A).set((acc & 0xf0) | (val >> 4));
                                (val << 4) | (acc & 0xf)
                            },
                            ShiftOp::RRD => {
                                yield self.clock.rising(3); // M4
                                let acc = self.rg(Reg::A).get();
                                self.rg(Reg::A).set((acc & 0xf0) | val & 0xf);
                                (val >> 4) | (acc << 4)
                            },
                        };

                        match op {
                            ShiftOp::RLCA | ShiftOp::RRCA | ShiftOp::RLA | ShiftOp::RRA => (),
                            _ => { // Set Z,S,P flags for all ops except above
                                flags.set_zs_flags_u8(result);
                                flags.set_parity_flag(result);
                            }
                        }

                        yield_from!(self.write_register(reg, result, instruction.displacement));

                        // Covers undocumented CCCB/FDCB opcodes which additionally write result
                        // to some register. E.g. RLC (IX+n),B
                        if let Some(dst) = maybe_dst {
                            self.rg(dst).set(result);
                        }

                        self.set_flags(flags);
                    },

                    // Bit Set, Reset and Test

                    Token::BIT(bit, reg) => {
                        let val = yield_from!(self.read_register(reg, instruction.displacement));
                        let mut flags = self.get_flags() | Flags::H & !Flags::N;
                        let zero = (val >> bit) & 0x1 == 0;
                        flags.set(Flags::Z, zero);
                        flags.set(Flags::P, zero);
                        self.set_flags(flags);
                    },
                    Token::SET(bit, reg, maybe_dst) | Token::RES(bit, reg, maybe_dst) => {
                        let val = yield_from!(self.read_register(reg, instruction.displacement));

                        let result = if let Token::SET(..) = instruction.opcode {
                            val | (0x1 << bit)
                        } else {
                            val & !(0x1 << bit)
                        };

                        yield_from!(self.write_register(reg, result, instruction.displacement));

                        // Covers undocumented CCCB/FDCB opcodes which additionally write result
                        // to some register. E.g. RLC (IX+n),B
                        if let Some(dst) = maybe_dst {
                            self.rg(dst).set(result);
                        }
                    },

                    // Jump, Call and Return

                    Token::JP(cond) => {
                        if self.get_flags().satisfy(cond) {
                            let dst = instruction.expect_word_data();
                            self.rp(RegPair::PC).set(dst);
                        }
                    },
                    Token::JP_RP(rpair) => {
                        self.rp(RegPair::PC).set(self.rp(rpair).get());
                    },
                    Token::JR(cond) => {
                        if self.get_flags().satisfy(cond) {
                            yield self.clock.rising(5); // M3 = 5 T-cycles
                            let offset = instruction.displacement.unwrap();
                            self.rp(RegPair::PC).update(|pc| pc.wrapping_add_signed(offset as i16));
                        }
                    },
                    Token::DJNZ => {
                        if self.rg(Reg::B).update(|b| b.wrapping_sub(1)) != 0 {
                            yield self.clock.rising(5); // M3 = 5 T-cycles
                            let offset = instruction.displacement.unwrap();
                            self.rp(RegPair::PC).update(|pc| pc.wrapping_add_signed(offset as i16));
                        }
                    },
                    Token::CALL(cond) => {
                        if self.get_flags().satisfy(cond) {
                            yield self.clock.rising(1); // complement M3 to 4 t-cycles
                            yield_from!(self.stack_push(self.rp(RegPair::PC).get()));
                            self.rp(RegPair::PC).set(instruction.expect_word_data());
                        }
                    },
                    Token::RET(Condition::None) => {
                        self.rp(RegPair::PC).set(yield_from!(self.stack_pop()));
                    },
                    Token::RET(cond) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        if self.get_flags().satisfy(cond) {
                            self.rp(RegPair::PC).set(yield_from!(self.stack_pop()));
                        }
                    },
                    Token::RETN | Token::RETI => {
                        // NOTE: According to the official Z80 documentation,
                        // RETI should behave like RET (not copying IFF2 to IFF1).
                        // But in practice it copies it the same way as RETN.
                        self.iff1.set(self.iff2.get());
                        self.rp(RegPair::PC).set(yield_from!(self.stack_pop()));
                    },
                    Token::RST(addr) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        yield_from!(self.stack_push(self.rp(RegPair::PC).get()));
                        self.rp(RegPair::PC).set(addr as u16);
                    },

                    // IO group

                    Token::IN_A_N => {
                        let addr = mkword!(self.rg(Reg::A).get(), instruction.expect_byte_data());
                        self.rg(Reg::A).set(yield_from!(self.io_read(addr)));
                    },
                    Token::OUT_N_A => {
                        let addr = mkword!(self.rg(Reg::A).get(), instruction.expect_byte_data());
                        yield_from!(self.io_write(addr, self.rg(Reg::A).get()));
                    },
                    Token::IN_RG_AtBC(reg) => {
                        let addr = self.rp(RegPair::BC).get();
                        self.rg(reg).set(yield_from!(self.io_read(addr)));
                    },
                    Token::OUT_AtBC_RG(reg) => {
                        let addr = self.rp(RegPair::BC).get();
                        yield_from!(self.io_write(addr, self.rg(reg).get()));
                    },
                    Token::IN_AtBC => {
                        let addr = self.rp(RegPair::BC).get();
                        yield_from!(self.io_read(addr));
                    },
                    Token::OUT_AtBC_0 => {
                        let addr = self.rp(RegPair::BC).get();
                        yield_from!(self.io_write(addr, 0));
                    },
                    Token::BLOP(BlockOp::INI)  => yield_from!(self.io_read_block ( 1, false)),
                    Token::BLOP(BlockOp::OUTI) => yield_from!(self.io_write_block( 1, false)),
                    Token::BLOP(BlockOp::IND)  => yield_from!(self.io_read_block (-1, false)),
                    Token::BLOP(BlockOp::OUTD) => yield_from!(self.io_write_block(-1, false)),
                    Token::BLOP(BlockOp::INIR) => yield_from!(self.io_read_block ( 1, true)),
                    Token::BLOP(BlockOp::OTIR) => yield_from!(self.io_write_block( 1, true)),
                    Token::BLOP(BlockOp::INDR) => yield_from!(self.io_read_block (-1, true)),
                    Token::BLOP(BlockOp::OTDR) => yield_from!(self.io_write_block(-1, true)),

                    // Non-opcode is not expected

                    Token::Prefix(..) | Token::Displacement(..) | Token::Data(..) => unreachable!()

                }

            }

        })

    }

}

impl Cpu {

    // Create new CPU instance
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self { bus, clock, ..Default::default() }
    }

    /// Get reference to register value
    pub fn rg(&self, reg: Reg) -> &Cell<u8> {
        match reg {
            Reg::B   => &self.bc.bytes().hi,
            Reg::C   => &self.bc.bytes().lo,
            Reg::D   => &self.de.bytes().hi,
            Reg::E   => &self.de.bytes().lo,
            Reg::H   => &self.hl.bytes().hi,
            Reg::L   => &self.hl.bytes().lo,
            Reg::A   => &self.af.bytes().hi,
            Reg::F   => &self.af.bytes().lo,
            Reg::I   => &self.ir.bytes().hi,
            Reg::R   => &self.ir.bytes().lo,
            Reg::IXH => &self.ix.bytes().hi,
            Reg::IXL => &self.ix.bytes().lo,
            Reg::IYH => &self.iy.bytes().hi,
            Reg::IYL => &self.iy.bytes().lo,
            _ => unreachable!()
        }
    }

    /// Get reference to regpair value
    pub fn rp(&self, rpair: RegPair) -> &Cell<u16> {
        match rpair {
            RegPair::BC => &self.bc.value(),
            RegPair::DE => &self.de.value(),
            RegPair::HL => &self.hl.value(),
            RegPair::AF => &self.af.value(),
            RegPair::SP => &self.sp.value(),
            RegPair::PC => &self.pc.value(),
            RegPair::IR => &self.ir.value(),
            RegPair::IX => &self.ix.value(),
            RegPair::IY => &self.iy.value(),
            _ => unreachable!()
        }
    }

    /// Get CPU flags
    pub fn get_flags(&self) -> Flags {
        Flags::from(self.rg(Reg::F).get())
    }

    /// Set CPU flags
    pub fn set_flags(&self, flags: Flags) {
        self.rg(Reg::F).set(flags.bits())
    }

    /// Swap primary and alternative accumulator (AF)
    fn swap_acc(&self) {
        self.af.value().swap(&self.alt_af.value());
    }

    /// Swap primary and alternative BC,DE and HL
    fn swap_regfile(&self) {
        self.bc.value().swap(&self.alt_bc.value());
        self.de.value().swap(&self.alt_de.value());
        self.hl.value().swap(&self.alt_hl.value());
    }

    /// Swap HL and DE
    fn swap_hlde(&self) {
        self.hl.value().swap(&self.de.value());
    }

    /// Calculate absolute address for IX+d or IY+d offsets
    fn idx_addr(&self, reg: Reg, displacement: i8) -> u16 {
        let rpair = match reg {
            Reg::AtIX => RegPair::IX,
            Reg::AtIY => RegPair::IY,
            _ => unreachable!()
        };
        let addr = self.rp(rpair).get() as i32 + displacement as i32;
        return addr as u16;
    }

    /// Probe INT & NMI bus lines and sets corresponding CPU flags
    fn probe_interrupts(&self) {
        if self.bus.nmi.probe().unwrap_or(false) {
            self.nmi.set(true);
        }
        if self.bus.int.probe().unwrap_or(false) {
            self.int.set(true);
        }
    }

    /// Probe WAIT signal and wait while it's set
    fn process_wait<'a>(&'a self) -> impl Task<()> + 'a {
        #[coroutine] move || {
            while self.bus.wait.probe().unwrap_or(false) {
                yield self.clock.falling(1); // wait 1 t-cycle
            }
        }
    }

    /// Probe BUSRQ signal and set address, data and ctrl outputs
    /// to high impedance state while it's set
    fn process_busrq<'a>(&'a self) -> impl Task<()> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1);
            self.bus.data.release(self);
            self.bus.addr.release(self);
            self.bus.ctrl.release(self);
            self.bus.busak.drive(self, true);
            while self.bus.busrq.probe().unwrap_or(false) {
                yield self.clock.rising(1); // wait 1 t-cycle
            }
            yield self.clock.falling(1);
            self.bus.busak.drive(self, false);
        }
    }

    /// Interrupt response m-cycle. Takes 6 t-cycles.
    /// Similar to M1, but with 2 additional wait cycles
    /// and IORQ instead of MREQ & RD.
    fn interrupt_response<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.data.release(self);
            self.bus.addr.drive(self, addr);
            self.bus.ctrl.drive(self, Ctrl::NONE);
            self.bus.m1.drive(self, true);
            yield self.clock.falling(3); // TW1 falling
            self.bus.ctrl.drive(self, Ctrl::IORQ);
            yield self.clock.falling(1); // TW2 falling
            yield_from!(self.process_wait());
            yield self.clock.rising(1); // T3 rising
            let byte = self.bus.data.expect();
            // Increment R (lower 7 bits)
            let r = self.rg(Reg::R).get();
            self.rg(Reg::R).set(((r + 1) & 0x7f) | (r & 0x80));
            self.bus.addr.drive(self, self.rp(RegPair::IR).get());
            self.bus.ctrl.drive(self, Ctrl::RFSH); // clears IORQ
            self.bus.m1.drive(self, false);
            yield self.clock.falling(1); // T3 falling
            self.bus.ctrl.drive(self, Ctrl::RFSH | Ctrl::MREQ);
            yield self.clock.rising(1); // T4 rising
            let busrq = self.bus.busrq.probe().unwrap_or(false);
            self.probe_interrupts();
            yield self.clock.falling(1); // T4 falling
            self.bus.ctrl.drive(self, Ctrl::RFSH); // clears MREQ
            if busrq { yield_from!(self.process_busrq()); }
            return byte;
        }
    }

    /// Instruction opcode fetch m-cycle
    /// (usually referred to as M1). Takes 4 t-cycles.
    fn opcode_read<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.data.release(self);
            self.bus.addr.drive(self, addr);
            self.bus.ctrl.drive(self, Ctrl::NONE);
            self.bus.m1.drive(self, true);
            yield self.clock.falling(1); // T1 falling
            self.bus.ctrl.drive(self, Ctrl::MREQ | Ctrl::RD);
            yield self.clock.falling(1); // T2 falling
            yield_from!(self.process_wait());
            yield self.clock.rising(1); // T3 rising
            let byte = self.bus.data.expect();
            // Increment R (lower 7 bits)
            let r = self.rg(Reg::R).get();
            self.rg(Reg::R).set(((r + 1) & 0x7f) | (r & 0x80));
            self.bus.addr.drive(self, self.rp(RegPair::IR).get());
            self.bus.ctrl.drive(self, Ctrl::RFSH); // clears MREQ & RD
            self.bus.m1.drive(self, false);
            yield self.clock.falling(1); // T3 falling
            self.bus.ctrl.drive(self, Ctrl::RFSH | Ctrl::MREQ);
            yield self.clock.rising(1); // T4 rising
            let busrq = self.bus.busrq.probe().unwrap_or(false);
            self.probe_interrupts();
            yield self.clock.falling(1); // T4 falling
            self.bus.ctrl.drive(self, Ctrl::RFSH); // clears MREQ
            if busrq { yield_from!(self.process_busrq()); }
            return byte;
        }
    }

    /// Memory read m-cycle. Takes 3 t-cycles.
    fn memory_read<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.data.release(self);
            self.bus.addr.drive(self, addr);
            self.bus.ctrl.drive(self, Ctrl::NONE);
            yield self.clock.falling(1); // T1 falling
            self.bus.ctrl.drive(self, Ctrl::MREQ | Ctrl::RD);
            yield self.clock.falling(1); // T2 falling
            yield_from!(self.process_wait());
            yield self.clock.rising(1); // T3 rising
            let busrq = self.bus.busrq.probe().unwrap_or(false);
            self.probe_interrupts();
            yield self.clock.falling(1); // T3 falling
            let byte = self.bus.data.expect();
            self.bus.ctrl.drive(self, Ctrl::NONE);
            if busrq { yield_from!(self.process_busrq()); }
            return byte;
        }
    }

    /// Memory write m-cycle. Takes 3 t-cycles.
    fn memory_write<'a>(&'a self, addr: u16, val: u8) -> impl Task<()> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.data.release(self);
            self.bus.addr.drive(self, addr);
            self.bus.ctrl.drive(self, Ctrl::NONE);
            yield self.clock.falling(1); // T1 falling
            self.bus.data.drive(self, val);
            self.bus.ctrl.drive(self, Ctrl::MREQ);
            yield self.clock.falling(1); // T2 falling
            self.bus.ctrl.drive(self, Ctrl::MREQ | Ctrl::WR);
            yield_from!(self.process_wait());
            yield self.clock.rising(1); // T3 rising
            let busrq = self.bus.busrq.probe().unwrap_or(false);
            self.probe_interrupts();
            yield self.clock.falling(1); // T3 falling
            self.bus.ctrl.drive(self, Ctrl::NONE);
            if busrq { yield_from!(self.process_busrq()); }
        }
    }

    /// Copy memory byte (DE)<-(HL), inc or dec HL&DE, dec BC
    fn memory_copy_block<'a>(&'a self, increment: i8, repeat: bool) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let src = self.rp(RegPair::HL).get();
            let dst = self.rp(RegPair::DE).get();
            let ctr = self.rp(RegPair::BC).get().wrapping_sub(1);
            let val = yield_from!(self.memory_read(src));
            yield_from!(self.memory_write(dst, val));
            yield self.clock.rising(2); // complement MW to 5 t-cycles
            self.rp(RegPair::HL).set(src.wrapping_add(increment as u16));
            self.rp(RegPair::DE).set(dst.wrapping_add(increment as u16));
            self.rp(RegPair::BC).set(ctr);
            let n = self.rg(Reg::A).get().wrapping_add(val);
            let mut flags = self.get_flags() & Flags::C;
            flags.set(Flags::P, ctr != 0);
            flags.set(Flags::Y, n & (1 << 1) != 0);
            flags.set(Flags::X, n & (1 << 3) != 0);
            self.set_flags(flags);
            if repeat && flags.contains(Flags::P) {
                yield self.clock.rising(5);
                self.rp(RegPair::PC).update(|pc| pc.wrapping_sub(2)); // rewind PC 2 bytes back
            }
        }
    }

    /// Compare A against (HL), inc or dec HL, dec BC
    fn memory_cmp_block<'a>(&'a self, increment: i8, repeat: bool) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let src = self.rp(RegPair::HL).get();
            let ctr = self.rp(RegPair::BC).get().wrapping_sub(1);
            let lhs = self.rg(Reg::A).get();
            let rhs = yield_from!(self.memory_read(src));
            yield self.clock.rising(5);
            self.rp(RegPair::HL).set(src.wrapping_add(increment as u16));
            self.rp(RegPair::BC).set(ctr);
            let mut n = lhs.wrapping_sub(rhs);
            let mut flags = (self.get_flags() & Flags::C) | Flags::N;
            flags.set_zs_flags_u8(n);
            flags.set(Flags::H, (lhs << 4).overflowing_sub(rhs << 4).1);
            if flags.contains(Flags::H) { n = n.wrapping_sub(1); }
            flags.set(Flags::P, ctr != 0);
            flags.set(Flags::Y, n & (1 << 1) != 0);
            flags.set(Flags::X, n & (1 << 3) != 0);
            self.set_flags(flags);
            if repeat && flags.contains(Flags::P) {
                yield self.clock.rising(5);
                self.rp(RegPair::PC).update(|pc| pc.wrapping_sub(2)); // rewind PC 2 bytes back
            }
        }
    }

    /// IO read m-cycle. Takes 3 t-cycles.
    fn io_read<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.data.release(self);
            self.bus.addr.drive(self, addr);
            self.bus.ctrl.drive(self, Ctrl::NONE);
            yield self.clock.rising(1); // T2 rising
            self.bus.ctrl.drive(self, Ctrl::IORQ | Ctrl::RD);
            yield self.clock.falling(2); // TW falling
            yield_from!(self.process_wait());
            yield self.clock.rising(1); // T3 rising
            let busrq = self.bus.busrq.probe().unwrap_or(false);
            self.probe_interrupts();
            yield self.clock.falling(1); // T3 falling
            let byte = self.bus.data.expect();
            self.bus.ctrl.drive(self, Ctrl::NONE);
            if busrq { yield_from!(self.process_busrq()); }
            return byte;
        }
    }

    /// IO write m-cycle. Takes 3 t-cycles.
    fn io_write<'a>(&'a self, addr: u16, val: u8) -> impl Task<()> + 'a {
        #[coroutine] move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.data.release(self);
            self.bus.addr.drive(self, addr);
            self.bus.ctrl.drive(self, Ctrl::NONE);
            yield self.clock.falling(1); // T1 falling
            self.bus.data.drive(self, val);
            yield self.clock.rising(1); // T2 rising
            self.bus.ctrl.drive(self, Ctrl::IORQ | Ctrl::WR);
            yield self.clock.falling(2); // TW falling
            yield_from!(self.process_wait());
            yield self.clock.rising(1); // T3 rising
            let busrq = self.bus.busrq.probe().unwrap_or(false);
            self.probe_interrupts();
            yield self.clock.falling(1); // T3 falling
            self.bus.ctrl.drive(self, Ctrl::NONE);
            if busrq { yield_from!(self.process_busrq()); }
        }
    }

    /// Reads port at (BC) and writes it to memory at (HL), inc or dec HL, dec B
    fn io_read_block<'a>(&'a self, increment: i8, repeat: bool) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let src = self.rp(RegPair::BC).get();
            let dst = self.rp(RegPair::HL).get();
            let ctr = self.rg(Reg::B).get().wrapping_sub(1);
            yield self.clock.rising(1); // complement M2 to 5 t-cycles
            let val = yield_from!(self.io_read(src));
            yield_from!(self.memory_write(dst, val));
            yield self.clock.rising(1); // complement M4 to 4 t-cycles
            self.rp(RegPair::HL).set(dst.wrapping_add(increment as u16));
            self.rg(Reg::B).set(ctr);
            let mut flags = (self.get_flags() & Flags::C) | Flags::N;
            flags.set_zs_flags_u8(ctr);
            self.set_flags(flags);
            if repeat && !flags.contains(Flags::Z) {
                yield self.clock.rising(5);
                self.rp(RegPair::PC).update(|pc| pc.wrapping_sub(2)); // rewind PC 2 bytes back
            }
        }
    }

    /// Reads memory byte at (HL) and outputs it to port (BC), inc or dec HL, dec B
    fn io_write_block<'a>(&'a self, increment: i8, repeat: bool) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let src = self.rp(RegPair::HL).get();
            let dst = self.rp(RegPair::BC).get();
            let ctr = self.rg(Reg::B).get().wrapping_sub(1);
            yield self.clock.rising(1); // complement M2 to 5 t-cycles
            let val = yield_from!(self.memory_read(src));
            yield_from!(self.io_write(dst, val));
            yield self.clock.rising(1); // complement M4 to 4 t-cycles
            self.rp(RegPair::HL).set(dst.wrapping_add(increment as u16));
            self.rg(Reg::B).set(ctr);
            let mut flags = (self.get_flags() & Flags::C) | Flags::N;
            flags.set_zs_flags_u8(ctr);
            self.set_flags(flags);
            if repeat && !flags.contains(Flags::Z) {
                yield self.clock.rising(5);
                self.rp(RegPair::PC).update(|pc| pc.wrapping_sub(2)); // rewind PC 2 bytes back
            }
        }
    }

    fn read_register<'a>(&'a self, reg: Reg, displacement: Option<i8>) -> impl Task<u8> + 'a {
        #[coroutine] move || {
            match reg {
                Reg::AtHL => {
                    let val  = yield_from!(self.memory_read(self.rp(RegPair::HL).get()));
                    yield self.clock.rising(1); // complement M3 to 4 t-cycles
                    val
                },
                Reg::AtIX | Reg::AtIY => {
                    yield self.clock.rising(1); // complement M4 to 5 t-cycles
                    let val = yield_from!(self.memory_read(
                        self.idx_addr(reg, displacement.unwrap())
                    ));
                    yield self.clock.rising(1); // complement M5 to 4 t-cycles
                    val
                },
                reg => self.rg(reg).get(),
            }
        }
    }

    fn write_register<'a>(&'a self, reg: Reg, value: u8, displacement: Option<i8>) -> impl Task<()> + 'a {
        #[coroutine] move || {
            match reg {
                Reg::AtHL => {
                    yield_from!(self.memory_write(self.rp(RegPair::HL).get(), value));
                },
                Reg::AtIX | Reg::AtIY => {
                    yield_from!(self.memory_write(
                        self.idx_addr(reg, displacement.unwrap()),
                        value
                    ));
                },
                reg => self.rg(reg).set(value),
            }
        }
    }

    fn stack_pop<'a>(&'a self) -> impl Task<u16> + 'a {
        #[coroutine] move || {
            let addr = self.rp(RegPair::SP).get();
            let lo = yield_from!(self.memory_read(addr));
            let hi = yield_from!(self.memory_read(addr + 1));
            self.rp(RegPair::SP).set(addr + 2);
            return mkword!(hi, lo);
        }
    }

    fn stack_push<'a>(&'a self, value: u16) -> impl Task<()> + 'a {
        #[coroutine] move || {
            let addr = self.rp(RegPair::SP).get();
            let (hi, lo) = spword!(value);
            yield_from!(self.memory_write(addr - 1, hi));
            yield_from!(self.memory_write(addr - 2, lo));
            self.rp(RegPair::SP).set(addr - 2);
        }
    }
}
