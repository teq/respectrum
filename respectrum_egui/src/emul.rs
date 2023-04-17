#![feature(generators, generator_trait)]

extern crate librespectrum;

use librespectrum::{
    bus::{CpuBus, Clock, Scheduler},
    devs::{mem::{Dynamic48k, Memory}, Device, Cpu, BusLogger}
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
    let cpu = Rc::new(Cpu::new(Rc::clone(&bus), Rc::clone(&clock)));
    let mem: Rc<dyn Memory> = {
        let mem = Rc::new(Dynamic48k::new(Rc::clone(&bus), Rc::clone(&clock)));
        let mut buffer: Vec<u8> = Vec::new();
        File::open("roms/48.rom").unwrap().read_to_end(&mut buffer).unwrap();
        mem.load(0, &buffer);
        mem
    };
    let logger = Rc::new(BusLogger::new(Rc::clone(&bus), Rc::clone(&clock)));

    let scheduler =  Rc::new(RefCell::new(
        Scheduler::new(Rc::clone(&clock), vec![cpu.run(), mem.run(), logger.run()])
    ));

    let app = Box::new(EmulApp {
        windows: vec![
            (true, Box::new(CpuWindow::new(Rc::clone(&cpu), Rc::clone(&scheduler)))),
            (true, Box::new(DisassmWindow::new(Rc::clone(&cpu), Rc::clone(&mem)))),
            (true, Box::new(MemoryWindow::new(Rc::clone(&mem)))),
            (true, Box::new(BusWindow::new(Rc::clone(&logger)))),
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
