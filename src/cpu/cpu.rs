use crate::types::*;

#[derive(Default, Debug)]
struct RegFile {
    /// Accumulator and flags
    af: Word,
    bc: Word,
    de: Word,
    hl: Word,
}

#[derive(Default, Debug)]
pub struct Cpu {
    /// Primary register file
    pri: RegFile,
    /// Alternative register file
    alt: RegFile,
    ix: Word,
    iy: Word,
    /// Stack pointer
    sp: Word,
    pc: Word,
}

impl Cpu {
    pub fn init() -> Cpu {
        Default::default()
    }
}
