use std::{
    cell::RefCell, cmp::min, ops::{Coroutine, CoroutineState}, pin::Pin, rc::Rc
};

use egui::*;

use librespectrum::{
    core::Scheduler, cpu::decoder::disassembler, devs::{BreakCondition, BreakpointManager, Cpu, mem::Memory}
};

use super::{SubWindow, draw_window, cursor_color};

/// Maximum bytes to process for each disassembled line
const LINE_BYTES: usize = 4;

pub struct DisassmWindow<'a> {
    scheduler: Rc<RefCell<Scheduler<'a>>>,
    cpu: Rc<Cpu>,
    memory: Rc<dyn Memory>,
    breakpoint_manager: Rc<BreakpointManager>,
    addr: u16,
    rows: usize,
    cursor: usize,
}

impl<'a> DisassmWindow<'a> {

    pub fn new(scheduler: &Rc<RefCell<Scheduler<'a>>>, cpu: &Rc<Cpu>, memory: &Rc<dyn Memory>, breakpoint_manager: &Rc<BreakpointManager>) -> Self {
        Self {
            scheduler: Rc::clone(scheduler),
            cpu: Rc::clone(cpu),
            memory: Rc::clone(memory),
            breakpoint_manager: Rc::clone(breakpoint_manager),
            addr: 0,
            rows: 24,
            cursor: 0
        }
    }

    fn prev_instr(&self, addr: u16) -> u16 {
        let mut ptr = addr.wrapping_sub((LINE_BYTES * 2) as u16);
        let mut disasm = disassembler(ptr, LINE_BYTES);
        let mut prev = addr;
        loop {
            let byte = self.memory.read(ptr);
            ptr = ptr.wrapping_add(1);
            if let CoroutineState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
                if (line.address.wrapping_sub(addr) as i16) >= 0 {
                    return prev;
                }
                prev = line.address;
            }
        }
    }

    fn next_instr(&self, addr: u16) -> u16 {
        let mut ptr = addr;
        let mut disasm = disassembler(ptr, LINE_BYTES);
        loop {
            let byte = self.memory.read(ptr);
            ptr = ptr.wrapping_add(1);
            if let CoroutineState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
                if (line.address.wrapping_sub(addr) as i16) > 0 {
                    return line.address;
                }
            }
        }
    }

    fn prev_page(&self) -> u16 {
        let mut prev = self.addr;
        for _ in 0..self.rows {
            prev = self.prev_instr(prev);
        }
        prev
    }

    fn next_page(&self) -> u16 {
        let mut next = self.addr;
        for _ in 0..self.rows {
            next = self.next_instr(next);
        }
        next
    }

    fn cursor_addr(&self) -> u16 {
        let mut addr = self.addr;
        for _ in 0..self.cursor {
            addr = self.next_instr(addr);
        }
        addr
    }

    fn follow_pc(&mut self) {
        let pc = self.cpu.pc.value().get();
        if pc < self.addr || pc >= self.next_page() {
            self.addr = pc;
            self.cursor = 0;
        }
    }

    fn handle_keyboard(&mut self, input: &InputState) {

        if input.key_pressed(Key::Enter) {
            // Advance to instruction at cursor
            let id = self.breakpoint_manager.add(BreakCondition::BeforeOpcodeRead(Some(self.cursor_addr())), true);
            while self.scheduler.borrow_mut().run(100) != Some(id) {}
        }

        if input.key_pressed(Key::Space) {
            // Advance to next instruction
            let id = self.breakpoint_manager.add(BreakCondition::BeforeOpcodeRead(None), true);
            while self.scheduler.borrow_mut().run(100) != Some(id) {}
            self.follow_pc();
        }

        if input.key_pressed(Key::ArrowUp) {
            self.cursor = if input.modifiers.alt {0} else {
                if self.cursor == 0 { self.addr = self.prev_instr(self.addr); }
                self.cursor.saturating_sub(1)
            };
        }

        if input.key_pressed(Key::ArrowDown) {
            self.cursor = if input.modifiers.alt {self.rows - 1} else {
                if self.cursor == self.rows - 1 { self.addr = self.next_instr(self.addr); }
                min(self.rows - 1, self.cursor + 1)
            };
        }

        if input.key_pressed(Key::PageUp) {
            self.addr = self.prev_page();
        }

        if input.key_pressed(Key::PageDown) {
            self.addr = self.next_page();
        }

    }

}

impl<'a> SubWindow for DisassmWindow<'a> {

    fn name(&self) -> String { String::from("Disassembler") }

    fn show(&mut self, ctx: &Context, focused: bool) -> Response {

        draw_window(self.name(), focused, ctx, |ui| {

            if focused {
                self.handle_keyboard(&ui.input());
            }

            Grid::new("memory").min_col_width(0.0).show(ui, |ui| {

                let mut disasm = disassembler(self.addr, LINE_BYTES);
                let mut ptr = self.addr;
                let pc = self.cpu.pc.value().get();

                for row in 0..self.rows {

                    // Feed bytes until line is disassembled
                    let line = loop {
                        let byte = self.memory.read(ptr);
                        ptr = ptr.wrapping_add(1);
                        if let CoroutineState::Yielded(Some(line)) = Pin::new(&mut disasm).resume(byte) {
                            break line;
                        }
                    };

                    let line_color = if line.address <= pc && pc < line.address + line.bytes.len() as u16 {
                        Color32::RED
                    } else {
                        Color32::BLACK
                    };

                    // Print address
                    if ui.add(
                        Label::new(
                            RichText::new(format!("{:0>4X}", line.address))
                                .background_color(if self.cursor == row {cursor_color(focused)} else {Color32::default()})
                                .color(line_color)
                        ).sense(Sense::click())
                    ).clicked() {
                        self.cursor = row;
                    }

                    // Print bytes
                    ui.add(Separator::default().vertical());
                    ui.add(Label::new(
                        RichText::new(format!(
                            "{:<bytes$}",
                            line.bytes.iter().map(|byte| format!("{:0>2X}", byte)).collect::<Vec<String>>().join(" "),
                            bytes = LINE_BYTES * 3 - 1
                        )).color(line_color)
                    ));

                    // Print mnemonic (if any)
                    ui.add(Separator::default().vertical());
                    if let Some(instr) = line.instruction {
                        ui.add(Label::new(
                            RichText::new(format!("{:<16}", instr.format_mnemonic())).color(line_color)
                        ));
                    } else {
                        ui.label("...");
                    }

                    ui.end_row();

                }

            });

        })

    }

}
