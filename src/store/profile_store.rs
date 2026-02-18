use std::fs;
use std::path::PathBuf;

use crate::config::paths;
use crate::model::profile::ConnectionProfile;

const PROFILES_FILE: &str = "profiles.json";

fn profiles_path() -> Option<PathBuf> {
    paths::config_dir().map(|d| d.join(PROFILES_FILE))
}

pub fn load_profiles() -> Vec<ConnectionProfile> {
    let Some(path) = profiles_path() else {
        return Vec::new();
    };
    let Ok(data) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_profiles(profiles: &[ConnectionProfile]) {
    let Some(path) = profiles_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(profiles) {
        let _ = fs::write(&path, data);
    }
}
