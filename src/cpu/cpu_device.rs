use std::{
    rc::Rc,
    pin::Pin,
    cell::Cell,
    ops::{Generator, GeneratorState},
};

use crate::{
    mkword,
    spword,
    yield_task,
    bus::*,
    cpu::*,
};

/// Z80 CPU device
pub struct CpuDevice {
    state: CpuState,
    bus: Rc<CpuBus>,
    clock: Rc<Clock>,
}

impl Device for CpuDevice {

    fn run<'a>(&'a mut self) -> Box<dyn NoReturnTask + 'a> {

        Box::new(move || {

            loop {

                let mut decoder = opcode_decoder();
                let mut upnext = TokenType::Opcode;
                let mut opcode: Option<Token> = None;
                let mut offset: Option<i8> = None;
                let mut operand: Option<OperandValue> = None;

                loop {

                    let pc = self.state.next_pc();

                    let byte = match upnext {
                        TokenType::Opcode => yield_task!(self.opcode_read(pc)),
                        TokenType::Offset | TokenType::Operand => yield_task!(self.memory_read(pc))
                    };

                    let state = Pin::new(&mut decoder).resume(byte);

                    match state {
                        GeneratorState::Yielded(result) | GeneratorState::Complete(result) => match result {
                            DecodeResult { upnext: next_upnext, .. } => upnext = next_upnext,
                        }
                    }

                    match state {
                        GeneratorState::Yielded(result) | GeneratorState::Complete(result) => match result {
                            DecodeResult { token: Token::Prefix(_), .. } => (),
                            DecodeResult { token: Token::Offset(value), .. } => offset = Some(value),
                            DecodeResult { token: Token::Operand(value), .. } => operand = Some(value),
                            DecodeResult { token, .. } => opcode = Some(token),
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
                        yield self.clock.rising(2); // end M3
                        let addr = self.state.idx_addr(dst, offset.unwrap());
                        let value = *self.state.rg(src);
                        yield_task!(self.memory_write(addr, value));
                    },
                    Token::LD_RG_RG(dst, src @ (Reg::AtIX | Reg::AtIY)) => {
                        yield self.clock.rising(2); // end M3
                        let addr = self.state.idx_addr(src, offset.unwrap());
                        let value = yield_task!(self.memory_read(addr));
                        *self.state.rg(dst) = value;
                    },
                    Token::LD_RG_RG(Reg::AtHL, src) => {
                        let addr = *self.state.rp(RegPair::HL);
                        let value = *self.state.rg(src);
                        yield_task!(self.memory_write(addr, value));
                    },
                    Token::LD_RG_RG(dst, Reg::AtHL) => {
                        let addr = *self.state.rp(RegPair::HL);
                        let value = yield_task!(self.memory_read(addr));
                        *self.state.rg(dst) = value;
                    },
                    Token::LD_RG_RG(dst @ (Reg::I | Reg::R), Reg::A) => {
                        yield self.clock.rising(1);
                        let value = *self.state.rg(Reg::A);
                        *self.state.rg(dst) = value;
                    },
                    Token::LD_RG_RG(Reg::A, src @ (Reg::I | Reg::R)) => {
                        yield self.clock.rising(1);
                        let value = *self.state.rg(src);
                        *self.state.rg(Reg::A) = value;
                    },
                    Token::LD_RG_RG(dst, src) => {
                        let value = *self.state.rg(src);
                        *self.state.rg(dst) = value;
                    },

                    Token::LD_AtRP_A(rpair) => {
                        let addr = *self.state.rp(rpair);
                        let value = *self.state.rg(Reg::A);
                        yield_task!(self.memory_write(addr, value));
                    },
                    Token::LD_A_AtRP(rpair) => {
                        let addr = *self.state.rp(rpair);
                        let value = yield_task!(self.memory_read(addr));
                        *self.state.rg(Reg::A) = value;
                    },

                    Token::LD_MM_RP(rpair) => {
                        if let Some(OperandValue::Word(addr)) = operand {
                            let (hi, lo) = spword!(*self.state.rp(rpair));
                            yield_task!(self.memory_write(addr, lo));
                            yield_task!(self.memory_write(addr + 1, hi));
                        } else {
                            panic!("Expecting address operand");
                        }
                    },
                    Token::LD_RP_MM(rpair) => {
                        if let Some(OperandValue::Word(addr)) = operand {
                            let lo = yield_task!(self.memory_read(addr));
                            let hi = yield_task!(self.memory_read(addr + 1));
                            *self.state.rp(rpair) = mkword!(hi, lo);
                        } else {
                            panic!("Expecting address operand");
                        }
                    },

                    Token::LD_MM_A => {},
                    Token::LD_A_MM => {},
                    Token::LD_SP_RP(rpair) => {},
                    Token::LD_RG_N(reg) => {},
                    Token::LD_RP_NN(rpair) => {},
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

impl CpuDevice {

    /// Create new CPU instance
    pub fn new(bus: Rc<CpuBus>, clock: Rc<Clock>) -> Self {
        Self {
            state: Default::default(),
            bus, clock
        }
    }

    /// Instruction opcode fetch m-cycle
    /// (usually referred to as M1). Takes 4 t-cycles.
    fn opcode_read<'a>(&'a mut self, addr: u16) -> impl Task<u8> + 'a {
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
            self.bus.addr.drive(*self.state.rp(RegPair::IR));
            self.bus.ctrl.drive(Ctls::NONE); // clear MREQ & RD
            self.bus.outs.drive(Outs::RFSH);
            yield self.clock.falling(1); // *** T3 falling ***
            self.bus.ctrl.drive(Ctls::MREQ);
            yield self.clock.falling(1); // *** T4 falling ***
            self.bus.ctrl.drive(Ctls::NONE); // clear MREQ
            // Increment R (lower 7 bits)
            let r = self.state.rg(Reg::R);
            *r = ((*r + 1) & 0x7f) | (*r & 0x80);
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
            let byte = self.bus.data.sample().expect("Expecting data on a bus");
            self.bus.ctrl.drive(Ctls::NONE);
            return byte;
        }
    }

    /// Memory write m-cycle.
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
