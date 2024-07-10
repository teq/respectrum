use std::{
    io::{self, BufReader},
    path::PathBuf,
    fs::File,
};
use druid::{
    piet::Color,
    widget::{Button, Flex, BackgroundBrush, Label},
    WindowId, Menu, Env, AppLauncher, Data, Lens,
    PlatformError, Widget, WidgetExt, WindowDesc,
};
use clap::Parser;

mod models;
mod palette;
mod widgets;

use crate::{
    models::{VideoMode, FrameBuffer},
    palette::{BlendingMode, ZXColor},
    widgets::{ZStack, FrameBufferView},
};

#[derive(Parser, Debug)]
#[clap(name = "Color Clash", version, author)]
#[clap(about = "ZX Spectrum graphics editor")]
#[clap(long_about = None)]
struct Args {

    /// File to open
    #[clap(short, long, value_name = "FILE")]
    open: Option<PathBuf>,

    /// Read image data from stdin
    #[clap(short, long)]
    stdin: bool,

}

#[derive(Clone, Lens, Data)]
pub struct AppState {
    frame_buffer: FrameBuffer,
    blending_mode: BlendingMode,
}

fn main() -> Result<(), PlatformError> {

    let args = Args::parse();

    let frame_buffer: FrameBuffer = if args.stdin {
        FrameBuffer::load(Box::new(BufReader::new(io::stdin())))
    } else if let Some(filename) = args.open {
        FrameBuffer::load(Box::new(BufReader::new(File::open(filename).unwrap())))
    } else {
        FrameBuffer::new_for(VideoMode::STD8X8)
    };

    let state = AppState {
        frame_buffer,
        blending_mode: Default::default(),
    };

    let main_window = WindowDesc::new(ui_builder())
        .menu(make_menu)
        .title("Color Clash");

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)

}

fn ui_builder() -> impl Widget<AppState> {

    fn color_label<T: Data>(text: &str, color: Color) -> impl Widget<T> {
        Label::new(text)
            // .with_text_color(Color::)
            .center()
            .fix_size(32.0, 32.0)
            .background(BackgroundBrush::Color(color))
    }

    let mut dim_colors = Flex::column();
    let mut bri_colors = Flex::column();
    for color in ZXColor::palette() {
        let [r, g, b] = color.rgb_dim();
        dim_colors.add_child(color_label("", Color::rgb8(r, g, b)));
        let [r, g, b] = color.rgb_bright();
        bri_colors.add_child(color_label("", Color::rgb8(r, g, b)));
    }

    ZStack::new()
        .with_child(
            FrameBufferView::new()
                .lens(AppState::frame_buffer)
                .scroll().center().expand()
        )
        .with_child(
            Flex::column()
                .with_child(Button::new("btn1"))
                .with_child(Button::new("btn2"))
                .padding(10.0)
                .background(BackgroundBrush::Color(Color::from_rgba32_u32(0x00000060)))
                .rounded(5.0)
                .padding(10.0)
                .align_left()
        )
        .with_child(
            Flex::row()
                .with_child(dim_colors)
                .with_child(bri_colors)
                .padding(10.0)
                .background(BackgroundBrush::Color(Color::from_rgba32_u32(0x00000060)))
                .rounded(5.0)
                .padding(10.0)
                .align_right()
        )
        // .debug_paint_layout()
        // .debug_invalidation()

}

fn make_menu(_: Option<WindowId>, _state: &AppState, _: &Env) -> Menu<AppState> {
    let mut base = Menu::empty();
    #[cfg(target_os = "macos")]
    {
        base = druid::platform_menus::mac::menu_bar();
    }
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "openbsd"))]
    {
        base = base.entry(druid::platform_menus::win::file::default());
    }
    // if state.menu_count != 0 {
    //     let mut custom = Menu::new(LocalizedString::new("Custom"));

    //     for i in 1..=state.menu_count {
    //         custom = custom.entry(
    //             MenuItem::new(
    //                 LocalizedString::new("hello-counter")
    //                     .with_arg("count", move |_: &State, _| i.into()),
    //             )
    //             .on_activate(move |_ctx, data, _env| data.selected = i)
    //             .enabled_if(move |_data, _env| i % 3 != 0)
    //             .selected_if(move |data, _env| i == data.selected),
    //         );
    //     }
    //     base = base.entry(custom);
    // }
    // base.rebuild_on(|old_data, data, _env| old_data.menu_count != data.menu_count)
    base
}
