use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Border, Color, Element, Font, Length, Theme};

use crate::net::managesieve::ScriptInfo;

#[derive(Debug, Clone)]
pub enum ScriptListMessage {
    SelectScript(String),
    ActivateScript(String),
    DeactivateScripts,
    DeleteScript(String),
}

pub fn view<'a>(scripts: &'a [ScriptInfo], selected: Option<&'a str>) -> Element<'a, ScriptListMessage> {
    let mut content = column![text("Scripts").size(14)].spacing(2).padding(4);

    if scripts.is_empty() {
        content = content.push(text("No scripts").size(12));
    }

    for script in scripts {
        let is_selected = selected == Some(script.name.as_str());

        let label = if script.active {
            format!("{} (active)", script.name)
        } else {
            script.name.clone()
        };

        let font = if script.active {
            Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }
        } else {
            Font::DEFAULT
        };

        let name = script.name.clone();
        let name2 = script.name.clone();
        let name3 = script.name.clone();

        let mut entry = column![
            button(text(label).font(font).size(13))
                .on_press(ScriptListMessage::SelectScript(name))
                .style(if is_selected {
                    button::primary
                } else {
                    button::text
                })
                .width(Length::Fill),
        ];

        // Context actions (shown for selected script)
        if is_selected {
            let mut actions = row![].spacing(2);
            if script.active {
                actions = actions.push(
                    button(text("Deactivate").size(11))
                        .on_press(ScriptListMessage::DeactivateScripts)
                        .style(button::secondary),
                );
            } else {
                actions = actions.push(
                    button(text("Activate").size(11))
                        .on_press(ScriptListMessage::ActivateScript(name2))
                        .style(button::secondary),
                );
            }
            actions = actions.push(
                button(text("Delete").size(11))
                    .on_press(ScriptListMessage::DeleteScript(name3))
                    .style(button::danger),
            );
            entry = entry.push(actions);
        }

        content = content.push(
            container(entry)
                .width(Length::Fill)
                .style(move |theme: &Theme| {
                    let palette = theme.palette();
                    container::Style {
                        border: Border {
                            color: Color::from_rgba(
                                palette.text.r,
                                palette.text.g,
                                palette.text.b,
                                0.1,
                            ),
                            width: if is_selected { 1.0 } else { 0.0 },
                            radius: 4.0.into(),
                        },
                        ..container::Style::default()
                    }
                }),
        );
    }

    container(
        scrollable(content)
            .height(Length::Fill)
            .width(Length::Fill),
    )
    .width(200)
    .height(Length::Fill)
    .into()
}
