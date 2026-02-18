use iced::widget::{button, container, horizontal_space, row, text};
use iced::{Border, Color, Element, Font, Length, Theme};

use crate::app::Message;
use crate::ui::icons;

pub fn view<'a>(connected: bool, dark_mode: bool) -> Element<'a, Message> {
    let (connect_icon, connect_label) = if connected {
        (icons::SHUT_DOWN, "Disconnect")
    } else {
        (icons::PLUG, "Connect")
    };

    let theme_icon = if dark_mode { icons::SUN } else { icons::MOON };
    let theme_label = if dark_mode { "Light" } else { "Dark" };

    let branding = row![
        text("SIEVE").size(20).font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        }),
        text("RT").size(20),
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center);

    let tb = row![
        branding,
        horizontal_space().width(24),
        toolbar_button(connect_icon, connect_label, Message::Connect),
        horizontal_space().width(12),
        toolbar_button(icons::FOLDER_OPEN, "Open", Message::OpenFile),
        toolbar_button(icons::SAVE, "Save", Message::SaveFile),
        toolbar_button(icons::UPLOAD_CLOUD, "Upload", Message::Upload),
        horizontal_space().width(Length::Fill),
        toolbar_button(theme_icon, theme_label, Message::ToggleTheme),
    ]
    .spacing(4)
    .padding(6)
    .align_y(iced::Alignment::Center);

    container(tb)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.palette();
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    palette.text.r,
                    palette.text.g,
                    palette.text.b,
                    0.04,
                ))),
                border: Border {
                    color: Color::from_rgba(
                        palette.text.r,
                        palette.text.g,
                        palette.text.b,
                        0.1,
                    ),
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

fn toolbar_button(icon: char, label: &str, msg: Message) -> iced::widget::Button<'_, Message> {
    button(icons::icon_text(icon, label))
        .on_press(msg)
        .style(|theme: &Theme, status| {
            let palette = theme.palette();
            let mut style = button::Style {
                background: Some(iced::Background::Color(palette.background)),
                text_color: palette.text,
                border: Border {
                    color: Color::from_rgba(
                        palette.text.r,
                        palette.text.g,
                        palette.text.b,
                        0.2,
                    ),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..button::Style::default()
            };
            if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                style.background = Some(iced::Background::Color(Color::from_rgba(
                    palette.text.r,
                    palette.text.g,
                    palette.text.b,
                    0.08,
                )));
            }
            style
        })
        .padding([4, 10])
}
