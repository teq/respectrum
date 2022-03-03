extern crate librespectrum;

use librespectrum::{bus, cpu, devs};
use eframe::{egui, epi};
use std::{
    rc::Rc,
    vec::Vec,
    fs::File,
    io::{prelude::*, SeekFrom}, borrow::BorrowMut,
};

mod windows;
use windows::{Window, CpuWindow, DisassmWindow, MemoryWindow};

struct EmulApp {
    windows: Vec<(bool, Box<dyn Window>)>
}

impl epi::App for EmulApp {

    fn name(&self) -> &str {
        "reSpectrum - ZX Spectrum emulator"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {

        // let mut style: egui::Style = Default::default();
        // style.visuals.override_text_color = Some(egui::Color32::RED);
        // ctx.set_style(style);

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

        for (open, window) in &mut self.windows {
            window.show(ctx, open);
        }

    }

}

fn main() {

    let bus: Rc<bus::CpuBus> = Default::default();
    let clock: Rc<bus::Clock> = Default::default();
    let cpu_state: Rc<cpu::CpuState> = Default::default();

    let cpu = Rc::new(cpu::Cpu::new(&bus, &clock, &cpu_state));
    let mem = Rc::new(devs::mem::Dynamic48k::new(&bus, &clock));

    let mut file = File::open("roms/48.rom").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    mem.load(0, &buffer);

    let app = EmulApp {
        windows: vec![
            (true, Box::new(CpuWindow::new(cpu_state))),
            (false, Box::new(DisassmWindow::new())),
            (true, Box::new(MemoryWindow::new(mem))),
        ]
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    let mut scheduler = bus::Scheduler::new(clock);
    scheduler.add(cpu.run());
    scheduler.add(mem.run());

}
