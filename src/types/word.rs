use std::fmt;

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "little")]
struct WordBytes {
    low: u8,
    high: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_endian = "big")]
struct WordBytes {
    high: u8,
    low: u8,
}

#[repr(C)]
pub union Word {
    word: u16,
    bytes: WordBytes,
}

impl Default for Word {
    fn default() -> Self { Self { word: 0x0000 } }
}

impl fmt::Debug for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("0x{:04x}", unsafe { self.word }))
    }
}
