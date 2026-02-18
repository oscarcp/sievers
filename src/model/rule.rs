use serde::{Deserialize, Serialize};

use crate::model::enums::{
    ActionType, AddressPartType, ConditionTest, LogicOperator, MatchType, SizeComparator,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub test_type: ConditionTest,
    pub header_names: Vec<String>,
    pub keys: Vec<String>,
    pub match_type: MatchType,
    pub address_part: AddressPartType,
    pub size_comparator: SizeComparator,
    pub size_value: String,
    pub negate: bool,
}

impl Default for Condition {
    fn default() -> Self {
        Self {
            test_type: ConditionTest::Header,
            header_names: vec!["From".to_string()],
            keys: vec![String::new()],
            match_type: MatchType::Contains,
            address_part: AddressPartType::All,
            size_comparator: SizeComparator::Over,
            size_value: "0".to_string(),
            negate: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub argument: String,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            action_type: ActionType::Keep,
            argument: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SieveRule {
    pub name: String,
    pub enabled: bool,
    pub logic: LogicOperator,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    /// Opaque text for unrecognized constructs
    pub raw_block: Option<String>,
}

impl Default for SieveRule {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            logic: LogicOperator::AllOf,
            conditions: Vec::new(),
            actions: Vec::new(),
            raw_block: None,
        }
    }
}
