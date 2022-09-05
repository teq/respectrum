use super::ZXColor;
use druid::{Data, Lens};

#[derive(Default)]
#[derive(Clone, Lens, Data)]
pub struct BlendingMode {
    pixel: PixelBlending,
    ink: Option<ZXColor>,
    paper: Option<ZXColor>,
    bright: bool,
    flash: bool,
}

#[derive(Default)]
#[derive(Clone, PartialEq, Data)]
pub enum PixelBlending {
    #[default] XOR,
    SET,
    RES,
    NOP,
}
