use crate::{
    mkword,
    spword,
    yield_task,
    bus::*,
    cpu::*,
};

/// Z80 CPU
pub struct Cpu<'a> {
    state: CpuState,
    bus: &'a CpuBus,
    clock: &'a Clock,
}

impl<'a> Device<'a> for Cpu<'a> {

    fn run(&'a self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(move || {

            // Instruction loop
            loop {

                let mut decoder = InstructionDecoder::new();

                // Instruction decode loop
                while {

                    let pc = self.state.rp(RegPair::PC).get(); // read current PC
                    self.state.rp(RegPair::PC).set(pc + 1); // increment PC to the next memory location

                    // Read next byte using appropriate M-cycle
                    let byte: u8 = match decoder.upnext() {
                        TokenType::Opcode => yield_task!(self.opcode_read(pc)),
                        TokenType::Offset | TokenType::Operand => yield_task!(self.memory_read(pc))
                    };

                    // Decode byte
                    decoder.decode(byte)

                } {}

                match decoder.expect_opcode() {

                    // 8-bit Load

                    Token::LD_RG_RG(dst @ (Reg::AtIX | Reg::AtIY), src) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.state.idx_addr(dst, decoder.expect_offset());
                        yield_task!(self.memory_write(addr, self.state.rg(src).get()));
                    },
                    Token::LD_RG_RG(dst, src @ (Reg::AtIX | Reg::AtIY)) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.state.idx_addr(src, decoder.expect_offset());
                        self.state.rg(dst).set(yield_task!(self.memory_read(addr)));
                    },
                    Token::LD_RG_RG(Reg::AtHL, src) => {
                        let addr = self.state.rp(RegPair::HL).get();
                        yield_task!(self.memory_write(addr, self.state.rg(src).get()));
                    },
                    Token::LD_RG_RG(dst, Reg::AtHL) => {
                        let addr = self.state.rp(RegPair::HL).get();
                        self.state.rg(dst).set(yield_task!(self.memory_read(addr)))
                    },
                    Token::LD_RG_RG(dst @ (Reg::I | Reg::R), Reg::A) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        self.state.rg(dst).set(self.state.rg(Reg::A).get());
                    },
                    Token::LD_RG_RG(Reg::A, src @ (Reg::I | Reg::R)) => {
                        yield self.clock.rising(1); // complement M1 to 5 t-cycles
                        self.state.rg(Reg::A).set(self.state.rg(src).get());
                    },
                    Token::LD_RG_RG(dst, src) => {
                        self.state.rg(dst).set(self.state.rg(src).get());
                    },
                    Token::LD_RG_N(Reg::AtHL) => {
                        let addr = self.state.rp(RegPair::HL).get();
                        yield_task!(self.memory_write(addr, decoder.expect_byte_operand()));
                    },
                    Token::LD_RG_N(reg) => {
                        self.state.rg(reg).set(decoder.expect_byte_operand());
                    },
                    Token::LD_A_AtRP(rpair) => {
                        let addr = self.state.rp(rpair).get();
                        self.state.rg(Reg::A).set(yield_task!(self.memory_read(addr)));
                    },
                    Token::LD_AtRP_A(rpair) => {
                        let addr = self.state.rp(rpair).get();
                        yield_task!(self.memory_write(addr, self.state.rg(Reg::A).get()));
                    },
                    Token::LD_A_MM => {
                        let addr = decoder.expect_word_operand();
                        self.state.rg(Reg::A).set(yield_task!(self.memory_read(addr)));
                    },
                    Token::LD_MM_A => {
                        let addr = decoder.expect_word_operand();
                        yield_task!(self.memory_write(addr, self.state.rg(Reg::A).get()));
                    },

                    // 16-bit Load

                    Token::LD_RP_NN(rpair) => {
                        self.state.rp(rpair).set(decoder.expect_word_operand());
                    },
                    Token::LD_RP_MM(rpair) => {
                        let addr = decoder.expect_word_operand();
                        let lo = yield_task!(self.memory_read(addr));
                        let hi = yield_task!(self.memory_read(addr + 1));
                        self.state.rp(rpair).set(mkword!(hi, lo));
                    },
                    Token::LD_MM_RP(rpair) => {
                        let addr = decoder.expect_word_operand();
                        let (hi, lo) = spword!(self.state.rp(rpair).get());
                        yield_task!(self.memory_write(addr, lo));
                        yield_task!(self.memory_write(addr + 1, hi));
                    },
                    Token::LD_SP_RP(rpair) => {
                        yield self.clock.rising(2); // complement M1 to 6 t-cycles
                        self.state.rp(RegPair::SP).set(self.state.rp(rpair).get());
                    },
                    Token::POP(rpair) => {
                        unimplemented!();
                    },
                    Token::PUSH(rpair) => {
                        unimplemented!();
                    },

                    // Exchange

                    Token::EX_DE_HL => self.state.swap_hlde(),
                    Token::EX_AF => self.state.swap_af(),
                    Token::EXX => self.state.swap_regfile(),
                    Token::EX_AtSP_RP(rpair) => {
                        unimplemented!();
                    },

                    // 8-bit arithmetic and logic

                    Token::ALU_N(op) => {
                        unimplemented!();
                    },
                    Token::ALU_RG(op, reg) => {
                        unimplemented!();
                    },
                    Token::INC_RG(reg) => {
                        unimplemented!();
                    },
                    Token::DEC_RG(reg) => {
                        unimplemented!();
                    },

                    // General-Purpose Arithmetic and CPU Control

                    Token::DAA => {
                        unimplemented!();
                    },
                    Token::CPL => {
                        unimplemented!();
                    },
                    Token::NEG => {
                        unimplemented!();
                    },
                    Token::CCF => {
                        unimplemented!();
                    },
                    Token::SCF => {
                        unimplemented!();
                    },
                    Token::NOP => {
                        unimplemented!();
                    },
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
                    Token::LDSH(dst, op, src) => {
                        unimplemented!();
                    },

                    // Bit Set, Reset and Test

                    Token::BIT(bit, reg) => {
                        unimplemented!();
                    },
                    Token::SET(bit, reg) => {
                        unimplemented!();
                    },
                    Token::LDSET(dst, bit, src) => {
                        unimplemented!();
                    },
                    Token::RES(bit, reg) => {
                        unimplemented!();
                    },
                    Token::LDRES(dst, bit, src) => {
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

                    // IO

                    Token::IN_A_N => {
                        let addr = mkword!(self.state.rg(Reg::A).get(), decoder.expect_byte_operand());
                        self.state.rg(Reg::A).set(yield_task!(self.io_read(addr)));
                    },
                    Token::OUT_N_A => {
                        let addr = mkword!(self.state.rg(Reg::A).get(), decoder.expect_byte_operand());
                        yield_task!(self.io_write(addr, self.state.rg(Reg::A).get()));
                    },
                    Token::IN_RG_AtBC(reg) => {
                        let addr = self.state.rp(RegPair::BC).get();
                        self.state.rg(reg).set(yield_task!(self.io_read(addr)));
                    },
                    Token::OUT_AtBC_RG(reg) => {
                        let addr = self.state.rp(RegPair::BC).get();
                        yield_task!(self.io_write(addr, self.state.rg(reg).get()));
                    },
                    Token::IN_AtBC => {
                        let addr = self.state.rp(RegPair::BC).get();
                        yield_task!(self.io_read(addr));
                    },
                    Token::OUT_AtBC_0 => {
                        let addr = self.state.rp(RegPair::BC).get();
                        yield_task!(self.io_write(addr, 0));
                    },

                    // Block transfer, search and IO

                    Token::BLOP(op) => {
                        unimplemented!();
                    },

                    // Non-opcode is not expected

                    Token::Prefix(_) | Token::Offset(_) | Token::Operand(_) => unreachable!()

                }

            }

        })

    }

}

impl<'a> Cpu<'a> {

    /// Instruction opcode fetch m-cycle
    /// (usually referred to as M1). Takes 4 t-cycles.
    fn opcode_read(&'a self, addr: u16) -> impl Task<u8> + 'a {
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
            let byte = self.bus.data.sample().expect("Expecting data on a bus");
            self.bus.addr.drive(self.state.rp(RegPair::IR).get());
            self.bus.ctrl.drive(Ctls::NONE); // clear MREQ & RD
            self.bus.outs.drive(Outs::RFSH);
            yield self.clock.falling(1); // *** T3 falling ***
            self.bus.ctrl.drive(Ctls::MREQ);
            yield self.clock.falling(1); // *** T4 falling ***
            self.bus.ctrl.drive(Ctls::NONE); // clear MREQ
            // Increment R (lower 7 bits)
            let r = self.state.rg(Reg::R).get();
            self.state.rg(Reg::R).set(((r + 1) & 0x7f) | (r & 0x80));
            return byte;
        }
    }

    /// Memory read m-cycle. Takes 3 t-cycles.
    fn memory_read(&'a self, addr: u16) -> impl Task<u8> + 'a {
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
            let byte = self.bus.data.sample().expect("Expecting data on a bus");
            self.bus.ctrl.drive(Ctls::NONE);
            return byte;
        }
    }

    /// Memory write m-cycle.
    fn memory_write(&'a self, addr: u16, val: u8) -> impl Task<()> + 'a {
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

    /// IO read m-cycle
    fn io_read(&'a self, addr: u16) -> impl Task<u8> + 'a {
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
    fn io_write(&'a self, addr: u16, val: u8) -> impl Task<()> + 'a {
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
