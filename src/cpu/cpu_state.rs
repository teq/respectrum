use std::fmt;

use crate::{
    cpu::*,
};

/// Z80 CPU state
#[derive(Default)]
pub struct CpuState {
    pri: RegFile,
    alt: RegFile,
    ix: Word,
    iy: Word,
    sp: Word,
    pc: Word,
    ir: Word,
    iff1: bool,
    iff2: bool,
    im: u8,
}

#[derive(Default)]
struct RegFile {
    af: Word,
    bc: Word,
    de: Word,
    hl: Word,
}

#[repr(C)]
union Word {
    w: u16,
    b: WordBytes,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "little")]
struct WordBytes {
    lo: u8,
    hi: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "big")]
struct WordBytes {
    hi: u8,
    lo: u8,
}

impl CpuState {

    /// Get current PC value incrementing it by 1
    pub fn next_pc(&mut self) -> u16 {
        let pc = unsafe { self.pc.w };
        self.pc.w = pc + 1;
        return pc;
    }

    /// Get mutable reference to register value
    pub fn rg(&mut self, reg: Reg) -> &mut u8 {
        match reg {
            Reg::B   => unsafe { &mut self.pri.bc.b.hi },
            Reg::C   => unsafe { &mut self.pri.bc.b.lo },
            Reg::D   => unsafe { &mut self.pri.de.b.hi },
            Reg::E   => unsafe { &mut self.pri.de.b.lo },
            Reg::H   => unsafe { &mut self.pri.hl.b.hi },
            Reg::L   => unsafe { &mut self.pri.hl.b.lo },
            Reg::A   => unsafe { &mut self.pri.af.b.hi },
            Reg::F   => unsafe { &mut self.pri.af.b.lo },
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
    pub fn rp(&mut self, rpair: RegPair) -> &mut u16 {
        match rpair {
            RegPair::BC => unsafe { &mut self.pri.bc.w },
            RegPair::DE => unsafe { &mut self.pri.de.w },
            RegPair::HL => unsafe { &mut self.pri.hl.w },
            RegPair::AF => unsafe { &mut self.pri.af.w },
            RegPair::SP => unsafe { &mut self.sp.w },
            RegPair::IR => unsafe { &mut self.ir.w },
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
        let addr = *self.rp(index_rpair) as i32 + offset as i32;
        return addr as u16;
    }

}

impl Default for Word {
    fn default() -> Self { Self { w: 0 } }
}

impl fmt::Debug for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:04x}h", unsafe { self.w } ))
    }
}
