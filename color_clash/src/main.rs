use druid::{
    widget::{Button, Flex, Label, Split, Image, FillStrat},
    piet::{ImageBuf, InterpolationMode, ImageFormat},
    AppLauncher, Data, Lens, PlatformError, Widget, WidgetExt, WindowDesc
};

mod types;
mod widgets;

use crate::{
    types::VideoMode,
    widgets::{FrameBuffer, FrameBufferView}
};

#[derive(Clone, Lens, Data)]
pub struct AppState {
    counter: u8,
    frame: FrameBuffer,
}

fn main() -> Result<(), PlatformError> {
    let state = AppState {
        counter: 0,
        frame: FrameBuffer::new_for(VideoMode::STD8X8)
    };
    let main_window = WindowDesc::new(ui_builder());
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
}

fn ui_builder() -> impl Widget<AppState> {

    Split::columns(

        Flex::column()
            .with_child(
                FrameBufferView {}
            )
            .lens(AppState::frame),

        Flex::column()
            .with_child(
                Label::new(|state: &AppState, _env: &_| format!("Counter is {}", state.counter))
                    .padding(5.0)
                    .center()
            )
            .with_child(
                Button::new("increment")
                    .on_click(|_ctx, counter, _env| *counter += 1)
                    .lens(AppState::counter)
                    .padding(5.0)
            )

    )

}
