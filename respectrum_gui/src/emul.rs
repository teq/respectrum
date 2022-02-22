extern crate librespectrum;

use librespectrum::{bus, cpu, devs};
use eframe::{egui, epi};
use std::{rc::Rc, vec::Vec};

mod windows;
use windows::{Window, CpuWindow, DisassmWindow, MemoryWindow};

struct EmulApp {
    windows: Vec<(bool, Box<dyn Window>)>
}

impl epi::App for EmulApp {

    fn name(&self) -> &str {
        "reSpectrum - ZX Spectrum emulator"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {

        // let mut style: egui::Style = Default::default();
        // style.visuals.override_text_color = Some(egui::Color32::RED);
        // ctx.set_style(style);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                egui::menu::menu(ui, "Window", |ui| {
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

    let cpu = cpu::Cpu::new(&bus, &clock, &cpu_state);
    let mem = devs::mem::Dynamic48k::new(&bus, &clock);

    let app = EmulApp {
        windows: vec![
            (true, Box::new(CpuWindow { cpu_state })),
            (false, Box::new(DisassmWindow {})),
            (true, Box::new(MemoryWindow {})),
        ]
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    let mut scheduler = bus::Scheduler::new(clock);
    scheduler.add(&cpu);
    scheduler.add(&mem);

}
