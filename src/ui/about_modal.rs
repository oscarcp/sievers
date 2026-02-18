use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced::{Border, Color, Element, Font, Length, Theme};

#[derive(Debug, Clone)]
pub enum AboutMessage {
    Close,
}

#[derive(Debug, Clone, Default)]
pub struct AboutState {
    pub visible: bool,
}

pub fn view(_state: &AboutState) -> Element<'_, AboutMessage> {
    let version = env!("CARGO_PKG_VERSION");
    let build_date = env!("BUILD_DATE");
    let git_commit = env!("GIT_COMMIT");

    let title = text("Sievers").size(24).font(Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    });

    let subtitle = text("SIEVE email filter manager").size(14);

    let info = column![
        info_row("Version", version),
        info_row("Build date", build_date),
        info_row("Commit", git_commit),
        info_row("Author", "Oscar Carballal Prego <oscar@lareira.digital>"),
        info_row("License", "GPL-3.0"),
    ]
    .spacing(4);

    let close_btn = button("Close")
        .on_press(AboutMessage::Close)
        .style(button::primary);

    let dialog = container(
        column![title, subtitle, horizontal_rule(1), info, horizontal_rule(1), close_btn,]
            .spacing(12)
            .padding(24)
            .max_width(420)
            .align_x(iced::Alignment::Center),
    )
    .style(|theme: &Theme| {
        let palette = theme.palette();
        container::Style {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                color: Color::from_rgba(
                    palette.text.r,
                    palette.text.g,
                    palette.text.b,
                    0.3,
                ),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..container::Style::default()
        }
    });

    // Overlay: dark semi-transparent background + centered dialog
    container(
        container(dialog)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        ..container::Style::default()
    })
    .into()
}

fn info_row<'a>(label: &'a str, value: &'a str) -> Element<'a, AboutMessage> {
    row![
        text(format!("{label}:"))
            .size(13)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(90),
        text(value).size(13),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
