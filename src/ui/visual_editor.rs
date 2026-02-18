use iced::widget::{button, column, container, scrollable, text, Space};
use iced::{Border, Color, Element, Font, Length, Theme};

use crate::app::Message;
use crate::model::rule::SieveRule;
use crate::ui::icons;
use crate::ui::rule_card;

pub fn view<'a>(rules: &'a [SieveRule], selected_rule: Option<usize>) -> Element<'a, Message> {
    let sidebar = view_sidebar(rules, selected_rule);
    let detail = view_detail(rules, selected_rule);

    iced::widget::row![sidebar, detail]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_sidebar<'a>(rules: &'a [SieveRule], selected_rule: Option<usize>) -> Element<'a, Message> {
    let mut content = column![].spacing(6).padding(8).width(Length::Fill);

    // Header
    content = content.push(
        text("Filters")
            .size(14)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }),
    );

    content = content.push(Space::with_height(4));

    // Filter cards
    for (i, rule) in rules.iter().enumerate() {
        let is_selected = selected_rule == Some(i);
        content = content.push(rule_card::sidebar_card_button(rule, is_selected, i));
    }

    content = content.push(Space::with_height(4));

    // Add filter button
    content = content.push(
        button(icons::icon_text(icons::ADD_CIRCLE, "Add Filter"))
            .on_press(Message::AddRule)
            .style(button::secondary)
            .width(Length::Fill),
    );

    let sidebar = container(scrollable(content).height(Length::Fill))
        .width(250)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let p = theme.palette();
            container::Style {
                border: Border {
                    color: Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.1),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                background: Some(iced::Background::Color(Color::from_rgba(
                    p.text.r, p.text.g, p.text.b, 0.02,
                ))),
                ..container::Style::default()
            }
        });

    sidebar.into()
}

fn view_detail<'a>(rules: &'a [SieveRule], selected_rule: Option<usize>) -> Element<'a, Message> {
    let selected = selected_rule.and_then(|idx| {
        if idx < rules.len() {
            Some((idx, &rules[idx]))
        } else {
            None
        }
    });

    let content: Element<'a, Message> = match selected {
        Some((idx, rule)) => {
            let mut detail = column![].spacing(12).padding(16).width(Length::Fill);

            // Filter Details section
            detail = detail.push(
                rule_card::detail_filter_info(rule).map(move |msg| Message::RuleMsg(idx, msg)),
            );

            // Conditions section
            detail = detail.push(
                rule_card::detail_conditions(rule).map(move |msg| Message::RuleMsg(idx, msg)),
            );

            // Actions section
            detail = detail.push(
                rule_card::detail_actions(rule).map(move |msg| Message::RuleMsg(idx, msg)),
            );

            // Remove button at the bottom
            detail = detail.push(
                button(icons::icon_text(icons::DELETE_BIN, "Remove Filter"))
                    .on_press(Message::RemoveRule(idx))
                    .style(button::danger)
                    .padding([6, 12]),
            );

            scrollable(detail).height(Length::Fill).into()
        }
        None => container(
            text("Select a filter from the sidebar to view its details.")
                .size(14)
                .style(|theme: &Theme| {
                    let p = theme.palette();
                    text::Style {
                        color: Some(Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.4)),
                    }
                }),
        )
        .center(Length::Fill)
        .into(),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
