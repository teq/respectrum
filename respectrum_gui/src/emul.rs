#![feature(generators, generator_trait)]

extern crate librespectrum;

use librespectrum::{bus, cpu, devs};
use eframe::{egui, epi};
use std::{
    rc::Rc,
    vec::Vec,
    fs::File,
    io::Read,
    ops::Deref,
};

mod windows;
use windows::{SubWindow, CpuWindow, DisassmWindow, MemoryWindow};

struct EmulApp {
    windows: Vec<(bool, Box<dyn SubWindow>)>,
    focus: Option<usize>,
}

impl epi::App for EmulApp {

    fn name(&self) -> &str { "reSpectrum - ZX Spectrum emulator" }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {

        let mut style = ctx.style().deref().clone();
        style.override_text_style = Some(egui::TextStyle::Monospace);
        ctx.set_style(style);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("Window", |ui| {
                    for (open, window) in &mut self.windows {
                        ui.checkbox(open, window.name());
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        for (idx, (open, window)) in self.windows.iter_mut().enumerate() {
            if *open {
                let response = window.show(ctx, self.focus == Some(idx));
                if response.clicked() || response.drag_started() {
                    self.focus = Some(idx);
                }
            }
        }

    }

}

fn main() {

    let bus: Rc<bus::CpuBus> = Default::default();
    let clock: Rc<bus::Clock> = Default::default();
    let cpu_state: Rc<cpu::CpuState> = Default::default();

    let cpu = Rc::new(cpu::Cpu::new(bus.clone(), clock.clone(), cpu_state.clone()));
    let mem = Rc::new(devs::mem::Dynamic48k::new(bus.clone(), clock.clone()));

    let mut file = File::open("roms/48.rom").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    mem.load(0, &buffer);

    let app = EmulApp {
        windows: vec![
            (true, Box::new(CpuWindow::new(cpu_state.clone()))),
            (true, Box::new(DisassmWindow::new(mem.clone()))),
            (true, Box::new(MemoryWindow::new(mem.clone()))),
        ],
        focus: Some(0),
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    let mut scheduler = bus::Scheduler::new(clock);
    scheduler.add(cpu.run());
    scheduler.add(mem.run());

}
