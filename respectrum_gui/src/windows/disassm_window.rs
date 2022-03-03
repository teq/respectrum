use eframe::egui;

use super::Window;

pub struct DisassmWindow {}

impl DisassmWindow {

    pub fn new() -> Self {
        Self { }
    }

}

impl Window for DisassmWindow {

    fn name(&self) -> &str {
        "Disassm"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    }

}
