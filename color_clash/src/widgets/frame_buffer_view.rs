use druid::{
    piet::{ImageFormat, InterpolationMode},
    widget::prelude::*
};

use crate::models::FrameBuffer;

pub struct FrameBufferView {
    pub base_zoom: f64,
    pub zoom: f64,
    pub image: Vec<u8>,
}

impl FrameBufferView {

    pub fn new() -> Self {
        Self { base_zoom: 4.0, zoom: 1.0, image: vec!() }
    }

}

impl Widget<FrameBuffer> for FrameBufferView {

    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut FrameBuffer, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FrameBuffer, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, old_data: &FrameBuffer, data: &FrameBuffer, _env: &Env) {
        if !data.same(old_data) {
            self.image = data.to_rgb_image();
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FrameBuffer, _env: &Env) -> Size {
        bc.constrain((
            data.width as f64 * self.base_zoom * self.zoom,
            data.height as f64 * self.base_zoom * self.zoom
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FrameBuffer, _env: &Env) {
        if self.image.is_empty() {
            self.image = data.to_rgb_image();
        }
        let rect = ctx.size().to_rect();
        let image = ctx.make_image(data.width, data.height, &self.image, ImageFormat::Rgb).unwrap();
        ctx.draw_image(&image, rect, InterpolationMode::NearestNeighbor);
    }

}
