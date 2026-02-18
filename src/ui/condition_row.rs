use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input};
use iced::{Color, Element, Length, Theme};

use crate::model::enums::*;
use crate::model::rule::Condition;
use crate::ui::icons;

#[derive(Debug, Clone)]
pub enum ConditionMessage {
    SetTestType(ConditionTestOption),
    SetMatchType(MatchTypeOption),
    SetAddressPart(AddressPartOption),
    SetSizeComparator(SizeComparatorOption),
    SetHeaders(String),
    SetValue(String),
    Remove,
}

// Wrapper types for pick_list (need Display + PartialEq)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionTestOption(pub ConditionTest);

impl std::fmt::Display for ConditionTestOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ConditionTest::Header => write!(f, "Header"),
            ConditionTest::Address => write!(f, "Address"),
            ConditionTest::Envelope => write!(f, "Envelope"),
            ConditionTest::Size => write!(f, "Size"),
            ConditionTest::Exists => write!(f, "Exists"),
            ConditionTest::Body => write!(f, "Body"),
            other => write!(f, "{}", other.as_sieve()),
        }
    }
}

pub const TEST_OPTIONS: &[ConditionTestOption] = &[
    ConditionTestOption(ConditionTest::Header),
    ConditionTestOption(ConditionTest::Address),
    ConditionTestOption(ConditionTest::Envelope),
    ConditionTestOption(ConditionTest::Size),
    ConditionTestOption(ConditionTest::Exists),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchTypeOption(pub MatchType);

impl std::fmt::Display for MatchTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            MatchType::Is => write!(f, "is"),
            MatchType::Contains => write!(f, "contains"),
            MatchType::Matches => write!(f, "matches"),
            MatchType::Regex => write!(f, "regex"),
        }
    }
}

pub const MATCH_OPTIONS: &[MatchTypeOption] = &[
    MatchTypeOption(MatchType::Is),
    MatchTypeOption(MatchType::Contains),
    MatchTypeOption(MatchType::Matches),
    MatchTypeOption(MatchType::Regex),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressPartOption(pub AddressPartType);

impl std::fmt::Display for AddressPartOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AddressPartType::All => write!(f, "all"),
            AddressPartType::Localpart => write!(f, "localpart"),
            AddressPartType::Domain => write!(f, "domain"),
        }
    }
}

pub const ADDRESS_PART_OPTIONS: &[AddressPartOption] = &[
    AddressPartOption(AddressPartType::All),
    AddressPartOption(AddressPartType::Localpart),
    AddressPartOption(AddressPartType::Domain),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeComparatorOption(pub SizeComparator);

impl std::fmt::Display for SizeComparatorOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            SizeComparator::Over => write!(f, "over"),
            SizeComparator::Under => write!(f, "under"),
        }
    }
}

pub const SIZE_OPTIONS: &[SizeComparatorOption] = &[
    SizeComparatorOption(SizeComparator::Over),
    SizeComparatorOption(SizeComparator::Under),
];

/// View a single condition with numbered heading and labeled grid layout.
pub fn view(cond: &Condition, number: usize) -> Element<'_, ConditionMessage> {
    let test_type = ConditionTestOption(cond.test_type);
    let is_size = cond.test_type == ConditionTest::Size;
    let is_exists = cond.test_type == ConditionTest::Exists;
    let is_address = matches!(
        cond.test_type,
        ConditionTest::Address | ConditionTest::Envelope
    );

    let mut content = column![].spacing(8);

    // Header: "Condition N" + trash icon
    let heading = row![
        text(format!("Condition {number}"))
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
        .on_press(ConditionMessage::Remove)
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

    // Field (test type)
    fields = fields.push(
        column![
            label_text("Field"),
            pick_list(TEST_OPTIONS, Some(test_type), ConditionMessage::SetTestType).width(120),
        ]
        .spacing(4),
    );

    // Address part (only for address/envelope)
    if is_address {
        fields = fields.push(
            column![
                label_text("Address Part"),
                pick_list(
                    ADDRESS_PART_OPTIONS,
                    Some(AddressPartOption(cond.address_part)),
                    ConditionMessage::SetAddressPart,
                )
                .width(110),
            ]
            .spacing(4),
        );
    }

    // Header name (not for size)
    if !is_size {
        let headers = cond.header_names.join(", ");
        fields = fields.push(
            column![
                label_text("Header"),
                text_input("Header name", &headers)
                    .on_input(ConditionMessage::SetHeaders)
                    .width(140),
            ]
            .spacing(4),
        );
    }

    // Operator (match type, not for size or exists)
    if !is_size && !is_exists {
        fields = fields.push(
            column![
                label_text("Operator"),
                pick_list(
                    MATCH_OPTIONS,
                    Some(MatchTypeOption(cond.match_type)),
                    ConditionMessage::SetMatchType,
                )
                .width(110),
            ]
            .spacing(4),
        );
    }

    // Size comparator (only for size)
    if is_size {
        fields = fields.push(
            column![
                label_text("Comparator"),
                pick_list(
                    SIZE_OPTIONS,
                    Some(SizeComparatorOption(cond.size_comparator)),
                    ConditionMessage::SetSizeComparator,
                )
                .width(90),
            ]
            .spacing(4),
        );
    }

    // Value field (not for exists)
    if !is_exists {
        let value = if is_size {
            &cond.size_value
        } else {
            cond.keys.first().map(String::as_str).unwrap_or("")
        };
        fields = fields.push(
            column![
                label_text("Value"),
                text_input("Value", value)
                    .on_input(ConditionMessage::SetValue)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
        );
    }

    content = content.push(fields);

    // Separator between conditions
    content = content.push(horizontal_rule(1));

    container(content)
        .padding([8, 0])
        .width(Length::Fill)
        .into()
}

fn label_text(label: &str) -> Element<'_, ConditionMessage> {
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
