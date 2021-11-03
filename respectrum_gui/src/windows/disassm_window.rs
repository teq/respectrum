use eframe::egui;

use super::Window;

pub struct DisassmWindow {}

impl Window for DisassmWindow {

    fn name(&self) -> &str {
        "Disassm"
    }

    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {
    }

}
