use std::cell::Cell;

use crate::cpu::tokens::IntMode;

use super::U16Cell;

/// Z80 CPU registers and state
#[derive(Default)]
pub struct CpuState {
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
}
