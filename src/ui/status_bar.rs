use iced::widget::{container, text};
use iced::{Element, Length};

use crate::app::Message;

pub fn view(status: &str) -> Element<'_, Message> {
    container(text(status).size(13))
        .width(Length::Fill)
        .padding([2, 8])
        .into()
}
