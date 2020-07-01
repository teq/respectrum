use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

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

                let mut decoder = opcode_decoder();
                let mut upnext = TokenType::Opcode;
                let mut opcode: Option<Token> = None;
                let mut offset: Option<i8> = None;
                let mut byte_operand: Option<u8> = None;
                let mut word_operand: Option<u16> = None;

                // Instruction decode loop
                loop {

                    let state = {

                        let pc = self.state.rp(RegPair::PC).get(); // read current PC
                        self.state.rp(RegPair::PC).set(pc + 1); // increment PC to the next memory location

                        // Read next byte using appropriate M-cycle
                        let byte: u8 = match upnext {
                            TokenType::Opcode => yield_task!(self.opcode_read(pc)),
                            TokenType::Offset | TokenType::Operand => yield_task!(self.memory_read(pc))
                        };

                        // Decode byte with opcode decoder
                        Pin::new(&mut decoder).resume(byte)

                    };

                    match state {
                        GeneratorState::Yielded(result) | GeneratorState::Complete(result) => match result.token {
                            Token::Prefix(_) => (),
                            Token::Offset(value) => offset = Some(value),
                            Token::Operand(OperandValue::Byte(value)) => byte_operand = Some(value),
                            Token::Operand(OperandValue::Word(value)) => word_operand = Some(value),
                            token => opcode = Some(token)
                        }
                    }

                    if let GeneratorState::Yielded(DecodeResult { upnext: next_upnext, .. }) = state {
                        upnext = next_upnext;
                    } else if let GeneratorState::Complete(_) = state {
                        break;
                    }

                }

                match opcode.unwrap() {
                    Token::LD_RG_RG(dst @ (Reg::AtIX | Reg::AtIY), src) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.state.idx_addr(dst, offset.unwrap());
                        yield_task!(self.memory_write(addr, self.state.rg(src).get()));
                    },
                    Token::LD_RG_RG(dst, src @ (Reg::AtIX | Reg::AtIY)) => {
                        yield self.clock.rising(2); // complement M3 to 5 t-cycles
                        let addr = self.state.idx_addr(src, offset.unwrap());
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
                    Token::LD_AtRP_A(rpair) => {
                        let addr = self.state.rp(rpair).get();
                        yield_task!(self.memory_write(addr, self.state.rg(Reg::A).get()));
                    },
                    Token::LD_A_AtRP(rpair) => {
                        let addr = self.state.rp(rpair).get();
                        self.state.rg(Reg::A).set(yield_task!(self.memory_read(addr)));
                    },
                    Token::LD_MM_RP(rpair) => {
                        let addr = word_operand.unwrap();
                        let (hi, lo) = spword!(self.state.rp(rpair).get());
                        yield_task!(self.memory_write(addr, lo));
                        yield_task!(self.memory_write(addr + 1, hi));
                    },
                    Token::LD_RP_MM(rpair) => {
                        let addr = word_operand.unwrap();
                        let lo = yield_task!(self.memory_read(addr));
                        let hi = yield_task!(self.memory_read(addr + 1));
                        self.state.rp(rpair).set(mkword!(hi, lo));
                    },
                    Token::LD_MM_A => {
                        let addr = word_operand.unwrap();
                        yield_task!(self.memory_write(addr, self.state.rg(Reg::A).get()));
                    },
                    Token::LD_A_MM => {
                        let addr = word_operand.unwrap();
                        self.state.rg(Reg::A).set(yield_task!(self.memory_read(addr)));
                    },
                    Token::LD_SP_RP(rpair) => {
                        yield self.clock.rising(2); // complement M1 to 6 t-cycles
                        self.state.rp(RegPair::SP).set(self.state.rp(rpair).get());
                    },
                    Token::LD_RG_N(Reg::AtHL) => {
                        let addr = self.state.rp(RegPair::HL).get();
                        yield_task!(self.memory_write(addr, byte_operand.unwrap()));
                    },
                    Token::LD_RG_N(reg) => {
                        self.state.rg(reg).set(byte_operand.unwrap());
                    },
                    Token::LD_RP_NN(rpair) => {
                        self.state.rp(rpair).set(word_operand.unwrap());
                    },
                    Token::EX_AF => self.state.swap_af(),
                    Token::EXX => self.state.swap_regfile(),
                    Token::EX_DE_HL => self.state.swap_hlde(),

                    Token::OUT_N_A => {},
                    Token::IN_A_N => {},
                    Token::IN_RG_AtBC(reg) => {},
                    Token::OUT_AtBC_RG(reg) => {},
                    Token::IN_AtBC => {}, // undocumented
                    Token::OUT_AtBC_0 => {}, // undocumented

                    _ => unreachable!()

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
