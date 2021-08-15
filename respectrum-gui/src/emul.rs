extern crate librespectrum;

use librespectrum::{bus, cpu, devs};
use eframe::{egui, epi};

struct CpuWindowState {
    open: bool,
}

impl Default for CpuWindowState {
    fn default() -> Self {
        Self {
            open: true
        }
    }
}

struct MemWindowState {
    open: bool,
}

impl Default for MemWindowState {
    fn default() -> Self {
        Self {
            open: true
        }
    }
}

#[derive(Default)]
struct AppState {
    cpu_window: CpuWindowState,
    mem_window: MemWindowState,
}

impl epi::App for AppState {

    fn name(&self) -> &str {
        "reSpectrum - ZX Spectrum emulator"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {

        let mut style: egui::style::Style = egui::style::Style::default();
        style.animation_time = 0.0;
        ctx.set_style(style);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                egui::menu::menu(ui, "Window", |ui| {
                    ui.checkbox(&mut self.cpu_window.open, "CPU");
                    ui.checkbox(&mut self.mem_window.open, "Memory");
                    ui.separator();
                    ui.label("Set layout:");
                    ui.separator();
                    ui.button("Normal");
                    ui.button("Developer");

                });
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        egui::Window::new("CPU").open(&mut self.cpu_window.open).show(ctx, |ui| {
            egui::Grid::new("cpu_regs").min_col_width(0.0).show(ui, |ui| {
                ui.label("AF:"); ui.label("0FA0h"); ui.label("AF':"); ui.label("0FE3h"); ui.end_row();
                ui.label("BC:"); ui.label("0FA0h"); ui.label("BC':"); ui.label("0FE3h"); ui.end_row();
                ui.label("DE:"); ui.label("0FA0h"); ui.label("DE':"); ui.label("0FE3h"); ui.end_row();
                ui.label("HL:"); ui.label("0FA0h"); ui.label("HL':"); ui.label("0FE3h"); ui.end_row();
            });
            ui.horizontal(|ui| {
                ui.label("C"); ui.label("N"); ui.label("P");
                ui.label("X"); ui.label("H"); ui.label("Y");
                ui.label("Z"); ui.label("S");
            });
            ui.horizontal(|ui| {
                ui.label("IM1");
                ui.label("IFF1");
                ui.label("IFF2");
            });
        });

        egui::Window::new("Memory").open(&mut self.mem_window.open).show(ctx, |ui| {
            egui::Grid::new("hexdump").min_col_width(0.0).show(ui, |ui| {
                ui.label("12"); ui.label("56"); ui.label("90"); ui.label("CD"); ui.end_row();
                ui.label("23"); ui.label("67"); ui.label("0A"); ui.label("DE"); ui.end_row();
                ui.label("34"); ui.label("78"); ui.label("AB"); ui.label("EF"); ui.end_row();
                ui.label("45"); ui.label("89"); ui.label("BC"); ui.label("F0"); ui.end_row();
            });
        });

    }

}

fn main() {

    let app = AppState::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    // let clock = bus::Clock::new();
    // let bus = bus::CpuBus::new();
    // let cpu = cpu::Cpu::new(&bus, &clock);
    // let mem = devs::mem::FlatRam::new(&bus, &clock);

    // let mut scheduler = bus::Scheduler::new(&clock);
    // scheduler.add(&cpu);
    // scheduler.add(&mem);

}
