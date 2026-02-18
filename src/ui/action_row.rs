use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Color, Element, Length, Theme};

use crate::model::enums::ActionType;
use crate::model::rule::Action;
use crate::ui::icons;

#[derive(Debug, Clone)]
pub enum ActionMessage {
    SetActionType(ActionTypeOption),
    SetArgument(String),
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionTypeOption(pub ActionType);

impl std::fmt::Display for ActionTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_sieve())
    }
}

pub const ACTION_OPTIONS: &[ActionTypeOption] = &[
    ActionTypeOption(ActionType::Fileinto),
    ActionTypeOption(ActionType::Redirect),
    ActionTypeOption(ActionType::Reject),
    ActionTypeOption(ActionType::Discard),
    ActionTypeOption(ActionType::Keep),
    ActionTypeOption(ActionType::Stop),
    ActionTypeOption(ActionType::Setflag),
    ActionTypeOption(ActionType::Addflag),
    ActionTypeOption(ActionType::Removeflag),
];

/// View a single action with numbered heading and labeled grid layout.
pub fn view(action: &Action, number: usize) -> Element<'_, ActionMessage> {
    let action_type = ActionTypeOption(action.action_type);
    let takes_arg = action.action_type.takes_argument();

    let mut content = column![].spacing(8);

    // Header: "Action N" + trash icon
    let heading = row![
        text(format!("Action {number}"))
            .size(13)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
        iced::widget::horizontal_space().width(Length::Fill),
        button(
            text(icons::DELETE_BIN.to_string())
                .font(icons::ICON_FONT)
                .size(14)
                .color(Color::from_rgb(0.85, 0.2, 0.2))
        )
        .on_press(ActionMessage::Remove)
        .style(|_theme: &Theme, _status| button::Style {
            background: None,
            ..button::Style::default()
        })
        .padding([2, 6]),
    ]
    .align_y(iced::Alignment::Center);

    content = content.push(heading);

    // Labeled fields in a row
    let mut fields = row![].spacing(12);

    fields = fields.push(
        column![
            label_text("Action Type"),
            pick_list(ACTION_OPTIONS, Some(action_type), ActionMessage::SetActionType).width(140),
        ]
        .spacing(4),
    );

    if takes_arg {
        fields = fields.push(
            column![
                label_text("Value"),
                text_input("Folder, address...", &action.argument)
                    .on_input(ActionMessage::SetArgument)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
        );
    }

    content = content.push(fields);
    content = content.push(horizontal_rule(1));

    container(content)
        .padding([8, 0])
        .width(Length::Fill)
        .into()
}

fn label_text(label: &str) -> Element<'_, ActionMessage> {
    text(label)
        .size(11)
        .style(|theme: &Theme| {
            let p = theme.palette();
            text::Style {
                color: Some(Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.5)),
            }
        })
        .into()
}
