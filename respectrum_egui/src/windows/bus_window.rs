use egui::*;
use egui_extras::{Size, TableBuilder};
use librespectrum::{devs::BusLogger, bus::Ctrl};
use std::rc::Rc;

use super::{SubWindow, draw_window};

pub struct BusWindow {
    logger: Rc<BusLogger>
}

impl BusWindow {
    pub fn new(logger: Rc<BusLogger>) -> Self {
        Self { logger }
    }
}

impl SubWindow for BusWindow {

    fn name(&self) -> String { String::from("Bus") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        draw_window(self.name(), focused, ctx, |ui| {

            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
                .columns(Size::initial(30.0), 16)
                .header(text_height, |mut header| {
                    header.col(|ui| { ui.label("T"); });
                    header.col(|ui| { ui.label("ADDR"); });
                    header.col(|ui| { ui.label("DATA"); });
                    header.col(|ui| { ui.label("RD"); });
                    header.col(|ui| { ui.label("WR"); });
                    header.col(|ui| { ui.label("MREQ"); });
                    header.col(|ui| { ui.label("IORQ"); });
                    header.col(|ui| { ui.label("RFSH"); });
                    header.col(|ui| { ui.label("M1"); });
                    header.col(|ui| { ui.label("BUSRQ"); });
                    header.col(|ui| { ui.label("BUSAK"); });
                    header.col(|ui| { ui.label("WAIT"); });
                    header.col(|ui| { ui.label("HALT"); });
                    header.col(|ui| { ui.label("INT"); });
                    header.col(|ui| { ui.label("NMI"); });
                    header.col(|ui| { ui.label("RESET"); });
                })
                .body(|mut body| {

                    for reading in self.logger.readings.borrow().iter_to_tail() {
                        body.row(text_height, |mut row| {
                            row.col(|ui| { ui.label(format!("{}", reading.htcyc)); });
                            row.col(|ui| { if let Some(addr) = reading.addr { ui.label(format!("{:04X}", addr)); } else { ui.colored_label(Color32::GRAY, "----"); } });
                            row.col(|ui| { if let Some(data) = reading.data { ui.label(format!("{:02X}", data)); } else { ui.colored_label(Color32::GRAY, "--"); } });
                            row.col(|ui| { if let Some(ctrl) = reading.ctrl { ui.label(if ctrl.contains(Ctrl::RD) { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(ctrl) = reading.ctrl { ui.label(if ctrl.contains(Ctrl::WR) { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(ctrl) = reading.ctrl { ui.label(if ctrl.contains(Ctrl::MREQ) { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(ctrl) = reading.ctrl { ui.label(if ctrl.contains(Ctrl::IORQ) { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(ctrl) = reading.ctrl { ui.label(if ctrl.contains(Ctrl::RFSH) { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(m1) = reading.m1 { ui.label(if m1 { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(busrq) = reading.busrq { ui.label(if busrq { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(busak) = reading.busak { ui.label(if busak { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(wait) = reading.wait { ui.label(if wait { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(halt) = reading.halt { ui.label(if halt { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(int) = reading.int { ui.label(if int { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(nmi) = reading.nmi { ui.label(if nmi { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                            row.col(|ui| { if let Some(reset) = reading.reset { ui.label(if reset { "H" } else { "L" }); } else { ui.colored_label(Color32::GRAY, "-"); } });
                        });
                    }

                });

        })

    }

}
