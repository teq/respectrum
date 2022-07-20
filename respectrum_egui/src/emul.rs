#![feature(generators, generator_trait)]

extern crate librespectrum;

use librespectrum::{
    bus::{CpuBus, Clock, Scheduler},
    devs::{mem::Dynamic48k, Device, Cpu}
};

use std::{
    rc::Rc,
    vec::Vec,
    fs::File,
    io::Read,
    ops::Deref,
    cell::RefCell,
};

mod windows;
use windows::{SubWindow, CpuWindow, DisassmWindow, MemoryWindow, BusWindow};

struct EmulApp<'a> {
    windows: Vec<(bool, Box<dyn SubWindow + 'a>)>,
    focus: usize,
}

impl eframe::App for EmulApp<'_> {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

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
                let response = window.show(ctx, self.focus == idx);
                if response.clicked() || response.drag_started() {
                    self.focus = idx;
                }
            }
        }

    }

}

fn main() {

    let bus: Rc<CpuBus> = Default::default();
    let clock: Rc<Clock> = Default::default();
    let cpu = Rc::new(Cpu::new(bus.clone(), clock.clone()));
    let mem = Rc::new(Dynamic48k::new(bus.clone(), clock.clone()));

    let mut file = File::open("roms/48.rom").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    mem.load(0, &buffer);

    let scheduler =  Rc::new(RefCell::new(
        Scheduler::new(clock.clone(), vec![cpu.run(), mem.run()])
    ));

    let app = Box::new(EmulApp {
        windows: vec![
            (true, Box::new(CpuWindow::new(cpu.clone(), scheduler.clone()))),
            (true, Box::new(DisassmWindow::new(cpu.clone(), mem.clone()))),
            (true, Box::new(MemoryWindow::new(mem.clone()))),
            (true, Box::new(BusWindow::new())),
        ],
        focus: 0,
    });

    run_native(app);

}

#[allow(unsafe_code)]
fn run_native<'a>(app: Box<dyn eframe::App + 'a>) -> ! {

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    let static_app = unsafe {
        std::mem::transmute::<Box<dyn eframe::App + 'a>, Box<dyn eframe::App + 'static>>(app)
    };

    eframe::run_native(
        "reSpectrum - ZX Spectrum emulator",
        native_options,
        Box::new(|_| static_app),
    );

}
