use std::{fmt, mem};

use crate::cpu::*;

/// Z80 CPU state
#[derive(Default)]
pub struct CpuState {
    pri_af: Word,
    pri: RegFile,
    alt_af: Word,
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

#[derive(Default, Copy, Clone)]
struct RegFile {
    bc: Word,
    de: Word,
    hl: Word,
}

#[repr(C)]
#[derive(Copy, Clone)]
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

    /// Swap primary and alternate AF
    pub fn swap_af(&mut self) {
        mem::swap(&mut self.pri_af, &mut self.alt_af);
    }

    /// Swap primary and alternate BC,DE and HL
    pub fn swap_regfile(&mut self) {
        mem::swap(&mut self.pri, &mut self.alt);
    }

    /// Swap HL and DE
    pub fn swap_hlde(&mut self) {
        mem::swap(&mut self.pri.hl, &mut self.pri.de);
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
            Reg::A   => unsafe { &mut self.pri_af.b.hi },
            Reg::F   => unsafe { &mut self.pri_af.b.lo },
            Reg::I   => unsafe { &mut self.ir.b.hi },
            Reg::R   => unsafe { &mut self.ir.b.lo },
            Reg::IXH => unsafe { &mut self.ix.b.hi },
            Reg::IXL => unsafe { &mut self.ix.b.lo },
            Reg::IYH => unsafe { &mut self.iy.b.hi },
            Reg::IYL => unsafe { &mut self.iy.b.lo },
            _ => panic!("Unable to get register: {:#?}", reg)
        }
    }

    /// Get mutable reference to regpair value
    pub fn rp(&mut self, rpair: RegPair) -> &mut u16 {
        match rpair {
            RegPair::BC => unsafe { &mut self.pri.bc.w },
            RegPair::DE => unsafe { &mut self.pri.de.w },
            RegPair::HL => unsafe { &mut self.pri.hl.w },
            RegPair::AF => unsafe { &mut self.pri_af.w },
            RegPair::SP => unsafe { &mut self.sp.w },
            RegPair::PC => unsafe { &mut self.pc.w },
            RegPair::IR => unsafe { &mut self.ir.w },
            RegPair::IX => unsafe { &mut self.ix.w },
            RegPair::IY => unsafe { &mut self.iy.w },
            _ => panic!("Unable to get register pair: {:#?}", rpair)
        }
    }

    /// Calculate absolute address for IX+d or IY+d
    pub fn idx_addr(&mut self, reg: Reg, offset: i8) -> u16 {
        let rpair = match reg {
            Reg::AtIX => RegPair::IX,
            Reg::AtIY => RegPair::IY,
            _ => panic!("Expecting (IX+d) or (IY+d), got: {:#?}", reg)
        };
        let addr = *self.rp(rpair) as i32 + offset as i32;
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
