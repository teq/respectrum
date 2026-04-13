use egui::*;
use egui_extras::{Size, TableBuilder};
use librespectrum::{devs::{BusLogger, DeviceManager}, bus::Ctrl};
use std::rc::Rc;

use super::{SubWindow, draw_window};

pub struct BusWindow {
    logger: Rc<BusLogger>,
    device_manager: Rc<DeviceManager>,
}

impl BusWindow {
    pub fn new(logger: Rc<BusLogger>, device_manager: Rc<DeviceManager>) -> Self {
        Self { logger, device_manager }
    }

    // Helper to display optional boolean signal
    fn signal_cell(&self, ui: &mut egui::Ui, signal: Option<(usize, bool)>) {
        match signal {
            Some((owner, value)) => {
                let label = ui.label(if value { "H" } else { "L" });
                if let Some(device_name) = self.device_manager.get_name(owner) {
                    label.on_hover_text(format!("Device: {}", device_name));
                }
            }
            None => { ui.colored_label(Color32::GRAY, "-"); }
        }
    }

    // Helper to display control signal
    fn ctrl_cell(&self, ui: &mut egui::Ui, ctrl: Option<(usize, Ctrl)>, flag: Ctrl) {
        self.signal_cell(ui, ctrl.map(|(owner, ctrl_val)| (owner, ctrl_val.contains(flag))));
    }

    // Helper to display hex values
    fn hex_cell(&self, ui: &mut egui::Ui, value: Option<(usize, impl std::fmt::UpperHex)>, placeholder: &str) {
        match value {
            Some((owner, val)) => {
                let label = ui.label(format!("{:0width$X}", val, width = placeholder.len()));
                if let Some(device_name) = self.device_manager.get_name(owner) {
                    label.on_hover_text(format!("Device: {}", device_name));
                }
            }
            None => { ui.colored_label(Color32::GRAY, placeholder); }
        }
    }
}

impl SubWindow for BusWindow {

    fn name(&self) -> String { String::from("Bus") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        draw_window(self.name(), focused, ctx, |ui| {

            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            let headers = ["T", "ADDR", "DATA", "RD", "WR", "MREQ", "IORQ", "RFSH", "M1", "BUSRQ", "BUSAK", "WAIT", "HALT", "INT", "NMI", "RESET"];

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
                .columns(Size::initial(30.0), headers.len())
                .header(text_height, |mut header| {
                    for &label in &headers {
                        header.col(|ui| { ui.label(label); });
                    }
                })
                .body(|mut body| {
                    for reading in self.logger.readings.borrow().iter_to_tail().take(32) {
                        body.row(text_height, |mut row| {
                            row.col(|ui| { ui.label(format!("{}", reading.htcyc)); });
                            row.col(|ui| { self.hex_cell(ui, reading.addr, "----"); });
                            row.col(|ui| { self.hex_cell(ui, reading.data, "--"); });
                            row.col(|ui| { self.ctrl_cell(ui, reading.ctrl, Ctrl::RD); });
                            row.col(|ui| { self.ctrl_cell(ui, reading.ctrl, Ctrl::WR); });
                            row.col(|ui| { self.ctrl_cell(ui, reading.ctrl, Ctrl::MREQ); });
                            row.col(|ui| { self.ctrl_cell(ui, reading.ctrl, Ctrl::IORQ); });
                            row.col(|ui| { self.ctrl_cell(ui, reading.ctrl, Ctrl::RFSH); });
                            row.col(|ui| { self.signal_cell(ui, reading.m1); });
                            row.col(|ui| { self.signal_cell(ui, reading.busrq); });
                            row.col(|ui| { self.signal_cell(ui, reading.busak); });
                            row.col(|ui| { self.signal_cell(ui, reading.wait); });
                            row.col(|ui| { self.signal_cell(ui, reading.halt); });
                            row.col(|ui| { self.signal_cell(ui, reading.int); });
                            row.col(|ui| { self.signal_cell(ui, reading.nmi); });
                            row.col(|ui| { self.signal_cell(ui, reading.reset); });
                        });
                    }
                });

        })

    }

}
