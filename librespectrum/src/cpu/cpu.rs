use std::{
    rc::Rc,
    cell::Cell,
};

use crate::{
    mkword,
    spword,
    yield_from,
    bus::{NoReturnTask, Clock, CpuBus, Task, Ctls, Outs},
    cpu::{
        tokens::{Token, TokenType, Reg, RegPair, BlockOp, AluOp},
        InstructionDecoder, Flags
    },
    misc::Word,
};

/// Register file
#[derive(Default)]
pub struct CpuState {
    pub af: Word,
    pub bc: Word,
    pub de: Word,
    pub hl: Word,
    pub alt_af: Word,
    pub alt_bc: Word,
    pub alt_de: Word,
    pub alt_hl: Word,
    pub ix: Word,
    pub iy: Word,
    pub sp: Word,
    pub pc: Word,
    pub ir: Word,
    pub iff1: bool,
    pub iff2: bool,
    pub im: u8,
}

/// Z80 CPU
pub struct Cpu {
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
    state: Rc<CpuState>,
}

#[inline]
fn parity(value: u8) -> bool {
    // TODO: Optimize it by using a CPU flag or
    // http://www.graphics.stanford.edu/~seander/bithacks.html#ParityParallel
    value.count_ones() % 2 == 0
}

impl Cpu {

    // Create new CPU instance
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>, state: Rc<CpuState>) -> Self {
        Self { bus, clock, state }
    }

    /// Run CPU device task
    pub fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(move || {

            // Instruction loop
            loop {

                let mut decoder = InstructionDecoder::new();

                // Instruction decode loop
                let opcode = loop {

                    let pc = self.next_pc(1);

                    // Read the next byte using appropriate M-cycle
                    let byte: u8 = match decoder.upnext() {
                        TokenType::Opcode => yield_from!(self.opcode_read(pc)),
                        TokenType::Displacement | TokenType::Data => yield_from!(self.memory_read(pc))
                    };

                    // Decode byte
                    let decoding = decoder.decode(byte);
                    if !decoding {
                        break decoder.expect_opcode(); // instruction decoded
                    }

                };

                match opcode {

                    // 8-bit Load

                    Token::LD_RG_RG(dst @ (Reg::AtIX | Reg::AtIY), src) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.idx_addr(dst, decoder.expect_displacement());
                        yield_from!(self.memory_write(addr, self.rg(src).get()));
                    },
                    Token::LD_RG_RG(dst, src @ (Reg::AtIX | Reg::AtIY)) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.idx_addr(src, decoder.expect_displacement());
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
                        flags.set(Flags::P, self.state.iff2);
                        flags.set(Flags::Z, value == 0);
                        flags.set(Flags::S, (value as i8) < 0);
                        self.set_flags(flags);
                    },
                    Token::LD_RG_RG(dst, src) => {
                        self.rg(dst).set(self.rg(src).get());
                    },
                    Token::LD_RG_N(Reg::AtHL) => {
                        let addr = self.rp(RegPair::HL).get();
                        yield_from!(self.memory_write(addr, decoder.expect_byte_data()));
                    },
                    Token::LD_RG_N(reg) => {
                        self.rg(reg).set(decoder.expect_byte_data());
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
                        let addr = decoder.expect_word_data();
                        self.rg(Reg::A).set(yield_from!(self.memory_read(addr)));
                    },
                    Token::LD_MM_A => {
                        let addr = decoder.expect_word_data();
                        yield_from!(self.memory_write(addr, self.rg(Reg::A).get()));
                    },

                    // 16-bit Load

                    Token::LD_RP_NN(rpair) => {
                        self.rp(rpair).set(decoder.expect_word_data());
                    },
                    Token::LD_RP_MM(rpair) => {
                        let addr = decoder.expect_word_data();
                        let lo = yield_from!(self.memory_read(addr));
                        let hi = yield_from!(self.memory_read(addr + 1));
                        self.rp(rpair).set(mkword!(hi, lo));
                    },
                    Token::LD_MM_RP(rpair) => {
                        let addr = decoder.expect_word_data();
                        let (hi, lo) = spword!(self.rp(rpair).get());
                        yield_from!(self.memory_write(addr, lo));
                        yield_from!(self.memory_write(addr + 1, hi));
                    },
                    Token::LD_SP_RP(rpair) => {
                        yield self.clock.rising(2); // complement M1 to 6 t-cycles
                        self.rp(RegPair::SP).set(self.rp(rpair).get());
                    },
                    Token::POP(rpair) => {
                        let addr = self.rp(RegPair::SP).get();
                        let lo = yield_from!(self.memory_read(addr));
                        let hi = yield_from!(self.memory_read(addr + 1));
                        self.rp(rpair).set(mkword!(hi, lo));
                        self.rp(RegPair::SP).set(addr + 2);
                    },
                    Token::PUSH(rpair) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        let addr = self.rp(RegPair::SP).get();
                        let (hi, lo) = spword!(self.rp(rpair).get());
                        yield_from!(self.memory_write(addr - 1, hi));
                        yield_from!(self.memory_write(addr - 2, lo));
                        self.rp(RegPair::SP).set(addr - 2);
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

                    Token::BLOP(BlockOp::LDI) => yield_from!(self.memory_copy(1, false)),
                    Token::BLOP(BlockOp::LDD) => yield_from!(self.memory_copy(-1, false)),
                    Token::BLOP(BlockOp::LDIR) => yield_from!(self.memory_copy(1, true)),
                    Token::BLOP(BlockOp::LDDR) => yield_from!(self.memory_copy(-1, true)),
                    Token::BLOP(BlockOp::CPI) => unimplemented!(),
                    Token::BLOP(BlockOp::CPD) => unimplemented!(),
                    Token::BLOP(BlockOp::CPIR) => unimplemented!(),
                    Token::BLOP(BlockOp::CPDR) => unimplemented!(),

                    // 8-bit arithmetic and logic

                    Token::ALU(op, maybe_reg) => {
                        let lhs = self.rg(Reg::A).get();
                        let rhs = if let Some(reg) = maybe_reg {
                            self.rg(reg).get()
                        } else {
                            decoder.expect_byte_data()
                        };
                        let mut flags = self.get_flags() & Flags::C;
                        let result = match op {
                            AluOp::ADD | AluOp::ADC => {
                                let (result, carry) = if op == AluOp::ADC && flags.contains(Flags::C) {
                                    lhs.overflowing_add(rhs.wrapping_add(1))
                                } else {
                                    lhs.overflowing_add(rhs)
                                };
                                flags.set(Flags::C, carry);
                                flags.set(Flags::P, (lhs as i8).overflowing_add(rhs as i8).1);
                                flags.set(Flags::H, (lhs << 4).overflowing_add(rhs << 4).1);
                                result
                            },
                            AluOp::CP | AluOp::SUB | AluOp::SBC => {
                                flags.insert(Flags::N);
                                let (result, carry) = if op == AluOp::SBC && flags.contains(Flags::C) {
                                    lhs.overflowing_sub(rhs.wrapping_sub(1))
                                } else {
                                    lhs.overflowing_sub(rhs)
                                };
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
                                flags.set(Flags::P, parity(result));
                                result
                            },
                        };
                        flags.set(Flags::Z, result == 0);
                        flags.set(Flags::S, (result as i8) < 0);
                        flags |= Flags::from(result) & Flags::XY;
                        if op != AluOp::CP {
                            self.rg(Reg::A).set(result);
                        }
                        self.set_flags(flags);
                    },
                    Token::INC_RG(reg) | Token::DEC_RG(reg) => {
                        let value = self.rg(reg).get();
                        let mut flags = self.get_flags() & Flags::C;
                        let result = if let Token::INC_RG(_) = opcode {
                            flags.set(Flags::P, (value as i8).overflowing_add(1 as i8).1);
                            flags.set(Flags::H, (value << 4).overflowing_add(1 << 4).1);
                            value.wrapping_add(1)
                        } else {
                            flags.insert(Flags::N);
                            flags.set(Flags::P, (value as i8).overflowing_sub(1 as i8).1);
                            flags.set(Flags::H, (value << 4).overflowing_sub(1 << 4).1);
                            value.wrapping_sub(1)
                        };
                        flags.set(Flags::Z, result == 0);
                        flags.set(Flags::S, (result as i8) < 0);
                        flags |= Flags::from(result) & Flags::XY;
                        self.rg(Reg::A).set(result);
                        self.set_flags(flags);
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
                        flags.set(Flags::P, parity(result));
                        flags.set(Flags::H, if flags.contains(Flags::N) {
                            (value << 4).overflowing_sub(correction << 4).1
                        } else {
                            (value << 4).overflowing_add(correction << 4).1
                        });
                        flags.set(Flags::Z, result == 0);
                        flags.set(Flags::S, (result as i8) < 0);
                        flags |= Flags::from(result) & Flags::XY;
                        self.rg(Reg::A).set(result);
                        self.set_flags(flags);
                    },
                    Token::CPL => {
                        let value = self.rg(Reg::A).get();
                        let mut flags = (self.get_flags() & !Flags::XY) | Flags::H | Flags::N;
                        let result = !value;
                        flags |= Flags::from(result) & Flags::XY;
                        self.rg(Reg::A).set(result);
                        self.set_flags(flags);
                    },
                    Token::NEG => {
                        let value = self.rg(Reg::A).get();

                        let (result, carry) = (0 as u8).overflowing_sub(value);
                        let mut flags = (Flags::from(result) & Flags::XY) | Flags::N;
                        flags.set(Flags::C, carry);
                        flags.set(Flags::P, (0 as i8).overflowing_sub(value as i8).1);
                        flags.set(Flags::H, (0 as u8).overflowing_sub(value << 4).1);
                        flags.set(Flags::Z, result == 0);
                        flags.set(Flags::S, (result as i8) < 0);

                        self.rg(Reg::A).set(value);
                        self.set_flags(flags);
                    },
                    Token::CCF => {
                        self.set_flags(self.get_flags() ^ Flags::C);
                    },
                    Token::SCF => {
                        self.set_flags(self.get_flags() | Flags::C);
                    },
                    Token::NOP => {},
                    Token::HALT => {
                        unimplemented!();
                    },
                    Token::DI => {
                        unimplemented!();
                    },
                    Token::EI => {
                        unimplemented!();
                    },
                    Token::IM(mode) => {
                        unimplemented!();
                    },

                    // 16-Bit Arithmetic

                    Token::ADD_RP_RP(dst, src) => {
                        unimplemented!();
                    },
                    Token::ADC_HL_RP(rpair) => {
                        unimplemented!();
                    },
                    Token::SBC_HL_RP(rpair) => {
                        unimplemented!();
                    },
                    Token::INC_RP(rpair) => {
                        unimplemented!();
                    },
                    Token::DEC_RP(rpair) => {
                        unimplemented!();
                    },

                    // Rotate and Shift

                    Token::RLCA => {
                        unimplemented!();
                    },
                    Token::RLA => {
                        unimplemented!();
                    },
                    Token::RRCA => {
                        unimplemented!();
                    },
                    Token::RRA => {
                        unimplemented!();
                    },
                    Token::SHOP(op, reg) => {
                        unimplemented!();
                    },
                    Token::RLD => {
                        unimplemented!();
                    },
                    Token::RRD => {
                        unimplemented!();
                    },
                    Token::SHOPLD(op, reg, dst) => {
                        unimplemented!();
                    },

                    // Bit Set, Reset and Test

                    Token::BIT(bit, reg) => {
                        unimplemented!();
                    },
                    Token::SET(bit, reg) => {
                        unimplemented!();
                    },
                    Token::SETLD(bit, reg, dst) => {
                        unimplemented!();
                    },
                    Token::RES(bit, reg) => {
                        unimplemented!();
                    },
                    Token::RESLD(bit, reg, dst) => {
                        unimplemented!();
                    },

                    // Jump, Call and Return

                    Token::JP(cond) => {
                        unimplemented!();
                    },
                    Token::JP_RP(rpair) => {
                        unimplemented!();
                    },
                    Token::JR(cond) => {
                        unimplemented!();
                    },
                    Token::DJNZ => {
                        unimplemented!();
                    },
                    Token::CALL(cond) => {
                        unimplemented!();
                    },
                    Token::RET(cond) => {
                        unimplemented!();
                    },
                    Token::RETI => {
                        unimplemented!();
                    },
                    Token::RETN => {
                        unimplemented!();
                    },
                    Token::RST(n) => {
                        unimplemented!();
                    },

                    // IO group

                    Token::IN_A_N => {
                        let addr = mkword!(self.rg(Reg::A).get(), decoder.expect_byte_data());
                        self.rg(Reg::A).set(yield_from!(self.io_read(addr)));
                    },
                    Token::OUT_N_A => {
                        let addr = mkword!(self.rg(Reg::A).get(), decoder.expect_byte_data());
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
                    Token::BLOP(BlockOp::INI) => unimplemented!(),
                    Token::BLOP(BlockOp::OUTI) => unimplemented!(),
                    Token::BLOP(BlockOp::IND) => unimplemented!(),
                    Token::BLOP(BlockOp::OUTD) => unimplemented!(),
                    Token::BLOP(BlockOp::INIR) => unimplemented!(),
                    Token::BLOP(BlockOp::OTIR) => unimplemented!(),
                    Token::BLOP(BlockOp::INDR) => unimplemented!(),
                    Token::BLOP(BlockOp::OTDR) => unimplemented!(),

                    // Non-opcode is not expected

                    Token::Prefix(_) | Token::Displacement(_) | Token::Data(_) => unreachable!()

                }

            }

        })

    }

    /// Get reference to register value
    pub fn rg(&self, reg: Reg) -> &Cell<u8> {
        match reg {
            Reg::B   => &self.state.bc.bytes().hi,
            Reg::C   => &self.state.bc.bytes().lo,
            Reg::D   => &self.state.de.bytes().hi,
            Reg::E   => &self.state.de.bytes().lo,
            Reg::H   => &self.state.hl.bytes().hi,
            Reg::L   => &self.state.hl.bytes().lo,
            Reg::A   => &self.state.af.bytes().hi,
            Reg::F   => &self.state.af.bytes().lo,
            Reg::I   => &self.state.ir.bytes().hi,
            Reg::R   => &self.state.ir.bytes().lo,
            Reg::IXH => &self.state.ix.bytes().hi,
            Reg::IXL => &self.state.ix.bytes().lo,
            Reg::IYH => &self.state.iy.bytes().hi,
            Reg::IYL => &self.state.iy.bytes().lo,
            _ => unreachable!()
        }
    }

    /// Get reference to regpair value
    pub fn rp(&self, rpair: RegPair) -> &Cell<u16> {
        match rpair {
            RegPair::BC => &self.state.bc.word(),
            RegPair::DE => &self.state.de.word(),
            RegPair::HL => &self.state.hl.word(),
            RegPair::AF => &self.state.af.word(),
            RegPair::SP => &self.state.sp.word(),
            RegPair::PC => &self.state.pc.word(),
            RegPair::IR => &self.state.ir.word(),
            RegPair::IX => &self.state.ix.word(),
            RegPair::IY => &self.state.iy.word(),
            _ => unreachable!()
        }
    }

    /// Get CPU flags
    pub fn get_flags(&self) -> Flags {
        Flags::from(self.rg(Reg::F).get())
    }

    /// Set CPU flags
    pub fn set_flags(&self, flags: Flags ) {
        self.rg(Reg::F).set(flags.bits())
    }

    /// Swap primary and alternative accumulator (AF)
    fn swap_acc(&self) {
        self.state.af.word().swap(&self.state.alt_af.word());
    }

    /// Swap primary and alternative BC,DE and HL
    fn swap_regfile(&self) {
        self.state.bc.word().swap(&self.state.alt_bc.word());
        self.state.de.word().swap(&self.state.alt_de.word());
        self.state.hl.word().swap(&self.state.alt_hl.word());
    }

    /// Swap HL and DE
    fn swap_hlde(&self) {
        self.state.hl.word().swap(&self.state.de.word());
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

    /// Return current PC value and inc or dec it for the next read
    fn next_pc(&self, increment: i8) -> u16 {
        let pc = self.rp(RegPair::PC).get(); // read current PC
        self.rp(RegPair::PC).set(pc.wrapping_add(increment as u16));
        return pc;
    }

    /// Instruction opcode fetch m-cycle
    /// (usually referred to as M1). Takes 4 t-cycles.
    fn opcode_read<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        move || {
            yield self.clock.rising(1); // *** T1 rising ***
            self.bus.addr.drive(addr);
            self.bus.ctrl.drive(Ctls::NONE);
            self.bus.outs.drive(Outs::M1);
            yield self.clock.falling(1); // *** T1 falling ***
            self.bus.ctrl.drive(Ctls::MREQ | Ctls::RD);
            yield self.clock.falling(1); // *** T2 falling ***
            while self.bus.wait.sample().unwrap_or(false) {
                yield self.clock.falling(1); // wait 1 t-cycle
            }
            yield self.clock.rising(1); // *** T3 rising ***
            let byte = self.bus.data.sample().unwrap();
            self.bus.addr.drive(self.rp(RegPair::IR).get());
            self.bus.ctrl.drive(Ctls::NONE); // clear MREQ & RD
            self.bus.outs.drive(Outs::RFSH);
            yield self.clock.falling(1); // *** T3 falling ***
            self.bus.ctrl.drive(Ctls::MREQ);
            yield self.clock.falling(1); // *** T4 falling ***
            self.bus.ctrl.drive(Ctls::NONE); // clear MREQ
            // Increment R (lower 7 bits)
            let r = self.rg(Reg::R).get();
            self.rg(Reg::R).set(((r + 1) & 0x7f) | (r & 0x80));
            return byte;
        }
    }

    /// Memory read m-cycle. Takes 3 t-cycles.
    fn memory_read<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.addr.drive(addr);
            self.bus.ctrl.drive(Ctls::NONE);
            self.bus.outs.drive(Outs::NONE);
            yield self.clock.falling(1); // T1 falling
            self.bus.ctrl.drive(Ctls::MREQ | Ctls::RD);
            yield self.clock.falling(1); // T2 falling
            while self.bus.wait.sample().unwrap_or(false) {
                yield self.clock.falling(1); // wait 1 t-cycle
            }
            yield self.clock.falling(1); // T3 falling
            let byte = self.bus.data.sample().unwrap();
            self.bus.ctrl.drive(Ctls::NONE);
            return byte;
        }
    }

    /// Memory write m-cycle. Takes 3 t-cycles
    fn memory_write<'a>(&'a self, addr: u16, val: u8) -> impl Task<()> + 'a {
        move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.addr.drive(addr);
            self.bus.ctrl.drive(Ctls::NONE);
            self.bus.outs.drive(Outs::NONE);
            yield self.clock.falling(1); // T1 falling
            let release_data = self.bus.data.drive_and_release(val);
            self.bus.ctrl.drive(Ctls::MREQ);
            yield self.clock.falling(1); // T2 falling
            self.bus.ctrl.drive(Ctls::MREQ | Ctls::WR);
            while self.bus.wait.sample().unwrap_or(false) {
                yield self.clock.falling(1); // wait 1 t-cycle
            }
            yield self.clock.falling(1); // T3 falling
            self.bus.ctrl.drive(Ctls::NONE);
            release_data();
        }
    }

    /// Copy memory byte (DE)<-(HL), inc or dec HL&DE, dec BC
    fn memory_copy<'a>(&'a self, increment: i8, repeat: bool) -> impl Task<()> + 'a {
        move || {
            let src = self.rp(RegPair::HL).get();
            let dst = self.rp(RegPair::DE).get();
            let counter = self.rp(RegPair::BC).get().wrapping_sub(1);
            let value = yield_from!(self.memory_read(src));
            yield_from!(self.memory_write(dst, value));
            yield self.clock.rising(2); // complement MW to 5 t-cycles
            self.rp(RegPair::HL).set(src.wrapping_add(increment as u16));
            self.rp(RegPair::DE).set(dst.wrapping_add(increment as u16));
            self.rp(RegPair::BC).set(counter);
            let summ = self.rg(Reg::A).get() + value;
            let mut flags = self.get_flags() & (Flags::C | Flags::Z | Flags::S);
            flags.set(Flags::P, counter != 0);
            flags.set(Flags::Y, summ & (1 << 1) != 0);
            flags.set(Flags::X, summ & (1 << 3) != 0);
            self.set_flags(flags);
            if repeat && counter != 0 {
                yield self.clock.rising(5);
                self.next_pc(-2); // rewind PC 2 bytes back
            }
        }
    }

    /// IO read m-cycle
    fn io_read<'a>(&'a self, addr: u16) -> impl Task<u8> + 'a {
        move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.addr.drive(addr);
            self.bus.ctrl.drive(Ctls::NONE);
            self.bus.outs.drive(Outs::NONE);
            yield self.clock.rising(1); // T2 rising
            self.bus.ctrl.drive(Ctls::IORQ | Ctls::RD);
            yield self.clock.falling(2); // TW falling
            while self.bus.wait.sample().unwrap_or(false) {
                yield self.clock.falling(1); // wait 1 t-cycle
            }
            yield self.clock.falling(1); // T3 falling
            let byte = self.bus.data.sample().expect("Expecting data on a bus");
            self.bus.ctrl.drive(Ctls::NONE);
            return byte;
        }
    }

    /// IO write m-cycle
    fn io_write<'a>(&'a self, addr: u16, val: u8) -> impl Task<()> + 'a {
        move || {
            yield self.clock.rising(1); // T1 rising
            self.bus.addr.drive(addr);
            self.bus.ctrl.drive(Ctls::NONE);
            self.bus.outs.drive(Outs::NONE);
            yield self.clock.falling(1); // T1 falling
            let release_data = self.bus.data.drive_and_release(val);
            yield self.clock.rising(1); // T2 rising
            self.bus.ctrl.drive(Ctls::IORQ | Ctls::WR);
            yield self.clock.falling(2); // TW falling
            while self.bus.wait.sample().unwrap_or(false) {
                yield self.clock.falling(1); // wait 1 t-cycle
            }
            yield self.clock.falling(1); // T3 falling
            self.bus.ctrl.drive(Ctls::NONE);
            release_data();
        }
    }

}