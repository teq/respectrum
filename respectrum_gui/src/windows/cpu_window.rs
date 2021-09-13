use librespectrum::cpu;
use eframe::egui;
use std::rc::Rc;

use super::Window;

pub struct CpuWindow {
    pub open: bool,
    pub cpu_state: Rc<cpu::CpuState>,
}

impl Window for CpuWindow {

    fn update(&mut self, ctx: &egui::CtxRef) {

        egui::Window::new("CPU").resizable(false).open(&mut self.open).show(ctx, |ui| {

            egui::Grid::new("cpu_regs").min_col_width(20.0) .show(ui, |ui| {
                let reg = |ui: &mut egui::Ui, label: &str, value: &str| {
                    ui.label(label);  ui.colored_label(egui::Color32::WHITE, value);
                };
                reg(ui, "AF:", "0FA0"); reg(ui, "AF':", "0FE3"); ui.end_row();
                reg(ui, "BC:", "0FA0"); reg(ui, "BC':", "0FE3"); ui.end_row();
                reg(ui, "DE:", "0FA0"); reg(ui, "DE':", "0FE3"); ui.end_row();
                reg(ui, "HL:", "0FA0"); reg(ui, "HL':", "0FE3"); ui.end_row();
                reg(ui, "IX:", "0FA0"); reg(ui, "IY:",  "0FE3"); ui.end_row();
                reg(ui, "PC:", "0FA0"); reg(ui, "SP:",  "0FE3"); ui.end_row();
            });

            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::GREEN, "C");
                ui.colored_label(egui::Color32::GRAY, "N");
                ui.colored_label(egui::Color32::GRAY, "P");
                ui.colored_label(egui::Color32::GRAY, "X");
                ui.colored_label(egui::Color32::GRAY, "H");
                ui.colored_label(egui::Color32::GRAY, "Y");
                ui.colored_label(egui::Color32::GRAY, "Z");
                ui.colored_label(egui::Color32::GRAY, "S");
            });

            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::WHITE, "IM1");
                ui.colored_label(egui::Color32::GREEN, "IFF1");
                ui.colored_label(egui::Color32::GRAY, "IFF2");
            });

        });

    }

}
