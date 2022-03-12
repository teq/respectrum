use eframe::egui::*;
use librespectrum::cpu;
use std::rc::Rc;

use super::{SubWindow, draw_window};

pub struct CpuWindow {
    cpu_state: Rc<cpu::CpuState>,
}

impl CpuWindow {

    pub fn new(cpu_state: Rc<cpu::CpuState>) -> Self {
        Self { cpu_state }
    }

}

impl SubWindow for CpuWindow {

    fn name(&self) -> String { String::from("CPU") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        let reg_label = |ui: &mut Ui, label: &str, value: u16| {
            ui.label(label);
            ui.label(format!("{:04X}", value));
        };

        let flag_label = |ui: &mut Ui, label: &str, is_set: bool| {
            let color = if is_set { Color32::GREEN } else { Color32::GRAY };
            ui.colored_label(color, label);
        };

        draw_window(self.name(), focused, ctx, |ui| {

            Grid::new("cpu_regs").min_col_width(20.0).show(ui, |ui| {

                reg_label(ui, "AF:", self.cpu_state.af.word().get());
                reg_label(ui, "AF':", self.cpu_state.alt_af.word().get());
                ui.end_row();

                reg_label(ui, "BC:", self.cpu_state.bc.word().get());
                reg_label(ui, "BC':", self.cpu_state.alt_bc.word().get());
                ui.end_row();

                reg_label(ui, "DE:", self.cpu_state.de.word().get());
                reg_label(ui, "DE':", self.cpu_state.alt_de.word().get());
                ui.end_row();

                reg_label(ui, "HL:", self.cpu_state.hl.word().get());
                reg_label(ui, "HL':", self.cpu_state.alt_hl.word().get());
                ui.end_row();

                reg_label(ui, "IX:", self.cpu_state.ix.word().get());
                reg_label(ui, "IY:", self.cpu_state.iy.word().get());
                ui.end_row();

                reg_label(ui, "PC:", self.cpu_state.pc.word().get());
                reg_label(ui, "SP:", self.cpu_state.sp.word().get());
                ui.end_row();

            });

            ui.horizontal(|ui| {

                let flags = cpu::Flags::from(self.cpu_state.af.bytes().lo.get());
                flag_label(ui, "C", flags.contains(cpu::Flags::C));
                flag_label(ui, "N", flags.contains(cpu::Flags::N));
                flag_label(ui, "P", flags.contains(cpu::Flags::P));
                flag_label(ui, "X", flags.contains(cpu::Flags::X));
                flag_label(ui, "H", flags.contains(cpu::Flags::H));
                flag_label(ui, "Y", flags.contains(cpu::Flags::Y));
                flag_label(ui, "Z", flags.contains(cpu::Flags::Z));
                flag_label(ui, "S", flags.contains(cpu::Flags::S));

            });

            ui.horizontal(|ui| {
                ui.colored_label(Color32::WHITE, format!("IM{}", self.cpu_state.im));
                flag_label(ui, "IFF1", self.cpu_state.iff1);
                flag_label(ui, "IFF2", self.cpu_state.iff2);
            });

        })

    }

}
