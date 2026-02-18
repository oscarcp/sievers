use iced::widget::text_editor;
use iced::{Element, Font};

use crate::app::Message;

pub fn view<'a>(content: &'a text_editor::Content) -> Element<'a, Message> {
    text_editor(content)
        .placeholder("Open a file or connect to a server...")
        .on_action(Message::EditorAction)
        .font(Font::MONOSPACE)
        .into()
}
