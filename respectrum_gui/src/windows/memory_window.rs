use eframe::egui;
use std::{cell::Cell, ops::Index, rc::Rc};

use super::Window;

pub struct MemoryWindow {
    mem_state: Rc<dyn Index<u16, Output = Cell<u8>>>,
    cols: usize,
    rows: usize,
    addr: u16,
    cursor: Option<(Option<usize>, usize)>,
}

impl MemoryWindow {

    pub fn new(mem_state: Rc<dyn Index<u16, Output = Cell<u8>>>) -> Self {
        Self {
            mem_state: mem_state.clone(),
            cols: 8, rows: 8, addr: 0, cursor: None
        }
    }

}

impl Window for MemoryWindow {

    fn name(&self) -> &str {
        "Memory"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {

        egui::Window::new(self.name()).resizable(false).open(open).show(ctx, |ui| {

            egui::Grid::new("memory").min_col_width(0.0).show(ui, |ui| {

                for row in 0..self.rows {

                    let row_addr = self.addr + (row * self.cols) as u16;

                    let label = egui::Label::new(
                        egui::RichText::new(
                            format!("{:04X}", row_addr)
                        ).background_color(
                            if self.cursor == Some((None, row)) {
                                egui::Color32::LIGHT_BLUE
                            } else {
                                Default::default()
                            }
                        )
                    ).sense(egui::Sense::click());

                    if ui.add(label).clicked() {
                        self.cursor = Some((None, row));
                    }

                    ui.label("|");

                    for col in 0..self.cols {

                        let label = egui::Label::new(
                            egui::RichText::new(
                                format!("{:02X}", self.mem_state[row_addr + col as u16].get())
                            ).background_color(
                                if self.cursor == Some((Some(col), row)) {
                                    egui::Color32::LIGHT_BLUE
                                } else {
                                    Default::default()
                                }
                            )
                        ).sense(egui::Sense::click());

                        if ui.add(label).clicked() {
                            self.cursor = Some((Some(col), row));
                        }

                    }

                    ui.label("|");

                    ui.horizontal(|ui| {

                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                        for col in 0..self.cols {

                            let byte = self.mem_state[row_addr + col as u16].get();

                            let is_ascii = byte.is_ascii_alphanumeric() || byte.is_ascii_graphic()
                                || byte.is_ascii_punctuation() || byte.is_ascii_whitespace();

                            let label = egui::Label::new(
                                egui::RichText::new(
                                    (if is_ascii {byte as char} else {'.'}).to_string()
                                ).background_color(
                                    if self.cursor == Some((Some(col), row)) {
                                        egui::Color32::LIGHT_BLUE
                                    } else {
                                        Default::default()
                                    }
                                ).color(
                                    if is_ascii {
                                        egui::Color32::DARK_GRAY
                                    } else {
                                        egui::Color32::LIGHT_GRAY
                                    }
                                )
                            ).sense(egui::Sense::click());

                            if ui.add(label).clicked() {
                                self.cursor = Some((Some(col), row));
                            }

                        }

                    });

                    ui.end_row();

                }

            });

        });

    }

}
