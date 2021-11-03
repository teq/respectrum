use librespectrum::cpu;
use eframe::egui;
use std::rc::Rc;

use super::Window;

pub struct CpuWindow {
    pub cpu_state: Rc<cpu::CpuState>,
}

impl Window for CpuWindow {

    fn name(&self) -> &str {
        "CPU"
    }

    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {

        egui::Window::new(self.name()).resizable(false).open(open).show(ctx, |ui| {

            egui::Grid::new("cpu_regs").min_col_width(20.0).show(ui, |ui| {

                let reg = |ui: &mut egui::Ui, label: &str, value: u16| {
                    ui.label(label);
                    ui.colored_label(egui::Color32::WHITE, format!("{:04X}h", value));
                };

                reg(ui, "AF:", self.cpu_state.af.word().get());
                reg(ui, "AF':", self.cpu_state.alt_af.word().get());
                ui.end_row();

                reg(ui, "BC:", self.cpu_state.bc.word().get());
                reg(ui, "BC':", self.cpu_state.alt_bc.word().get());
                ui.end_row();

                reg(ui, "DE:", self.cpu_state.de.word().get());
                reg(ui, "DE':", self.cpu_state.alt_de.word().get());
                ui.end_row();

                reg(ui, "HL:", self.cpu_state.hl.word().get());
                reg(ui, "HL':", self.cpu_state.alt_hl.word().get());
                ui.end_row();

                reg(ui, "IX:", self.cpu_state.ix.word().get());
                reg(ui, "IY:", self.cpu_state.iy.word().get());
                ui.end_row();

                reg(ui, "PC:", self.cpu_state.pc.word().get());
                reg(ui, "SP:", self.cpu_state.sp.word().get());
                ui.end_row();

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
