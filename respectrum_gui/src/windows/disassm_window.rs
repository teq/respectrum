use eframe::egui::*;
use std::{cell::Cell, ops::Index, rc::Rc};

use super::SubWindow;

pub struct DisassmWindow {
    mem_state: Rc<dyn Index<u16, Output = Cell<u8>>>,
    addr: u16,
}

impl DisassmWindow {

    pub fn new(mem_state: Rc<dyn Index<u16, Output = Cell<u8>>>) -> Self {
        Self { mem_state, addr: 0 }
    }

}

impl SubWindow for DisassmWindow {

    fn name(&self) -> &str { "Disassembler" }

    fn show(&mut self, ctx: &Context, open: &mut bool) {

        Window::new(self.name()).resizable(false).open(open).show(ctx, |ui| {

        });

    }

}
