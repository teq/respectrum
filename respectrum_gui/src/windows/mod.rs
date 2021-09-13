use eframe::egui;

mod cpu_window;
pub use cpu_window::CpuWindow;
mod disassm_window;
pub use disassm_window::DisassmWindow;
mod memory_window;
pub use memory_window::MemoryWindow;

pub trait Window {
    fn update(&mut self, ctx: &egui::CtxRef);
}
