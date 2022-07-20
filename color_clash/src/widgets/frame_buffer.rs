use druid::{
    Data, Lens,
    im::Vector,
    piet::{ImageFormat, InterpolationMode},
    widget::prelude::*,
};

use crate::types::{
    VideoMode, Attributes,
    NORMAL_PALETTE, BRIGHT_PALETTE
};

/// ZX Spectrum frame buffer
#[derive(Clone, Lens, Data)]
pub struct FrameBuffer {
    /// Attribute columns
    cols: usize,
    /// Attribute rows
    rows: usize,
    /// Width in pixels
    width: usize,
    /// Height in pixels
    height: usize,
    /// Pixel width / height
    pixel_aspect_ratio: f64,
    /// Frame pixels
    pixels: Vector<bool>,
    /// Frame attributes
    attrs: Vector<Attributes>,
}

impl FrameBuffer {

    /// Creates frame buffer for given video mode
    pub fn new_for(mode: VideoMode) -> FrameBuffer {

        let pixel_aspect_ratio = 1.0f64;

        let (cols, rows, width, height) = match mode {
            VideoMode::STD8X8 => (32usize, 24usize, 256usize, 192usize),
            VideoMode::MUL8X4 => (32usize, 48usize, 256usize, 192usize),
        };

        FrameBuffer {
            cols, rows, width, height,
            pixel_aspect_ratio,
            pixels: vec![false; width * height].into(),
            attrs: vec![Default::default(); cols * rows].into(),
        }

    }

    /// Returns framebuffer as RGB pixel array
    pub fn to_rgb(&self) -> Vec<u8> {

        let mut result: Vec<u8> = Vec::with_capacity(self.width * self.height * 3);

        for y in 0..self.height {
            let row = y * self.rows / self.height;

            for x in 0..self.width {
                let col = x * self.cols / self.width;
                let attr = &self.attrs[row * self.cols + col];

                let color = if self.pixels[y * self.width + x] {
                    attr.get_ink()
                } else {
                    attr.get_paper()
                };

                let rgb = if attr.contains(Attributes::BRIGHT) {
                    &BRIGHT_PALETTE[color as usize]
                } else {
                    &NORMAL_PALETTE[color as usize]
                };

                result.extend(rgb);

            }

        }

        result

    }

}

pub struct FrameBufferView {}

impl Widget<FrameBuffer> for FrameBufferView {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FrameBuffer, env: &Env) {
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FrameBuffer, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &FrameBuffer, _data: &FrameBuffer, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FrameBuffer, _env: &Env) -> Size {
        bc.constrain((data.width as f64, data.height as f64))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FrameBuffer, _env: &Env) {
        let rect = ctx.size().to_rect();
        let image = ctx
            .make_image(data.width, data.height, &data.to_rgb(), ImageFormat::Rgb)
            .unwrap();
        ctx.draw_image(&image, rect, InterpolationMode::NearestNeighbor);
    }

}
