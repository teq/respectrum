use druid::{
    widget::prelude::*,
    Data, WidgetPod, Point
};

pub struct ZStack<T> {
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>
}

impl<T: Data> ZStack<T> {

    pub fn new() -> Self {
        Self { children: vec!() }
    }

    pub fn add_child(&mut self, child: impl Widget<T> + 'static) {
        self.children.push(WidgetPod::new(Box::new(child)));
    }

    pub fn with_child(mut self, child: impl Widget<T> + 'static) -> Self {
        self.add_child(child);
        self
    }

}

impl<T: Data> Widget<T> for ZStack<T> {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for child in self.children.iter_mut() {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        for child in self.children.iter_mut() {
            child.layout(ctx, bc, data, env);
            child.set_origin(ctx, data, env, Point::ORIGIN);
        }
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.paint(ctx, data, env);
        }
    }

}
