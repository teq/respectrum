use eframe::egui;

mod cpu_window;
pub use cpu_window::CpuWindow;
mod disassm_window;
pub use disassm_window::DisassmWindow;
mod memory_window;
pub use memory_window::MemoryWindow;

pub trait Window {

    /// Window name
    fn name(&self) -> &str;

    /// Window draw function
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);

}
