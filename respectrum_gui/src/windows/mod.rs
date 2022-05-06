use std::hash::Hash;
use eframe::egui::*;

mod bus_window;
pub use bus_window::BusWindow;
mod cpu_window;
pub use cpu_window::CpuWindow;
mod disassm_window;
pub use disassm_window::DisassmWindow;
mod memory_window;
pub use memory_window::MemoryWindow;

pub trait SubWindow {

    /// Window name
    fn name(&self) -> String;

    /// Window draw function
    fn show(&mut self, ctx: &Context, focused: bool) -> Response;

}

/// Draw widow with title
pub fn draw_window(
    name: impl Into<String> + Hash,
    focused: bool,
    ctx: &Context,
    add_contents: impl FnOnce(&mut Ui) -> (),
) -> Response {

    Area::new(&name).show(ctx, |ui| {

        Frame::window(&ctx.style())
            .inner_margin(style::Margin::same(0.0))
            .fill(if focused {Color32::LIGHT_BLUE} else {Color32::LIGHT_GRAY})
            .show(ui, |ui|
        {

            Frame::window(&ctx.style())
                .stroke(Stroke::none())
                .fill(Color32::TRANSPARENT)
                .show(ui, |ui|
            {
                ui.add(Label::new(RichText::new(name).color(
                    if focused {Color32::BLACK} else {Color32::GRAY}
                )));
            });

            ui.add_space(-3.0);

            Frame::window(&ctx.style())
                .rounding(Rounding {
                    nw: 0.0, ne: 0.0,
                    ..ctx.style().visuals.window_rounding
                })
                .stroke(Stroke::none())
                .show(ui, |ui|
            {
                add_contents(ui);
            });

        });

    }).response

}

pub fn cursor_color(focused: bool) -> Color32 {
    if focused {Color32::LIGHT_BLUE} else {Color32::LIGHT_GRAY}
}
