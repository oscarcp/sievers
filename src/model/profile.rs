use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub use_starttls: bool,
}

impl Default for ConnectionProfile {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            port: 4190,
            username: String::new(),
            use_starttls: true,
        }
    }
}
