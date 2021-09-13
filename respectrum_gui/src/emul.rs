extern crate librespectrum;

use librespectrum::{bus, cpu, devs};
use eframe::{egui, epi};
use std::rc::Rc;

mod windows;
use windows::{Window, CpuWindow, DisassmWindow, MemoryWindow};

struct EmulWindow {
    cpu_window: CpuWindow,
    disassm_window: DisassmWindow,
    mem_window: MemoryWindow,
}

impl epi::App for EmulWindow {

    fn name(&self) -> &str {
        "reSpectrum - ZX Spectrum emulator"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {

        // let mut style: egui::Style = Default::default();
        // style.visuals .override_text_color = Some(egui::Color32::RED);
        // ctx.set_style(style);

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

        self.cpu_window.update(ctx);
        self.disassm_window.update(ctx);
        self.mem_window.update(ctx);

    }

}

fn main() {

    let bus = Rc::new(bus::CpuBus::new());
    let clock = Rc::new(bus::Clock::new());
    let cpu_state: Rc<cpu::CpuState> = Default::default();
    let cpu = cpu::Cpu::new(Rc::clone(&bus), Rc::clone(&clock), Rc::clone(&cpu_state));
    let mem = devs::mem::FlatRam::new(Rc::clone(&bus), Rc::clone(&clock));

    let app = EmulWindow {
        cpu_window: CpuWindow {
            open: true,
            cpu_state: Rc::clone(&cpu_state),
        },
        disassm_window: Default::default(),
        mem_window: Default::default(),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    let mut scheduler = bus::Scheduler::new(clock);
    scheduler.add(&cpu);
    scheduler.add(&mem);

}
