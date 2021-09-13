use eframe::egui;

use super::Window;

pub struct DisassmWindow {
    pub open: bool,
}

impl Default for DisassmWindow {
    fn default() -> Self {
        Self {
            open: true
        }
    }
}

impl Window for DisassmWindow {

    fn update(&mut self, ctx: &egui::CtxRef) {
    }

}
