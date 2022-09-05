use std::io::BufRead;
use bitvec::prelude::*;
use druid::{
    Data, Lens,
    im::Vector,
};

use crate::models::{VideoMode, Attributes};

/// ZX Spectrum frame buffer
#[derive(Clone, Lens, Data)]
pub struct FrameBuffer {
    /// Attribute columns
    pub cols: usize,
    /// Attribute rows
    pub rows: usize,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Frame pixels
    pub pixels: Vector<bool>,
    /// Frame attributes
    pub attrs: Vector<Attributes>,
}

impl FrameBuffer {

    /// Creates frame buffer for given video mode
    pub fn new_for(mode: VideoMode) -> Self {

        let (cols, rows, width, height) = match mode {
            VideoMode::STD8X8 => (32usize, 24usize, 256usize, 192usize),
            VideoMode::MUL8X4 => (32usize, 48usize, 256usize, 192usize),
        };

        Self {
            cols, rows, width, height,
            pixels: vec![false; width * height].into(),
            attrs: vec![Default::default(); cols * rows].into(),
        }

    }

    /// Reads framebuffer from file
    pub fn load(mut reader: Box<dyn BufRead>) -> Self {

        let mut buffer = [Default::default(); 6912];
        reader.read_exact(&mut buffer).unwrap();

        let (cols, rows, width, height) = (32usize, 24usize, 256usize, 192usize);
        let (pix_buf, attr_buf) = buffer.split_at(cols * height);

        let mut pixels = Vector::<bool>::new();

        for line in 0..height {
            let line_addr = (line & 0xc0) << 5 | (line & 0x7) << 8 | (line & 0x38) << 2;
            let line_buf = &pix_buf[line_addr..line_addr+cols];
            pixels.append(line_buf.view_bits::<Msb0>().iter().map(|x| x == true).collect());
        }

        Self {
            cols, rows, width, height, pixels,
            attrs: attr_buf.iter().map(|x| (*x).into()).collect(),
        }

    }

    /// Returns framebuffer as RGB pixel array
    pub fn to_rgb_image(&self) -> Vec<u8> {

        let mut result: Vec<u8> = Vec::with_capacity(self.width * self.height * 3);

        for y in 0..self.height {
            let row = y * self.rows / self.height;

            for x in 0..self.width {
                let col = x * self.cols / self.width;
                let attr = &self.attrs[row * self.cols + col];
                let pixel_rgb = if self.pixels[y * self.width + x] {
                    attr.get_ink_rgb()
                } else {
                    attr.get_paper_rgb()
                };
                result.extend(pixel_rgb);
            }
        }

        result

    }

}
