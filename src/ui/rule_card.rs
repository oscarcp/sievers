use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, text, text_input, toggler,
};
use iced::{Border, Color, Element, Font, Length, Theme};

use crate::app::Message;
use crate::model::enums::LogicOperator;
use crate::model::rule::SieveRule;
use crate::ui::action_row::{self, ActionMessage};
use crate::ui::condition_row::{self, ConditionMessage};
use crate::ui::icons;

#[derive(Debug, Clone)]
pub enum RuleMessage {
    SetName(String),
    SetEnabled(bool),
    SetLogic(LogicOption),
    RemoveRule,
    AddCondition,
    AddAction,
    ConditionMsg(usize, ConditionMessage),
    ActionMsg(usize, ActionMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicOption(pub LogicOperator);

impl std::fmt::Display for LogicOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            LogicOperator::AllOf => write!(f, "All of (AND)"),
            LogicOperator::AnyOf => write!(f, "Any of (OR)"),
        }
    }
}

pub const LOGIC_OPTIONS: &[LogicOption] = &[
    LogicOption(LogicOperator::AllOf),
    LogicOption(LogicOperator::AnyOf),
];

// ─── Sidebar card (compact) ────────────────────────────────────────

/// Sidebar card as a clickable button that sends `Message::SelectRule`.
pub fn sidebar_card_button<'a>(
    rule: &'a SieveRule,
    selected: bool,
    idx: usize,
) -> Element<'a, Message> {
    let name = if rule.name.is_empty() {
        "(unnamed)"
    } else {
        &rule.name
    };

    let mut content = column![].spacing(4);

    // Name (bold)
    content = content.push(
        text(name)
            .size(14)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }),
    );

    // Status + summary row
    let mut info = row![].spacing(6).align_y(iced::Alignment::Center);

    if rule.enabled {
        info = info.push(
            container(text("active").size(10).color(Color::WHITE))
                .padding([1, 6])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.7, 0.3))),
                    border: Border {
                        radius: 8.0.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }),
        );
    } else {
        info = info.push(
            container(text("disabled").size(10).style(muted_text))
                .padding([1, 6])
                .style(|theme: &Theme| {
                    let p = theme.palette();
                    container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            p.text.r, p.text.g, p.text.b, 0.08,
                        ))),
                        border: Border {
                            radius: 8.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                }),
        );
    }

    let nc = rule.conditions.len();
    let na = rule.actions.len();
    info = info.push(text(format!("{nc} cond, {na} act")).size(11).style(muted_text));

    content = content.push(info);

    button(content)
        .on_press(Message::SelectRule(idx))
        .width(Length::Fill)
        .padding(10)
        .style(move |theme: &Theme, _status| {
            let p = theme.palette();
            let border_color = if selected {
                Color::from_rgb(0.2, 0.45, 0.85)
            } else {
                Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.12)
            };
            let bg = if selected {
                Color::from_rgba(0.2, 0.45, 0.85, 0.08)
            } else {
                p.background
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.text,
                border: Border {
                    color: border_color,
                    width: if selected { 2.0 } else { 1.0 },
                    radius: 8.0.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

// ─── Detail panel sections ─────────────────────────────────────────

/// Filter Details card: name, enabled toggler, logic operator
pub fn detail_filter_info(rule: &SieveRule) -> Element<'_, RuleMessage> {
    let content = column![
        // Header
        text("Filter Details")
            .size(15)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }),
        horizontal_rule(1),
        // Filter Name
        column![
            text("Filter Name").size(11).style(muted_text),
            text_input("Filter name", &rule.name)
                .on_input(RuleMessage::SetName)
                .width(Length::Fill),
        ]
        .spacing(4),
        // Enable toggle
        row![
            column![
                text("Enable Filter").size(13),
                text("Activate this filter for incoming emails")
                    .size(11)
                    .style(muted_text),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(rule.enabled).on_toggle(RuleMessage::SetEnabled),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(12),
        horizontal_rule(1),
        // Logic operator
        column![
            text("Match Logic").size(11).style(muted_text),
            pick_list(
                LOGIC_OPTIONS,
                Some(LogicOption(rule.logic)),
                RuleMessage::SetLogic
            )
            .width(180),
        ]
        .spacing(4),
    ]
    .spacing(10);

    section_card(content)
}

/// Conditions card with "+ Add Condition" button in header
pub fn detail_conditions(rule: &SieveRule) -> Element<'_, RuleMessage> {
    let mut content = column![].spacing(6);

    // Header row
    content = content.push(
        row![
            text("Conditions")
                .size(15)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
            iced::widget::horizontal_space().width(Length::Fill),
            button(icons::icon_text(icons::ADD_CIRCLE, "Add Condition"))
                .on_press(RuleMessage::AddCondition)
                .style(button::secondary)
                .padding([3, 8]),
        ]
        .align_y(iced::Alignment::Center),
    );

    content = content.push(horizontal_rule(1));

    if rule.conditions.is_empty() {
        content = content.push(
            text("No conditions yet. Add one to start filtering.")
                .size(12)
                .style(muted_text),
        );
    } else {
        for (i, cond) in rule.conditions.iter().enumerate() {
            content = content.push(
                condition_row::view(cond, i + 1)
                    .map(move |msg| RuleMessage::ConditionMsg(i, msg)),
            );
        }
    }

    section_card(content)
}

/// Actions card with "+ Add Action" button in header
pub fn detail_actions(rule: &SieveRule) -> Element<'_, RuleMessage> {
    let mut content = column![].spacing(6);

    // Header row
    content = content.push(
        row![
            text("Actions")
                .size(15)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
            iced::widget::horizontal_space().width(Length::Fill),
            button(icons::icon_text(icons::ADD_CIRCLE, "Add Action"))
                .on_press(RuleMessage::AddAction)
                .style(button::secondary)
                .padding([3, 8]),
        ]
        .align_y(iced::Alignment::Center),
    );

    content = content.push(horizontal_rule(1));

    // Raw block display
    if let Some(raw) = &rule.raw_block {
        content = content.push(text("Unrecognized construct (raw):").size(12));
        content = content.push(
            container(text(raw).size(12))
                .padding(4)
                .width(Length::Fill)
                .style(|theme: &Theme| {
                    let p = theme.palette();
                    container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            p.text.r, p.text.g, p.text.b, 0.05,
                        ))),
                        border: Border {
                            radius: 4.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                }),
        );
    }

    if rule.actions.is_empty() && rule.raw_block.is_none() {
        content = content.push(
            text("No actions yet. Add one to define what happens.")
                .size(12)
                .style(muted_text),
        );
    } else {
        for (i, action) in rule.actions.iter().enumerate() {
            content = content.push(
                action_row::view(action, i + 1).map(move |msg| RuleMessage::ActionMsg(i, msg)),
            );
        }
    }

    section_card(content)
}

// ─── Shared helpers ────────────────────────────────────────────────

fn section_card(content: iced::widget::Column<'_, RuleMessage>) -> Element<'_, RuleMessage> {
    container(content)
        .padding(16)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let p = theme.palette();
            container::Style {
                background: Some(iced::Background::Color(p.background)),
                border: Border {
                    color: Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.12),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Theme-aware muted text style (50% opacity of the theme's text color).
fn muted_text(theme: &Theme) -> text::Style {
    let p = theme.palette();
    text::Style {
        color: Some(Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.5)),
    }
}
