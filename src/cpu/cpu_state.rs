use crate::{
    cpu::*,
    types::Word
};

#[derive(Default)]
pub struct RegFile {
    pub af: Word,
    pub bc: Word,
    pub de: Word,
    pub hl: Word,
}

/// Z80 CPU state
#[derive(Default)]
pub struct CpuState {
    pub pri: RegFile,
    pub alt: RegFile,
    pub ix: Word,
    pub iy: Word,
    pub sp: Word,
    pub pc: Word,
    pub ir: Word,
    pub iff1: bool,
    pub iff2: bool,
    pub im: u8,
}

impl CpuState {

    /// Get current PC value incrementing it by 1
    pub fn next_pc(&mut self) -> u16 {
        let pc = self.pc.w();
        self.pc.w = self.pc.w() + 1;
        return pc;
    }

    /// Get mutable reference to register value
    pub fn reg(&mut self, reg: Reg) -> &mut u8 {
        match reg {
            Reg::B   => unsafe { &mut self.pri.bc.b.hi },
            Reg::C   => unsafe { &mut self.pri.bc.b.lo },
            Reg::D   => unsafe { &mut self.pri.de.b.hi },
            Reg::E   => unsafe { &mut self.pri.de.b.lo },
            Reg::H   => unsafe { &mut self.pri.hl.b.hi },
            Reg::L   => unsafe { &mut self.pri.hl.b.lo },
            Reg::A   => unsafe { &mut self.pri.af.b.hi },
            Reg::I   => unsafe { &mut self.ir.b.hi },
            Reg::R   => unsafe { &mut self.ir.b.lo },
            Reg::IXH => unsafe { &mut self.ix.b.hi },
            Reg::IXL => unsafe { &mut self.ix.b.lo },
            Reg::IYH => unsafe { &mut self.iy.b.hi },
            Reg::IYL => unsafe { &mut self.iy.b.lo },
            _ => panic!("Invalid register: {:#?}", reg)
        }
    }

    /// Get mutable reference to regpair value
    pub fn rpair(&mut self, rpair: RegPair) -> &mut u16 {
        match rpair {
            RegPair::BC => unsafe { &mut self.pri.bc.w },
            RegPair::DE => unsafe { &mut self.pri.de.w },
            RegPair::HL => unsafe { &mut self.pri.hl.w },
            RegPair::AF => unsafe { &mut self.pri.af.w },
            RegPair::SP => unsafe { &mut self.sp.w },
            RegPair::IX => unsafe { &mut self.ix.w },
            RegPair::IY => unsafe { &mut self.iy.w },
            _ => panic!("Invalid register pair: {:#?}", rpair)
        }
    }

    /// Calculate absolute address for IX+d or IY+d
    pub fn idx_addr(&mut self, reg: Reg, offset: i8) -> u16 {
        let index_rpair = match reg {
            Reg::AtIX => RegPair::IX,
            Reg::AtIY => RegPair::IY,
            _ => panic!("Expecting (IX+d) or (IY+d), got: {:#?}", reg)
        };
        let addr = *self.rpair(index_rpair) as i32 + offset as i32;
        return addr as u16;
    }

}
