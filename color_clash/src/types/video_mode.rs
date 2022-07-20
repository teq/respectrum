use druid::Data;

/// ZX Spectrum video mode
#[derive(Clone, PartialEq, Data)]
pub enum VideoMode {
    /// Standart ZX video mode
    STD8X8,
    /// Multicolor with 8x4 attr blocks
    MUL8X4,
}
