use serde::{Deserialize, Serialize};

use crate::model::rule::SieveRule;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SieveScript {
    pub name: String,
    pub rules: Vec<SieveRule>,
    pub requires: Vec<String>,
    pub active: bool,
}
