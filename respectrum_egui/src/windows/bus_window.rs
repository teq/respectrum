use egui::*;
use egui_extras::{Size, TableBuilder};
use librespectrum::devs::BusLogger;
use std::{rc::Rc};

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
                    header.col(|ui| { ui.label("M1"); });
                    header.col(|ui| { ui.label("BUSRQ"); });
                    header.col(|ui| { ui.label("BUSAK"); });
                    header.col(|ui| { ui.label("WAIT"); });
                    header.col(|ui| { ui.label("HALT"); });
                    header.col(|ui| { ui.label("RFSH"); });
                    header.col(|ui| { ui.label("INT"); });
                    header.col(|ui| { ui.label("NMI"); });
                    header.col(|ui| { ui.label("RESET"); });
                })
                .body(|body| {
                    body.rows(text_height, 10, |row_index, mut row| {
                        row.col(|ui| { ui.label(format!("{:x}", row_index)); });
                        row.col(|ui| { ui.label("0000"); });
                        row.col(|ui| { ui.label("00"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                        row.col(|ui| { ui.label("0"); });
                    });
                });

            // ui.vertical(|ui| {
            // });

        })

    }

}