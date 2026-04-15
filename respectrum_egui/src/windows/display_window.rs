use std::rc::Rc;
use egui::*;

use librespectrum::devs::mem::Memory;

use super::{SubWindow, draw_window};

/// ZX Spectrum standard color palette (normal and bright variants)
const PALETTE: [[u8; 3]; 16] = [
    // Normal
    [0x00, 0x00, 0x00], // black
    [0x00, 0x00, 0xCD], // blue
    [0xCD, 0x00, 0x00], // red
    [0xCD, 0x00, 0xCD], // magenta
    [0x00, 0xCD, 0x00], // green
    [0x00, 0xCD, 0xCD], // cyan
    [0xCD, 0xCD, 0x00], // yellow
    [0xCD, 0xCD, 0xCD], // white
    // Bright
    [0x00, 0x00, 0x00], // black
    [0x00, 0x00, 0xFF], // blue
    [0xFF, 0x00, 0x00], // red
    [0xFF, 0x00, 0xFF], // magenta
    [0x00, 0xFF, 0x00], // green
    [0x00, 0xFF, 0xFF], // cyan
    [0xFF, 0xFF, 0x00], // yellow
    [0xFF, 0xFF, 0xFF], // white
];

const SCREEN_W: usize = 256;
const SCREEN_H: usize = 192;
const BITMAP_BASE: u16 = 0x4000;
const ATTR_BASE: u16 = 0x5800;

pub struct DisplayWindow {
    memory: Rc<dyn Memory>,
    pixels: Vec<Color32>,
    scale: usize,
    texture: Option<TextureHandle>,
}

impl DisplayWindow {

    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Self {
            memory,
            pixels: vec![Color32::BLACK; SCREEN_W * SCREEN_H],
            scale: 2,
            texture: None,
        }
    }

    /// Render the ZX Spectrum screen from memory into the pixel buffer
    fn render(&mut self) {
        for py in 0..SCREEN_H {
            // ZX Spectrum bitmap address calculation:
            // bit layout of py: [Y7 Y6 Y5 Y4 Y3 Y2 Y1 Y0]
            // address offset:   [Y7 Y6 Y2 Y1 Y0 Y5 Y4 Y3 0 0 0 0 0]
            let y7y6 = py & 0b1100_0000;
            let y5y3 = (py & 0b0011_1000) >> 3;
            let y2y0 = py & 0b0000_0111;
            let row_addr = BITMAP_BASE + ((y7y6 | y2y0 << 3 | y5y3) << 5) as u16;

            let attr_row = py / 8;

            for col in 0..32u16 {
                let byte = self.memory.read(row_addr + col);
                let attr = self.memory.read(ATTR_BASE + attr_row as u16 * 32 + col);

                let ink = (attr & 0x07) as usize;
                let paper = ((attr >> 3) & 0x07) as usize;
                let bright = if attr & 0x40 != 0 { 8 } else { 0 };

                let ink_rgb = PALETTE[ink + bright];
                let paper_rgb = PALETTE[paper + bright];

                for bit in 0..8 {
                    let px = col as usize * 8 + bit;
                    let set = byte & (0x80 >> bit) != 0;
                    let rgb = if set { ink_rgb } else { paper_rgb };
                    self.pixels[py * SCREEN_W + px] = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                }
            }
        }
    }

}

impl SubWindow for DisplayWindow {

    fn name(&self) -> String { String::from("Display") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        self.render();

        let image = ColorImage {
            size: [SCREEN_W, SCREEN_H],
            pixels: self.pixels.clone(),
        };

        let texture = ctx.load_texture("zx_screen", image);

        let response = draw_window(self.name(), focused, ctx, |ui| {
            let size = Vec2::new(
                (SCREEN_W * self.scale) as f32,
                (SCREEN_H * self.scale) as f32,
            );
            ui.image(texture.id(), size);
        });

        self.texture = Some(texture);

        response

    }

}
