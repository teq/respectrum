use std::{
    cmp::min,
    rc::Rc,
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use eframe::{egui::*};

use librespectrum::{
    devs::mem::Memory,
    cpu::decoder::disassembler
};

use super::{SubWindow, draw_window, cursor_color};

/// Maximum bytes to process for each disassembled line
const LINE_BYTES: usize = 4;

pub struct DisassmWindow {
    memory: Rc<dyn Memory>,
    addr: u16,
    rows: usize,
    cursor: usize,
}

impl DisassmWindow {

    pub fn new(memory: Rc<dyn Memory>) -> Self {
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

        draw_window(self.name(), focused, ctx, |ui| {

            if focused {
                self.handle_keyboard(&ui.input());
            }

            Grid::new("memory").min_col_width(0.0).show(ui, |ui| {

                let mut disasm = disassembler(self.addr, LINE_BYTES);
                let mut memory = self.memory.iter().cycle().skip(self.addr as usize);

                for row in 0..self.rows {

                    // Feed bytes until line is disassembled
                    let line = loop {
                        if let GeneratorState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(memory.next().unwrap().get()) {
                            break line;
                        }
                    };

                    // Print address
                    let label = Label::new(
                        RichText::new(
                            format!("{:0>4X}", line.address)
                        ).background_color(
                            if self.cursor == row {cursor_color(focused)} else {Color32::default()}
                        ).color(Color32::BLACK)
                    ).sense(Sense::click());

                    if ui.add(label).clicked() {
                        self.cursor = row;
                    }

                    // Print bytes
                    ui.add(Separator::default().vertical());
                    ui.label(format!("{:<bytes$}",
                        line.bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect::<Vec<String>>().join(" "),
                        bytes = LINE_BYTES * 3 - 1
                    ));

                    // Print mnemonic (if any)
                    ui.add(Separator::default().vertical());
                    if let Some(instr) = line.instruction {
                        ui.label(format!("{:<16}", instr.format_mnemonic()));
                    } else {
                        ui.label("...");
                    }

                    ui.end_row();

                }

            });

        })

    }

}
