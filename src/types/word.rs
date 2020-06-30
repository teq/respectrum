use std::fmt;

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "little")]
pub struct WordBytes {
    pub lo: u8,
    pub hi: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "big")]
pub struct WordBytes {
    pub hi: u8,
    pub lo: u8,
}

#[repr(C)]
pub union Word {
    pub w: u16,
    pub b: WordBytes,
}

impl Default for Word {
    fn default() -> Self { Self { w: 0 } }
}

impl Word {
    pub fn w(&self)  -> u16 { unsafe { self.w    } }
    pub fn hi(&self) -> u8  { unsafe { self.b.hi } }
    pub fn lo(&self) -> u8  { unsafe { self.b.lo } }
}

impl fmt::Debug for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:04x}h", self.w()))
    }
}
