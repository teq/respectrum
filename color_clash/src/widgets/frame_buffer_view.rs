use druid::{
    piet::{ImageFormat, InterpolationMode, StrokeStyle},
    kurbo::{Line, Rect},
    widget::prelude::*,
    Event, MouseButton, Point, Color,
};

use crate::models::FrameBuffer;

pub struct FrameBufferView {
    pub base_zoom: f64,
    pub zoom: f64,
    pub image: Vec<u8>,
}

impl FrameBufferView {

    pub fn new() -> Self {
        Self { base_zoom: 2.0, zoom: 1.0, image: vec!() }
    }

    fn effective_zoom(&self) -> f64 {
        self.base_zoom * self.zoom
    }

    fn widget2fb_coords(&self, point: Point) -> (usize, usize) {
        (
            (point.x / self.effective_zoom()) as usize,
            (point.y / self.effective_zoom()) as usize
        )
    }

}

impl Widget<FrameBuffer> for FrameBufferView {

    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, data: &mut FrameBuffer, _env: &Env) {
        match event {
            Event::MouseDown(e) if e.button == MouseButton::Left => {
                let (x, y) = self.widget2fb_coords(e.pos);
                data.pixels[data.width * y + x] = true;
            },
            Event::MouseMove(e) => {},
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FrameBuffer, _env: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &FrameBuffer, data: &FrameBuffer, _env: &Env) {
        if !data.same(old_data) {
            self.image = data.to_rgb_image();
            ctx.request_paint();
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

        // Draw image
        let image_size = ctx.size();
        let image = ctx.make_image(data.width, data.height, &self.image, ImageFormat::Rgb).unwrap();
        ctx.draw_image(&image, image_size.to_rect(), InterpolationMode::NearestNeighbor);

        // Draw attribute grid
        let stroke_style = StrokeStyle::new().dash_pattern(&[4.0, 4.0]);
        ctx.stroke_styled(
            Rect::from_origin_size(Point::ORIGIN, image_size),
            &Color::YELLOW, 1.0, &stroke_style
        );

    }

}
