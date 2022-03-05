use eframe::egui::*;
use librespectrum::devs::mem::Memory;
use std::rc::Rc;

use super::SubWindow;

pub struct DisassmWindow {
    memory: Rc<dyn Memory>,
    addr: u16,
}

impl DisassmWindow {

    pub fn new(mem_state: Rc<dyn Memory>) -> Self {
        Self { memory: mem_state, addr: 0 }
    }

}

impl SubWindow for DisassmWindow {

    fn name(&self) -> &str { "Disassembler" }

    fn show(&mut self, ctx: &Context, open: &mut bool) {

        Window::new(self.name()).resizable(false).open(open).show(ctx, |ui| {

        });

    }

}
