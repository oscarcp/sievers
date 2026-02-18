use iced::widget::{row, text};
use iced::{Element, Font};

/// Remix Icon font, embedded from assets/remixicon.ttf (Apache 2.0 license).
pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../assets/remixicon.ttf");

pub const ICON_FONT: Font = Font::with_name("remixicon");

// Codepoints from Remix Icon v4.6
pub const PLUG: char = '\u{f019}';           // plug-line
pub const SHUT_DOWN: char = '\u{f126}';      // shut-down-line
pub const FOLDER_OPEN: char = '\u{ed70}';    // folder-open-line
pub const SAVE: char = '\u{f0b3}';           // save-line
pub const UPLOAD_CLOUD: char = '\u{f24e}';   // upload-cloud-line
pub const ADD_CIRCLE: char = '\u{ea11}';     // add-circle-line
pub const SERVER: char = '\u{f0e0}';         // server-line
pub const DELETE_BIN: char = '\u{ec1d}';     // delete-bin-line
pub const SUN: char = '\u{f1bc}';            // sun-line
pub const MOON: char = '\u{ef72}';           // moon-line
pub const INFORMATION: char = '\u{ee58}';    // information-line

/// Create an icon + label button content.
pub fn icon_text<'a, M: 'a>(icon: char, label: &'a str) -> Element<'a, M> {
    row![
        text(icon.to_string()).font(ICON_FONT).size(16),
        text(label).size(14),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Create a standalone icon element.
pub fn icon<'a, M: 'a>(codepoint: char, size: u16) -> Element<'a, M> {
    text(codepoint.to_string())
        .font(ICON_FONT)
        .size(size)
        .into()
}
