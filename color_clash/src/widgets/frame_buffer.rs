use druid::{
    piet::{ImageFormat, InterpolationMode},
    widget::prelude::*,
};

use crate::models::FrameBuffer;

pub struct FrameBufferView {}

impl Widget<FrameBuffer> for FrameBufferView {

    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut FrameBuffer, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FrameBuffer, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &FrameBuffer, _data: &FrameBuffer, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FrameBuffer, _env: &Env) -> Size {
        bc.constrain((data.width as f64, data.height as f64))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FrameBuffer, _env: &Env) {
        let rect = ctx.size().to_rect();
        let image = ctx
            .make_image(data.width, data.height, &data.to_rgb_image(), ImageFormat::Rgb)
            .unwrap();
        ctx.draw_image(&image, rect, InterpolationMode::NearestNeighbor);
    }

}
