use egui::*;
use librespectrum::{cpu::Flags, bus::Scheduler, devs::Cpu};
use std::{rc::Rc, cell::RefCell};

use super::{SubWindow, draw_window};

pub struct CpuWindow<'a> {
    cpu: Rc<Cpu>,
    scheduler: Rc<RefCell<Scheduler<'a>>>,
}

impl<'a> CpuWindow<'a> {

    pub fn new(cpu: Rc<Cpu>, scheduler: Rc<RefCell<Scheduler<'a>>>) -> Self {
        CpuWindow { cpu, scheduler }
    }

    fn handle_keyboard(&mut self, input: &InputState) {

        if input.key_pressed(Key::Enter) {
            self.scheduler.borrow_mut().advance(1);
        }

    }

}

impl SubWindow for CpuWindow<'_> {

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

            self.handle_keyboard(&ui.input());

            Grid::new("cpu_regs").min_col_width(20.0).show(ui, |ui| {

                reg_label(ui, "AF:", self.cpu.af.value().get());
                reg_label(ui, "AF':", self.cpu.alt_af.value().get());
                ui.end_row();

                reg_label(ui, "BC:", self.cpu.bc.value().get());
                reg_label(ui, "BC':", self.cpu.alt_bc.value().get());
                ui.end_row();

                reg_label(ui, "DE:", self.cpu.de.value().get());
                reg_label(ui, "DE':", self.cpu.alt_de.value().get());
                ui.end_row();

                reg_label(ui, "HL:", self.cpu.hl.value().get());
                reg_label(ui, "HL':", self.cpu.alt_hl.value().get());
                ui.end_row();

                reg_label(ui, "IX:", self.cpu.ix.value().get());
                reg_label(ui, "IY:", self.cpu.iy.value().get());
                ui.end_row();

                reg_label(ui, "PC:", self.cpu.pc.value().get());
                reg_label(ui, "SP:", self.cpu.sp.value().get());
                ui.end_row();

            });

            ui.horizontal(|ui| {
                let flags = Flags::from(self.cpu.af.bytes().lo.get());
                flag_label(ui, "C", flags.contains(Flags::C));
                flag_label(ui, "N", flags.contains(Flags::N));
                flag_label(ui, "P", flags.contains(Flags::P));
                flag_label(ui, "X", flags.contains(Flags::X));
                flag_label(ui, "H", flags.contains(Flags::H));
                flag_label(ui, "Y", flags.contains(Flags::Y));
                flag_label(ui, "Z", flags.contains(Flags::Z));
                flag_label(ui, "S", flags.contains(Flags::S));
            });

            ui.horizontal(|ui| {
                ui.colored_label(Color32::WHITE, format!("IM{}", self.cpu.im));
                flag_label(ui, "IFF1", self.cpu.iff1);
                flag_label(ui, "IFF2", self.cpu.iff2);
            });

        })

    }

}
