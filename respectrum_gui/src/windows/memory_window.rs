use eframe::egui;

use super::Window;

pub struct MemoryWindow {
    pub open: bool,
}

impl Default for MemoryWindow {
    fn default() -> Self {
        Self {
            open: true
        }
    }
}

impl Window for MemoryWindow {

    fn update(&mut self, ctx: &egui::CtxRef) {

        egui::Window::new("Memory").resizable(false).open(&mut self.open).show(ctx, |ui| {
            egui::Grid::new("hexdump").min_col_width(0.0).show(ui, |ui| {
                ui.label("12"); ui.label("56"); ui.label("90"); ui.label("CD"); ui.end_row();
                ui.label("23"); ui.label("67"); ui.label("0A"); ui.label("DE"); ui.end_row();
                ui.label("34"); ui.label("78"); ui.label("AB"); ui.label("EF"); ui.end_row();
                ui.label("45"); ui.label("89"); ui.label("BC"); ui.label("F0"); ui.end_row();
            });
        });

    }

}
