use super::Color;

pub struct BlendingMode {
    pixel: PixelBlending,
    color: ColorBlending,
}

pub enum PixelBlending {
    SET,
    RESET,
    TOGGLE,
    NOP,
}

pub enum ColorBlending {
    SET(Color),
    NOP,
}
