use iced::widget::{
    button, checkbox, column, container, pick_list, row, text, text_input,
};
use iced::{Border, Color, Element, Length, Theme};

use crate::model::profile::ConnectionProfile;

#[derive(Debug, Clone)]
pub enum ConnectionMessage {
    SelectProfile(String),
    NewProfile,
    DeleteProfile,
    SetName(String),
    SetHost(String),
    SetPort(String),
    SetUsername(String),
    SetPassword(String),
    ToggleStartTls(bool),
    Connect,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub visible: bool,
    pub profiles: Vec<ConnectionProfile>,
    pub selected_index: Option<usize>,
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub use_starttls: bool,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            visible: false,
            profiles: Vec::new(),
            selected_index: None,
            name: String::new(),
            host: String::new(),
            port: "4190".to_string(),
            username: String::new(),
            password: String::new(),
            use_starttls: true,
        }
    }
}

impl ConnectionState {
    pub fn open(&mut self, profiles: Vec<ConnectionProfile>) {
        self.profiles = profiles;
        self.visible = true;
        if !self.profiles.is_empty() {
            self.select(0);
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn select(&mut self, index: usize) {
        if index < self.profiles.len() {
            let p = &self.profiles[index];
            self.selected_index = Some(index);
            self.name = p.name.clone();
            self.host = p.host.clone();
            self.port = p.port.to_string();
            self.username = p.username.clone();
            self.use_starttls = p.use_starttls;
            self.password.clear();
        }
    }

    pub fn to_profile(&self) -> ConnectionProfile {
        ConnectionProfile {
            name: if self.name.is_empty() {
                self.host.clone()
            } else {
                self.name.clone()
            },
            host: self.host.clone(),
            port: self.port.parse().unwrap_or(4190),
            username: self.username.clone(),
            use_starttls: self.use_starttls,
        }
    }

    fn profile_names(&self) -> Vec<String> {
        self.profiles.iter().map(|p| p.name.clone()).collect()
    }

    fn selected_name(&self) -> Option<String> {
        self.selected_index
            .and_then(|i| self.profiles.get(i))
            .map(|p| p.name.clone())
    }
}

pub fn view(state: &ConnectionState) -> Element<'_, ConnectionMessage> {
    let profile_names = state.profile_names();
    let selected = state.selected_name();

    let profile_row = row![
        pick_list(profile_names, selected, ConnectionMessage::SelectProfile).width(200),
        button("New").on_press(ConnectionMessage::NewProfile),
        button("Delete")
            .on_press(ConnectionMessage::DeleteProfile)
            .style(button::danger),
    ]
    .spacing(4);

    let form = column![
        labeled_input("Profile Name:", &state.name, ConnectionMessage::SetName),
        labeled_input("Host:", &state.host, ConnectionMessage::SetHost),
        labeled_input("Port:", &state.port, ConnectionMessage::SetPort),
        labeled_input("Username:", &state.username, ConnectionMessage::SetUsername),
        labeled_password("Password:", &state.password, ConnectionMessage::SetPassword),
        checkbox("Use STARTTLS", state.use_starttls).on_toggle(ConnectionMessage::ToggleStartTls),
    ]
    .spacing(6);

    let buttons = row![
        button("Connect")
            .on_press(ConnectionMessage::Connect)
            .style(button::primary),
        button("Cancel").on_press(ConnectionMessage::Cancel),
    ]
    .spacing(8);

    let dialog = container(
        column![
            text("Connect to Server").size(18),
            profile_row,
            form,
            buttons,
        ]
        .spacing(12)
        .padding(20)
        .max_width(450),
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

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> ConnectionMessage + 'a,
) -> Element<'a, ConnectionMessage> {
    row![
        text(label).width(120).size(14),
        text_input("", value).on_input(on_input).width(280),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn labeled_password<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> ConnectionMessage + 'a,
) -> Element<'a, ConnectionMessage> {
    row![
        text(label).width(120).size(14),
        text_input("", value)
            .on_input(on_input)
            .secure(true)
            .width(280),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
