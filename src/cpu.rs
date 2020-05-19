use std::fmt;
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "little")]
struct Word {
    low: u8,
    high: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "big")]
struct Word {
    high: u8,
    low: u8,
}

#[repr(C)]
union RegPair {
    word: u16,
    bytes: Word,
}

impl Default for RegPair {
    fn default() -> Self { Self { word: 0x0000 } }
}

impl fmt::Debug for RegPair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("0x{:04x}", unsafe { self.word }))
    }
}

#[derive(Default, Debug)]
struct RegFile {
    /// Accumulator and flags
    af: RegPair,
    bc: RegPair,
    de: RegPair,
    hl: RegPair,
}

#[derive(Default, Debug)]
pub struct Cpu {
    /// Primary register file
    pri: RegFile,
    /// Alternative register file
    alt: RegFile,
    ix: RegPair,
    iy: RegPair,
    sp: RegPair,
    pc: RegPair,
}

impl Cpu {
    pub fn init() -> Cpu {
        Default::default()
    }
}
