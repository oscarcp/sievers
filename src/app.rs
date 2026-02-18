use iced::widget::{column, container, row, text_editor};
use iced::{Element, Length, Subscription, Task, Theme};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::model::enums::*;
use crate::model::profile::ConnectionProfile;
use crate::model::rule::{Action, Condition, SieveRule};
use crate::net::managesieve::{ManageSieveClient, ScriptInfo};
use crate::sieve::converter;
use crate::store::{profile_store, script_io};
use crate::ui;
use crate::ui::action_row::ActionMessage;
use crate::ui::condition_row::ConditionMessage;
use crate::ui::about_modal::{AboutMessage, AboutState};
use crate::ui::connection_modal::{ConnectionMessage, ConnectionState};
use crate::ui::rule_card::RuleMessage;
use crate::ui::script_list::ScriptListMessage;

const RAW_SYNC_DEBOUNCE_MS: u64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Visual,
    Raw,
}

pub struct Sievers {
    // Editor state
    pub editor_content: text_editor::Content,
    pub rules: Vec<SieveRule>,
    pub active_tab: Tab,

    // File
    pub current_path: Option<PathBuf>,
    pub current_script_name: Option<String>,
    pub status: String,

    // Connection
    pub connected: bool,
    pub connection: ConnectionState,
    pub server_scripts: Vec<ScriptInfo>,
    pub selected_script: Option<String>,
    client: Arc<Mutex<ManageSieveClient>>,

    // Visual editor selection
    pub selected_rule: Option<usize>,

    // Theme
    pub dark_mode: bool,

    // About
    pub about: AboutState,

    // Sync state
    syncing: bool,
    raw_dirty: bool,
    last_raw_edit: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Toolbar
    Connect,
    OpenFile,
    SaveFile,
    Upload,
    ToggleTheme,
    ShowAbout,
    AboutMsg(AboutMessage),

    // Tab
    SwitchTab(Tab),

    // Raw editor
    EditorAction(text_editor::Action),

    // Visual editor
    SelectRule(usize),
    RuleMsg(usize, RuleMessage),
    AddRule,
    RemoveRule(usize),

    // Sync
    DebounceCheck,

    // File I/O
    FileOpened(Result<(PathBuf, String), String>),
    FileSaved(Result<PathBuf, String>),

    // Connection modal
    ConnectionMsg(ConnectionMessage),

    // Server operations
    Connected(Result<Vec<ScriptInfo>, String>),
    Disconnected,
    ScriptsLoaded(Result<Vec<ScriptInfo>, String>),
    ScriptDownloaded(Result<(String, String), String>),
    ScriptUploaded(Result<String, String>),
    ScriptDeleted(Result<String, String>),
    ScriptActivated(Result<String, String>),

    // Script list
    ScriptListMsg(ScriptListMessage),
}

impl Default for Sievers {
    fn default() -> Self {
        Self {
            editor_content: text_editor::Content::new(),
            rules: Vec::new(),
            active_tab: Tab::Raw,
            current_path: None,
            current_script_name: None,
            status: "Ready".to_string(),
            connected: false,
            connection: ConnectionState::default(),
            server_scripts: Vec::new(),
            selected_script: None,
            client: Arc::new(Mutex::new(ManageSieveClient::new())),
            selected_rule: None,
            dark_mode: false,
            about: AboutState::default(),
            syncing: false,
            raw_dirty: false,
            last_raw_edit: None,
        }
    }
}

pub fn update(state: &mut Sievers, message: Message) -> Task<Message> {
    match message {
        Message::ToggleTheme => {
            state.dark_mode = !state.dark_mode;
            Task::none()
        }

        Message::ShowAbout => {
            state.about.visible = true;
            Task::none()
        }

        Message::AboutMsg(AboutMessage::Close) => {
            state.about.visible = false;
            Task::none()
        }

        Message::Connect => {
            if state.connected {
                // Disconnect
                let client = state.client.clone();
                state.connected = false;
                state.server_scripts.clear();
                state.selected_script = None;
                state.status = "Disconnected".to_string();
                return Task::perform(
                    async move {
                        client.lock().await.disconnect().await;
                    },
                    |_| Message::Disconnected,
                );
            }
            let profiles = profile_store::load_profiles();
            state.connection.open(profiles);
            Task::none()
        }

        Message::OpenFile => {
            state.status = "Opening file...".to_string();
            Task::perform(open_file_dialog(), Message::FileOpened)
        }

        Message::SaveFile => {
            if state.active_tab == Tab::Visual && !state.syncing {
                sync_visual_to_raw(state);
            }
            let text = state.editor_content.text();
            let current = state.current_path.clone();
            state.status = "Saving...".to_string();
            Task::perform(save_file_dialog(current, text), Message::FileSaved)
        }

        Message::Upload => {
            if !state.connected {
                state.status = "Not connected. Connect first.".to_string();
                return Task::none();
            }
            if state.active_tab == Tab::Visual && !state.syncing {
                sync_visual_to_raw(state);
            }
            let name = state
                .current_script_name
                .clone()
                .unwrap_or_else(|| "default".to_string());
            let content = state.editor_content.text();
            let client = state.client.clone();
            state.status = format!("Uploading {name}...");
            Task::perform(
                async move {
                    client
                        .lock()
                        .await
                        .put_script(&name, &content)
                        .await
                        .map(|_| name)
                        .map_err(|e| e.to_string())
                },
                Message::ScriptUploaded,
            )
        }

        Message::SwitchTab(tab) => {
            if tab == state.active_tab {
                return Task::none();
            }
            if tab == Tab::Visual && state.raw_dirty {
                sync_raw_to_visual(state);
            } else if tab == Tab::Raw && !state.syncing {
                sync_visual_to_raw(state);
            }
            state.active_tab = tab;
            Task::none()
        }

        Message::EditorAction(action) => {
            let is_edit = action.is_edit();
            state.editor_content.perform(action);
            if is_edit && !state.syncing {
                state.raw_dirty = true;
                state.last_raw_edit = Some(Instant::now());
            }
            Task::none()
        }

        Message::SelectRule(idx) => {
            if idx < state.rules.len() {
                state.selected_rule = Some(idx);
            }
            Task::none()
        }

        Message::RuleMsg(idx, msg) => {
            if idx < state.rules.len() {
                handle_rule_message(state, idx, msg);
                if !state.syncing {
                    sync_visual_to_raw(state);
                }
            }
            Task::none()
        }

        Message::AddRule => {
            let name = format!("New rule {}", state.rules.len() + 1);
            state.rules.push(SieveRule {
                name,
                ..Default::default()
            });
            state.selected_rule = Some(state.rules.len() - 1);
            if !state.syncing {
                sync_visual_to_raw(state);
            }
            Task::none()
        }

        Message::RemoveRule(idx) => {
            if idx < state.rules.len() {
                state.rules.remove(idx);
                // Adjust selected_rule
                if state.rules.is_empty() {
                    state.selected_rule = None;
                } else if let Some(sel) = state.selected_rule {
                    if sel >= state.rules.len() {
                        state.selected_rule = Some(state.rules.len() - 1);
                    } else if sel > idx {
                        state.selected_rule = Some(sel - 1);
                    } else if sel == idx {
                        // Keep same index if valid, otherwise go to last
                        if sel >= state.rules.len() {
                            state.selected_rule = Some(state.rules.len() - 1);
                        }
                    }
                }
                if !state.syncing {
                    sync_visual_to_raw(state);
                }
            }
            Task::none()
        }

        Message::DebounceCheck => {
            if let Some(last) = state.last_raw_edit {
                if last.elapsed().as_millis() >= RAW_SYNC_DEBOUNCE_MS as u128 && state.raw_dirty {
                    sync_raw_to_visual(state);
                }
            }
            Task::none()
        }

        Message::FileOpened(result) => {
            match result {
                Ok((path, text)) => {
                    state.editor_content = text_editor::Content::with_text(&text);
                    state.status = format!("Opened: {}", path.display());
                    state.current_path = Some(path);
                    state.raw_dirty = false;
                    state.last_raw_edit = None;
                    sync_raw_to_visual(state);
                }
                Err(e) if e != "Cancelled" => {
                    state.status = format!("Error: {e}");
                }
                _ => {}
            }
            Task::none()
        }

        Message::FileSaved(result) => {
            match result {
                Ok(path) => {
                    state.status = format!("Saved: {}", path.display());
                    state.current_path = Some(path);
                }
                Err(e) if e != "Cancelled" => {
                    state.status = format!("Error: {e}");
                }
                _ => {}
            }
            Task::none()
        }

        // --- Connection modal ---
        Message::ConnectionMsg(cmsg) => handle_connection_message(state, cmsg),

        // --- Server operation results ---
        Message::Connected(result) => {
            match result {
                Ok(scripts) => {
                    state.connected = true;
                    state.server_scripts = scripts;
                    state.connection.close();
                    state.status = "Connected".to_string();
                }
                Err(e) => {
                    state.status = format!("Connection failed: {e}");
                }
            }
            Task::none()
        }

        Message::Disconnected => Task::none(),

        Message::ScriptsLoaded(result) => {
            match result {
                Ok(scripts) => {
                    state.server_scripts = scripts;
                }
                Err(e) => {
                    state.status = format!("Error listing scripts: {e}");
                }
            }
            Task::none()
        }

        Message::ScriptDownloaded(result) => {
            match result {
                Ok((name, content)) => {
                    state.current_script_name = Some(name.clone());
                    state.editor_content = text_editor::Content::with_text(&content);
                    state.status = format!("Downloaded: {name}");
                    state.raw_dirty = false;
                    state.last_raw_edit = None;
                    sync_raw_to_visual(state);
                }
                Err(e) => {
                    state.status = format!("Error downloading: {e}");
                }
            }
            Task::none()
        }

        Message::ScriptUploaded(result) => {
            match result {
                Ok(name) => {
                    state.status = format!("Uploaded: {name}");
                    return refresh_scripts(state);
                }
                Err(e) => {
                    state.status = format!("Upload error: {e}");
                }
            }
            Task::none()
        }

        Message::ScriptDeleted(result) => {
            match result {
                Ok(name) => {
                    state.status = format!("Deleted: {name}");
                    if state.selected_script.as_deref() == Some(&name) {
                        state.selected_script = None;
                    }
                    return refresh_scripts(state);
                }
                Err(e) => {
                    state.status = format!("Delete error: {e}");
                }
            }
            Task::none()
        }

        Message::ScriptActivated(result) => {
            match result {
                Ok(name) => {
                    state.status = format!("Activated: {name}");
                    return refresh_scripts(state);
                }
                Err(e) => {
                    state.status = format!("Activate error: {e}");
                }
            }
            Task::none()
        }

        // --- Script list ---
        Message::ScriptListMsg(msg) => handle_script_list_message(state, msg),
    }
}

fn handle_connection_message(state: &mut Sievers, msg: ConnectionMessage) -> Task<Message> {
    match msg {
        ConnectionMessage::SelectProfile(name) => {
            if let Some(idx) = state.connection.profiles.iter().position(|p| p.name == name) {
                state.connection.select(idx);
            }
            Task::none()
        }
        ConnectionMessage::NewProfile => {
            state
                .connection
                .profiles
                .push(ConnectionProfile {
                    name: "New Server".to_string(),
                    ..Default::default()
                });
            let idx = state.connection.profiles.len() - 1;
            state.connection.select(idx);
            Task::none()
        }
        ConnectionMessage::DeleteProfile => {
            if let Some(idx) = state.connection.selected_index {
                state.connection.profiles.remove(idx);
                profile_store::save_profiles(&state.connection.profiles);
                if !state.connection.profiles.is_empty() {
                    state.connection.select(0);
                }
            }
            Task::none()
        }
        ConnectionMessage::SetName(s) => {
            state.connection.name = s;
            Task::none()
        }
        ConnectionMessage::SetHost(s) => {
            state.connection.host = s;
            Task::none()
        }
        ConnectionMessage::SetPort(s) => {
            state.connection.port = s;
            Task::none()
        }
        ConnectionMessage::SetUsername(s) => {
            state.connection.username = s;
            Task::none()
        }
        ConnectionMessage::SetPassword(s) => {
            state.connection.password = s;
            Task::none()
        }
        ConnectionMessage::ToggleStartTls(b) => {
            state.connection.use_starttls = b;
            Task::none()
        }
        ConnectionMessage::Connect => {
            if state.connection.host.is_empty()
                || state.connection.username.is_empty()
                || state.connection.password.is_empty()
            {
                state.status = "Host, username, and password are required.".to_string();
                return Task::none();
            }

            let profile = state.connection.to_profile();
            let password = state.connection.password.clone();

            // Save profile
            if let Some(idx) = state.connection.selected_index {
                if idx < state.connection.profiles.len() {
                    state.connection.profiles[idx] = profile.clone();
                }
            } else {
                state.connection.profiles.push(profile.clone());
            }
            profile_store::save_profiles(&state.connection.profiles);

            state.status = format!("Connecting to {}...", profile.host);
            let client = state.client.clone();

            Task::perform(
                async move {
                    let mut client = client.lock().await;
                    client.connect(&profile, &password).await.map_err(|e| e.to_string())?;
                    client.list_scripts().await.map_err(|e| e.to_string())
                },
                Message::Connected,
            )
        }
        ConnectionMessage::Cancel => {
            state.connection.close();
            Task::none()
        }
    }
}

fn handle_script_list_message(state: &mut Sievers, msg: ScriptListMessage) -> Task<Message> {
    match msg {
        ScriptListMessage::SelectScript(name) => {
            state.selected_script = Some(name.clone());
            state.current_script_name = Some(name.clone());
            let client = state.client.clone();
            state.status = format!("Downloading {name}...");
            Task::perform(
                async move {
                    let mut client = client.lock().await;
                    let content = client.get_script(&name).await.map_err(|e| e.to_string())?;
                    Ok((name, content))
                },
                Message::ScriptDownloaded,
            )
        }
        ScriptListMessage::ActivateScript(name) => {
            let client = state.client.clone();
            state.status = format!("Activating {name}...");
            Task::perform(
                async move {
                    client
                        .lock()
                        .await
                        .set_active(&name)
                        .await
                        .map(|_| name)
                        .map_err(|e| e.to_string())
                },
                Message::ScriptActivated,
            )
        }
        ScriptListMessage::DeactivateScripts => {
            let client = state.client.clone();
            state.status = "Deactivating all scripts...".to_string();
            Task::perform(
                async move {
                    client
                        .lock()
                        .await
                        .set_active("")
                        .await
                        .map(|_| String::new())
                        .map_err(|e| e.to_string())
                },
                Message::ScriptActivated,
            )
        }
        ScriptListMessage::DeleteScript(name) => {
            let client = state.client.clone();
            state.status = format!("Deleting {name}...");
            Task::perform(
                async move {
                    client
                        .lock()
                        .await
                        .delete_script(&name)
                        .await
                        .map(|_| name)
                        .map_err(|e| e.to_string())
                },
                Message::ScriptDeleted,
            )
        }
    }
}

fn refresh_scripts(state: &mut Sievers) -> Task<Message> {
    let client = state.client.clone();
    Task::perform(
        async move {
            client
                .lock()
                .await
                .list_scripts()
                .await
                .map_err(|e| e.to_string())
        },
        Message::ScriptsLoaded,
    )
}

fn handle_rule_message(state: &mut Sievers, idx: usize, msg: RuleMessage) {
    let rule = &mut state.rules[idx];
    match msg {
        RuleMessage::SetName(name) => rule.name = name,
        RuleMessage::SetEnabled(enabled) => rule.enabled = enabled,
        RuleMessage::SetLogic(opt) => rule.logic = opt.0,
        RuleMessage::RemoveRule => {
            state.rules.remove(idx);
        }
        RuleMessage::AddCondition => {
            rule.conditions.push(Condition::default());
        }
        RuleMessage::AddAction => {
            rule.actions.push(Action::default());
        }
        RuleMessage::ConditionMsg(ci, cmsg) => {
            if ci < rule.conditions.len() {
                handle_condition_message(&mut rule.conditions, ci, cmsg);
            }
        }
        RuleMessage::ActionMsg(ai, amsg) => {
            if ai < rule.actions.len() {
                handle_action_message(&mut rule.actions, ai, amsg);
            }
        }
    }
}

fn handle_condition_message(conditions: &mut Vec<Condition>, idx: usize, msg: ConditionMessage) {
    match msg {
        ConditionMessage::SetTestType(opt) => conditions[idx].test_type = opt.0,
        ConditionMessage::SetMatchType(opt) => conditions[idx].match_type = opt.0,
        ConditionMessage::SetAddressPart(opt) => conditions[idx].address_part = opt.0,
        ConditionMessage::SetSizeComparator(opt) => conditions[idx].size_comparator = opt.0,
        ConditionMessage::SetHeaders(s) => {
            conditions[idx].header_names = s.split(',').map(|h| h.trim().to_string()).collect();
        }
        ConditionMessage::SetValue(s) => {
            if conditions[idx].test_type == ConditionTest::Size {
                conditions[idx].size_value = s;
            } else {
                conditions[idx].keys = vec![s];
            }
        }
        ConditionMessage::Remove => {
            conditions.remove(idx);
        }
    }
}

fn handle_action_message(actions: &mut Vec<Action>, idx: usize, msg: ActionMessage) {
    match msg {
        ActionMessage::SetActionType(opt) => actions[idx].action_type = opt.0,
        ActionMessage::SetArgument(s) => actions[idx].argument = s,
        ActionMessage::Remove => {
            actions.remove(idx);
        }
    }
}

// --- Bidirectional sync ---

fn sync_visual_to_raw(state: &mut Sievers) {
    state.syncing = true;
    let script = crate::model::script::SieveScript {
        rules: state.rules.clone(),
        ..Default::default()
    };
    let text = converter::script_to_text(&script);
    state.editor_content = text_editor::Content::with_text(&text);
    state.raw_dirty = false;
    state.last_raw_edit = None;
    state.syncing = false;
}

fn sync_raw_to_visual(state: &mut Sievers) {
    state.syncing = true;
    let text = state.editor_content.text();
    let script = converter::text_to_script(&text, "");
    state.rules = script.rules;
    state.raw_dirty = false;
    state.last_raw_edit = None;
    // Auto-select first rule if none selected
    if !state.rules.is_empty() && state.selected_rule.is_none() {
        state.selected_rule = Some(0);
    }
    // Clamp selected_rule if rules shrunk
    if let Some(sel) = state.selected_rule {
        if sel >= state.rules.len() {
            state.selected_rule = if state.rules.is_empty() {
                None
            } else {
                Some(state.rules.len() - 1)
            };
        }
    }
    state.syncing = false;
}

// --- View ---

pub fn view(state: &Sievers) -> Element<'_, Message> {
    let toolbar = ui::toolbar::view(state.connected, state.dark_mode);
    let tab_bar = view_tab_bar(state.active_tab);

    let editor_area = match state.active_tab {
        Tab::Visual => ui::visual_editor::view(&state.rules, state.selected_rule),
        Tab::Raw => ui::raw_editor::view(&state.editor_content),
    };

    let status_bar = ui::status_bar::view(&state.status);

    // Main layout: optional sidebar + editor
    let main_content: Element<'_, Message> = if state.connected {
        let sidebar = ui::script_list::view(
            &state.server_scripts,
            state.selected_script.as_deref(),
        )
        .map(Message::ScriptListMsg);

        row![sidebar, column![tab_bar, editor_area].width(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        column![tab_bar, editor_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    let mut content: Element<'_, Message> =
        column![toolbar, main_content, status_bar].into();

    // Connection modal overlay
    if state.connection.visible {
        content = iced::widget::stack![
            content,
            ui::connection_modal::view(&state.connection).map(Message::ConnectionMsg),
        ]
        .into();
    }

    // About modal overlay
    if state.about.visible {
        content = iced::widget::stack![
            content,
            ui::about_modal::view(&state.about).map(Message::AboutMsg),
        ]
        .into();
    }

    content
}

fn view_tab_bar(active: Tab) -> Element<'static, Message> {
    let visual_style = if active == Tab::Visual {
        iced::widget::button::primary
    } else {
        iced::widget::button::secondary
    };
    let raw_style = if active == Tab::Raw {
        iced::widget::button::primary
    } else {
        iced::widget::button::secondary
    };

    container(
        row![
            iced::widget::button("Visual")
                .on_press(Message::SwitchTab(Tab::Visual))
                .style(visual_style),
            iced::widget::button("Raw")
                .on_press(Message::SwitchTab(Tab::Raw))
                .style(raw_style),
        ]
        .spacing(2),
    )
    .padding([4, 8])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.palette();
        container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.text.r,
                palette.text.g,
                palette.text.b,
                0.03,
            ))),
            ..container::Style::default()
        }
    })
    .into()
}

pub fn theme(state: &Sievers) -> Theme {
    if state.dark_mode {
        Theme::Dark
    } else {
        Theme::Light
    }
}

pub fn subscription(state: &Sievers) -> Subscription<Message> {
    let mut subs = vec![iced::keyboard::on_key_press(handle_key_press)];

    if state.raw_dirty && state.last_raw_edit.is_some() {
        subs.push(
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| Message::DebounceCheck),
        );
    }

    Subscription::batch(subs)
}

fn handle_key_press(
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<Message> {
    use iced::keyboard::key::Named;
    use iced::keyboard::Key;

    if modifiers.control() {
        match &key {
            Key::Character(c) if c.as_str() == "o" => Some(Message::OpenFile),
            Key::Character(c) if c.as_str() == "s" => Some(Message::SaveFile),
            Key::Character(c) if c.as_str() == "u" => Some(Message::Upload),
            Key::Character(c) if c.as_str() == "C" && modifiers.shift() => Some(Message::Connect),
            Key::Named(Named::Tab) => Some(Message::SwitchTab(Tab::Visual)), // Ctrl+Tab toggles
            _ => None,
        }
    } else {
        None
    }
}

// --- Async file operations ---

async fn open_file_dialog() -> Result<(PathBuf, String), String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open SIEVE Script")
        .add_filter("SIEVE Scripts", &["siv", "sieve"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await;

    match handle {
        Some(handle) => {
            let path = handle.path().to_path_buf();
            let text = script_io::load_script(&path).map_err(|e| e.to_string())?;
            Ok((path, text))
        }
        None => Err("Cancelled".to_string()),
    }
}

async fn save_file_dialog(
    current_path: Option<PathBuf>,
    text: String,
) -> Result<PathBuf, String> {
    let path = if let Some(path) = current_path {
        path
    } else {
        let handle = rfd::AsyncFileDialog::new()
            .set_title("Save SIEVE Script")
            .add_filter("SIEVE Scripts", &["siv", "sieve"])
            .add_filter("All Files", &["*"])
            .save_file()
            .await;

        match handle {
            Some(handle) => handle.path().to_path_buf(),
            None => return Err("Cancelled".to_string()),
        }
    };

    script_io::save_script(&path, &text).map_err(|e| e.to_string())?;
    Ok(path)
}
