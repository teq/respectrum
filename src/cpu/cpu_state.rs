use std::{fmt, cell::Cell};

use crate::cpu::*;

/// Z80 CPU state
#[derive(Default)]
pub struct CpuState {
    af: Word,
    bc: Word,
    de: Word,
    hl: Word,
    alt_af: Word,
    alt_bc: Word,
    alt_de: Word,
    alt_hl: Word,
    ix: Word,
    iy: Word,
    sp: Word,
    pc: Word,
    ir: Word,
    pub iff1: bool,
    pub iff2: bool,
    im: u8,
}

#[repr(C)]
union Word {
    w: Cell<u16>,
    b: WordBytes,
}

#[repr(C)]
#[cfg(target_endian = "little")]
struct WordBytes {
    lo: Cell<u8>,
    hi: Cell<u8>,
}

#[repr(C)]
#[cfg(target_endian = "big")]
struct WordBytes {
    hi: Cell<u8>,
    lo: Cell<u8>,
}

impl CpuState {

    /// Swap primary and alternate AF
    pub fn swap_af(&self) {
        unsafe { self.af.w.swap(&self.alt_af.w); }
    }

    /// Swap primary and alternate BC,DE and HL
    pub fn swap_regfile(&self) {
        unsafe {
            self.bc.w.swap(&self.alt_bc.w);
            self.de.w.swap(&self.alt_de.w);
            self.hl.w.swap(&self.alt_hl.w);
        }
    }

    /// Swap HL and DE
    pub fn swap_hlde(&self) {
        unsafe { self.hl.w.swap(&self.de.w); }
    }

    /// Get reference to register value
    pub fn rg(&self, reg: Reg) -> &Cell<u8> {
        match reg {
            Reg::B   => unsafe { &self.bc.b.hi },
            Reg::C   => unsafe { &self.bc.b.lo },
            Reg::D   => unsafe { &self.de.b.hi },
            Reg::E   => unsafe { &self.de.b.lo },
            Reg::H   => unsafe { &self.hl.b.hi },
            Reg::L   => unsafe { &self.hl.b.lo },
            Reg::A   => unsafe { &self.af.b.hi },
            Reg::F   => unsafe { &self.af.b.lo },
            Reg::I   => unsafe { &self.ir.b.hi },
            Reg::R   => unsafe { &self.ir.b.lo },
            Reg::IXH => unsafe { &self.ix.b.hi },
            Reg::IXL => unsafe { &self.ix.b.lo },
            Reg::IYH => unsafe { &self.iy.b.hi },
            Reg::IYL => unsafe { &self.iy.b.lo },
            _ => panic!("Unable to get register: {:#?}", reg)
        }
    }

    /// Get reference to regpair value
    pub fn rp(&self, rpair: RegPair) -> &Cell<u16> {
        match rpair {
            RegPair::BC => unsafe { &self.bc.w },
            RegPair::DE => unsafe { &self.de.w },
            RegPair::HL => unsafe { &self.hl.w },
            RegPair::AF => unsafe { &self.af.w },
            RegPair::SP => unsafe { &self.sp.w },
            RegPair::PC => unsafe { &self.pc.w },
            RegPair::IR => unsafe { &self.ir.w },
            RegPair::IX => unsafe { &self.ix.w },
            RegPair::IY => unsafe { &self.iy.w },
            _ => panic!("Unable to get register pair: {:#?}", rpair)
        }
    }

    /// Calculate absolute address for IX+d or IY+d
    pub fn idx_addr(&self, reg: Reg, offset: i8) -> u16 {
        let rpair = match reg {
            Reg::AtIX => RegPair::IX,
            Reg::AtIY => RegPair::IY,
            _ => panic!("Expecting (IX+d) or (IY+d), got: {:#?}", reg)
        };
        let addr = self.rp(rpair).get() as i32 + offset as i32;
        return addr as u16;
    }

}

impl Default for Word {
    fn default() -> Self { Self { w: Cell::new(0) } }
}

impl fmt::Debug for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:04x}h", unsafe { self.w.get() } ))
    }
}
