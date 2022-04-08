use std::{cmp::min, rc::Rc};
use eframe::egui::*;

use librespectrum::devs::mem::Memory;

use super::{SubWindow, draw_window, cursor_color};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Cursor {
    Address(usize),
    Memory(usize, usize),
    Text(usize, usize),
}

pub struct MemoryWindow {
    memory: Rc<dyn Memory>,
    cols: usize,
    rows: usize,
    addr: u16,
    cursor: Cursor,
}

impl MemoryWindow {

    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Self { memory, cols: 8, rows: 16, addr: 0, cursor: Cursor::Address(0) }
    }

    fn handle_keyboard(&mut self, input: &InputState) {

        let (Cursor::Address(row) | Cursor::Memory(_, row) | Cursor::Text(_, row)) = self.cursor;

        if input.key_pressed(Key::ArrowUp) {
            self.cursor = if input.modifiers.alt {
                match self.cursor {
                    Cursor::Address(_) => Cursor::Address(0),
                    Cursor::Memory(col, _) => Cursor::Memory(col, 0),
                    Cursor::Text(col, _) => Cursor::Text(col, 0)
                }
            } else {
                if row == 0 { self.addr = self.addr.overflowing_sub(self.cols as u16).0; }
                match self.cursor {
                    Cursor::Address(row) => Cursor::Address(row.saturating_sub(1)),
                    Cursor::Memory(col, row) => Cursor::Memory(col, row.saturating_sub(1)),
                    Cursor::Text(col, row) => Cursor::Text(col, row.saturating_sub(1))
                }
            }
        }

        if input.key_pressed(Key::ArrowDown) {
            self.cursor = if input.modifiers.alt {
                match self.cursor {
                    Cursor::Address(_) => Cursor::Address(self.rows - 1),
                    Cursor::Memory(col, _) => Cursor::Memory(col, self.rows - 1),
                    Cursor::Text(col, _) => Cursor::Text(col, self.rows - 1)
                }
            } else {
                if row == self.rows - 1 { self.addr = self.addr.overflowing_add(self.cols as u16).0; }
                match self.cursor {
                    Cursor::Address(row) => Cursor::Address(min(self.rows - 1, row + 1)),
                    Cursor::Memory(col, row) => Cursor::Memory(col, min(self.rows - 1, row + 1)),
                    Cursor::Text(col, row) => Cursor::Text(col, min(self.rows - 1, row + 1))
                }
            }
        }

        if input.key_pressed(Key::ArrowLeft) {
            self.cursor = if input.modifiers.alt {
                match self.cursor {
                    Cursor::Memory(col, row) if col == 0 => Cursor::Address(row),
                    Cursor::Memory(_, row) => Cursor::Memory(0, row),
                    Cursor::Text(col, row) if col == 0 => Cursor::Memory(0, row),
                    Cursor::Text(_, row) => Cursor::Text(0, row),
                    _ => self.cursor
                }
            } else {
                match self.cursor {
                    Cursor::Memory(col, row) if col == 0 => Cursor::Address(row),
                    Cursor::Memory(col, row) => Cursor::Memory(col - 1, row),
                    Cursor::Text(col, row) if col == 0 => Cursor::Memory(self.cols - 1, row),
                    Cursor::Text(col, row) => Cursor::Text(col - 1, row),
                    _ => self.cursor
                }
            }
        }

        if input.key_pressed(Key::ArrowRight) {
            self.cursor = if input.modifiers.alt {
                match self.cursor {
                    Cursor::Address(row) => Cursor::Memory(self.cols - 1, row),
                    Cursor::Memory(col, row) if col == self.cols - 1 => Cursor::Text(self.cols - 1, row),
                    Cursor::Memory(_, row) => Cursor::Memory(self.cols - 1, row),
                    Cursor::Text(_, row) => Cursor::Text(self.cols - 1, row),
                    _ => self.cursor
                }
            } else {
                match self.cursor {
                    Cursor::Address(row) => Cursor::Memory(0, row),
                    Cursor::Memory(col, row) if col == self.cols - 1 => Cursor::Text(0, row),
                    Cursor::Memory(col, row) => Cursor::Memory(col + 1, row),
                    Cursor::Text(col, row) if col < self.cols - 1 => Cursor::Text(col + 1, row),
                    _ => self.cursor
                }
            }
        }

        if input.key_pressed(Key::PageUp) {
            self.addr = self.addr.overflowing_sub((self.cols * self.rows) as u16).0;
        }

        if input.key_pressed(Key::PageDown) {
            self.addr = self.addr.overflowing_add((self.cols * self.rows) as u16).0;
        }

        if input.key_pressed(Key::Home) {
            self.cursor = Cursor::Address(row);
        }

        if input.key_pressed(Key::End) {
            self.cursor = Cursor::Text(self.cols - 1, row);
        }

    }

}

impl SubWindow for MemoryWindow {

    fn name(&self) -> String { String::from("Memory") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        draw_window(self.name(), focused, ctx, |ui| {

            if focused {
                self.handle_keyboard(&ui.input());
            }

            Grid::new("memory").min_col_width(0.0).show(ui, |ui| {

                for row in 0..self.rows {

                    let row_addr = self.addr.overflowing_add((row * self.cols) as u16).0;

                    let label = Label::new(
                        RichText::new(
                            format!("{:04X}", row_addr)
                        ).background_color(
                            if self.cursor == Cursor::Address(row) {cursor_color(focused)} else {Color32::default()}
                        )
                    ).sense(Sense::click());

                    if ui.add(label).clicked() {
                        self.cursor = Cursor::Address(row);
                    }

                    ui.add(Separator::default().vertical());

                    for col in 0..self.cols {

                        let addr = row_addr.overflowing_add(col as u16).0;

                        let label = Label::new(
                            RichText::new(
                                format!("{:02X}", self.memory[addr].get())
                            ).background_color(
                                if self.cursor == Cursor::Memory(col, row) {cursor_color(focused)} else {Color32::default()}
                            ).color(
                                if self.memory.writable(addr) {Color32::BLACK} else {Color32::GRAY}
                            )
                        ).sense(Sense::click());

                        if ui.add(label).clicked() {
                            self.cursor = Cursor::Memory(col, row);
                        }

                    }

                    ui.add(Separator::default().vertical());

                    ui.horizontal(|ui| {

                        ui.spacing_mut().item_spacing.x = 0.0;

                        for col in 0..self.cols {

                            let addr = row_addr.overflowing_add(col as u16).0;

                            let byte = self.memory[addr].get();

                            let is_ascii = byte.is_ascii_alphanumeric() || byte.is_ascii_graphic()
                                || byte.is_ascii_punctuation() || byte == 0x20 /* space */;

                            let label = Label::new(
                                RichText::new(
                                    (if is_ascii {byte as char} else {'.'}).to_string()
                                ).background_color(
                                    if self.cursor == Cursor::Text(col, row) {cursor_color(focused)} else {Color32::default()}
                                ).color(
                                    if is_ascii {Color32::DARK_GRAY} else {Color32::LIGHT_GRAY}
                                )
                            ).sense(Sense::click());

                            if ui.add(label).clicked() {
                                self.cursor = Cursor::Text(col, row);
                            }

                        }

                    });

                    ui.end_row();

                }

            });

        })

    }

}
