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

    fn prev_instr(&self) -> u16 {
        let mut ptr = self.addr.wrapping_sub((LINE_BYTES * 2) as u16);
        let mut disasm = disassembler(ptr, LINE_BYTES);
        let mut prev = self.addr;
        loop {
            let byte = self.memory.read(ptr);
            ptr = ptr.wrapping_add(1);
            if let GeneratorState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
                if (line.address.wrapping_sub(self.addr) as i16) >= 0 {
                    return prev;
                }
                prev = line.address;
            }
        }
    }

    fn next_instr(&self) -> u16 {
        let mut ptr = self.addr;
        let mut disasm = disassembler(ptr, LINE_BYTES);
        loop {
            let byte = self.memory.read(ptr);
            ptr = ptr.wrapping_add(1);
            if let GeneratorState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
                if (line.address.wrapping_sub(self.addr) as i16) > 0 {
                    return line.address;
                }
            }
        }
    }

    fn handle_keyboard(&mut self, input: &InputState) {

        if input.key_pressed(Key::ArrowUp) {
            self.cursor = if input.modifiers.alt {0} else {
                if self.cursor == 0 { self.addr = self.prev_instr(); }
                self.cursor.saturating_sub(1)
            };
        }

        if input.key_pressed(Key::ArrowDown) {
            self.cursor = if input.modifiers.alt {self.rows - 1} else {
                if self.cursor == self.rows - 1 { self.addr = self.next_instr(); }
                min(self.rows - 1, self.cursor + 1)
            };
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
                let mut ptr = self.addr;

                for row in 0..self.rows {

                    // Feed bytes until line is disassembled
                    let line = loop {
                        let byte = self.memory.read(ptr);
                        ptr = ptr.wrapping_add(1);
                        if let GeneratorState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
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
