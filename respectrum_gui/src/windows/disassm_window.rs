use std::{
    cmp::min,
    rc::Rc,
    pin::Pin,
    ops::{Generator, GeneratorState},
};
use eframe::{egui::*};

use librespectrum::{
    devs::mem,
    tools
};

use super::{SubWindow, draw_window, cursor_color};

pub struct DisassmWindow {
    memory: Rc<dyn mem::Memory>,
    addr: u16,
    rows: usize,
    cursor: usize,
}

impl DisassmWindow {

    pub fn new(memory: Rc<dyn mem::Memory>) -> Self {
        Self { memory, addr: 0, rows: 16, cursor: 0 }
    }

    fn handle_keyboard(&mut self, input: &InputState) {

        if input.key_pressed(Key::ArrowUp) {
            self.cursor = self.cursor.saturating_sub(1);
        }

        if input.key_pressed(Key::ArrowDown) {
            self.cursor = min(self.rows - 1, self.cursor + 1)
        }

    }

}

impl SubWindow for DisassmWindow {

    fn name(&self) -> String { String::from("Disassembler") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        let mut addr = self.addr;
        let mut disassembler = tools::disassembler(addr);
        let formatter = tools::InstructionFormatter::default();

        draw_window(self.name(), focused, ctx, |ui| {

            if focused {
                self.handle_keyboard(&ui.input());
            }

            Grid::new("memory").min_col_width(0.0).show(ui, |ui| {

                for row in 0..self.rows {

                    loop { // Start instruction decode loop

                        let result = Pin::new(&mut disassembler).resume(self.memory[addr].get());
                        addr += 1;

                        if let GeneratorState::Yielded(Some(op)) = result {

                            let label = Label::new(
                                RichText::new(
                                    formatter.format_addr(&op)
                                ).background_color(
                                    if self.cursor == row {cursor_color(focused)} else {Color32::default()}
                                ).color(Color32::BLACK)
                            ).sense(Sense::click());

                            if ui.add(label).clicked() {
                                self.cursor = row;
                            }

                            ui.add(Separator::default().vertical());
                            ui.label(formatter.format_bytes(&op));
                            ui.add(Separator::default().vertical());
                            ui.label(formatter.format_mnemonic(&op));
                            ui.end_row();

                            break; // Break instruction decode loop

                        }

                    }

                }

            });

        })

    }

}
