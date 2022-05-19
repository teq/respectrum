use iced::alignment::{self, Alignment};
use iced::executor;
use iced::keyboard;
use iced::pure::widget::pane_grid::{self, PaneGrid};
use iced::pure::{button, column, container, scrollable, text};
use iced::pure::{Application, Element};
use iced::{Command, Length, Settings, Subscription};
use iced_native::{event, subscription, Event};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Resized(pane_grid::ResizeEvent),
    Close(pane_grid::Pane),
    CloseFocused,
}

impl Application for Example {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (panes, _) = pane_grid::State::new(Pane::new(0));
        (
            Example {
                panes,
                panes_created: 1,
                focus: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("reSpectrum - ZX Spectrum emulator")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Split(axis, pane) => {
                let result = self.panes.split(
                    axis,
                    &pane,
                    Pane::new(self.panes_created),
                );

                if let Some((pane, _)) = result {
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(
                        axis,
                        &pane,
                        Pane::new(self.panes_created),
                    );

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) =
                        self.panes.adjacent(&pane, direction)
                    {
                        self.focus = Some(adjacent);
                    }
                }
            }
            Message::Clicked(pane) => {
                self.focus = Some(pane);
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    self.focus = Some(sibling);
                }
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some((_, sibling)) = self.panes.close(&pane) {
                        self.focus = Some(sibling);
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    modifiers,
                    key_code,
                }) if modifiers.command() => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<Message> {

        let pane_grid = PaneGrid::new(&self.panes, |id, pane| {

            let is_focused = self.focus == Some(id);

            let title = text(format!("Pane {}", pane.id.to_string())).size(16);

            let title_bar = pane_grid::TitleBar::new(title)
                .padding(5)
                .style(if is_focused {style::TitleBar::Focused} else {style::TitleBar::Active});

            pane_grid::Content::new(view_content(id, self.panes.len()))
            .title_bar(title_bar)
            .style(if is_focused {style::Pane::Focused} else {style::Pane::Active})

        })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::Window::Active)
        .spacing(6)
        .on_click(Message::Clicked)
        .on_resize(10, Message::Resized);

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Window::Active)
            .padding(6)
            .into()

    }

}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
}

struct Pane {
    id: usize,
}

impl Pane {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
) -> Element<'a, Message> {

    let button = |label, message, style| {
        button(
            text(label)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .size(16),
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(message)
        .style(style)
    };

    let mut controls = column()
        .spacing(5)
        .max_width(150)
        .push(button(
            "Split horizontally",
            Message::Split(pane_grid::Axis::Horizontal, pane),
            style::Button::Primary,
        ))
        .push(button(
            "Split vertically",
            Message::Split(pane_grid::Axis::Vertical, pane),
            style::Button::Primary,
        ));

    if total_panes > 1 {
        controls = controls.push(button(
            "Close",
            Message::Close(pane),
            style::Button::Destructive,
        ));
    }

    let content = column()
        .width(Length::Fill)
        .spacing(10)
        .align_items(Alignment::Center)
        .push(controls);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into()

}

mod style {

    use iced::{button, container, pane_grid, Background, Color};

    macro_rules! mkcolor {
        ($r: expr, $g: expr, $b: expr) => {
            Color::from_rgb(
                $r as f32 / 255.0,
                $g as f32 / 255.0,
                $b as f32 / 255.0,
            )
        }
    }

    const BASE03:  Color = mkcolor!(0x00, 0x2b, 0x36);
    const BASE02:  Color = mkcolor!(0x07, 0x36, 0x42);
    const BASE01:  Color = mkcolor!(0x58, 0x6e, 0x75);
    const BASE00:  Color = mkcolor!(0x65, 0x7b, 0x83);
    const BASE0:   Color = mkcolor!(0x83, 0x94, 0x96);
    const BASE1:   Color = mkcolor!(0x93, 0xa1, 0xa1);
    const BASE2:   Color = mkcolor!(0xee, 0xe8, 0xd5);
    const BASE3:   Color = mkcolor!(0xfd, 0xf6, 0xe3);
    const YELLOW:  Color = mkcolor!(0xb5, 0x89, 0x00);
    const ORANGE:  Color = mkcolor!(0xcb, 0x4b, 0x16);
    const RED:     Color = mkcolor!(0xdc, 0x32, 0x2f);
    const MAGENTA: Color = mkcolor!(0xd3, 0x36, 0x82);
    const VIOLET:  Color = mkcolor!(0x6c, 0x71, 0xc4);
    const BLUE:    Color = mkcolor!(0x26, 0x8b, 0xd2);
    const CYAN:    Color = mkcolor!(0x2a, 0xa1, 0x98);
    const GREEN:   Color = mkcolor!(0x85, 0x99, 0x00);

    pub enum Window {
        Active
    }

    impl container::StyleSheet for Window {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: Some(BASE01),
                background: Some(BASE3.into()),
                ..Default::default()
            }
        }
    }

    impl pane_grid::StyleSheet for Window {
        fn picked_split(&self) -> Option<pane_grid::Line> {
            Some(pane_grid::Line {
                color: BASE1,
                width: 2.0,
            })
        }

        fn hovered_split(&self) -> Option<pane_grid::Line> {
            Some(pane_grid::Line {
                color: BASE2,
                width: 2.0,
            })
        }
    }

    pub enum TitleBar {
        Active,
        Focused,
    }

    impl container::StyleSheet for TitleBar {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: Some(BASE3),
                background: Some(match self {
                    TitleBar::Active => BASE1,
                    TitleBar::Focused => BASE0,
                }.into()),
                ..Default::default()
            }
        }
    }

    pub enum Pane {
        Active,
        Focused,
    }

    impl container::StyleSheet for Pane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(BASE2)),
                ..Default::default()
            }
        }
    }

    pub enum Button {
        Primary,
        Destructive,
    }

    impl button::StyleSheet for Button {

        fn active(&self) -> button::Style {

            let (bg_color, text_color) = match self {
                Button::Primary => (Some(BASE1), BASE2),
                Button::Destructive => {(None, RED)}
            };

            button::Style {
                text_color,
                background: bg_color.map(Background::Color),
                ..button::Style::default()
            }

        }

        fn hovered(&self) -> button::Style {

            let active = self.active();

            let bg_color = match self {
                Button::Primary => Some(BASE0),
                Button::Destructive => Some(Color { a: 0.2, ..active.text_color }),
            };

            button::Style {
                background: bg_color.map(Background::Color),
                ..active
            }

        }

    }

}
