use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    Is,
    Contains,
    Matches,
    Regex,
}

impl MatchType {
    pub fn as_sieve(&self) -> &'static str {
        match self {
            Self::Is => ":is",
            Self::Contains => ":contains",
            Self::Matches => ":matches",
            Self::Regex => ":regex",
        }
    }

    pub fn from_sieve(s: &str) -> Option<Self> {
        match s {
            ":is" => Some(Self::Is),
            ":contains" => Some(Self::Contains),
            ":matches" => Some(Self::Matches),
            ":regex" => Some(Self::Regex),
            _ => None,
        }
    }
}

impl fmt::Display for MatchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sieve())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressPartType {
    All,
    Localpart,
    Domain,
}

impl AddressPartType {
    pub fn as_sieve(&self) -> &'static str {
        match self {
            Self::All => ":all",
            Self::Localpart => ":localpart",
            Self::Domain => ":domain",
        }
    }

    pub fn from_sieve(s: &str) -> Option<Self> {
        match s {
            ":all" => Some(Self::All),
            ":localpart" => Some(Self::Localpart),
            ":domain" => Some(Self::Domain),
            _ => None,
        }
    }
}

impl fmt::Display for AddressPartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sieve())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeComparator {
    Over,
    Under,
}

impl SizeComparator {
    pub fn as_sieve(&self) -> &'static str {
        match self {
            Self::Over => ":over",
            Self::Under => ":under",
        }
    }

    pub fn from_sieve(s: &str) -> Option<Self> {
        match s {
            ":over" => Some(Self::Over),
            ":under" => Some(Self::Under),
            _ => None,
        }
    }
}

impl fmt::Display for SizeComparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sieve())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicOperator {
    AllOf,
    AnyOf,
}

impl LogicOperator {
    pub fn as_sieve(&self) -> &'static str {
        match self {
            Self::AllOf => "allof",
            Self::AnyOf => "anyof",
        }
    }

    pub fn from_sieve(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "allof" => Some(Self::AllOf),
            "anyof" => Some(Self::AnyOf),
            _ => None,
        }
    }
}

impl fmt::Display for LogicOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sieve())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Fileinto,
    Redirect,
    Reject,
    Discard,
    Keep,
    Stop,
    Setflag,
    Addflag,
    Removeflag,
}

impl ActionType {
    pub fn as_sieve(&self) -> &'static str {
        match self {
            Self::Fileinto => "fileinto",
            Self::Redirect => "redirect",
            Self::Reject => "reject",
            Self::Discard => "discard",
            Self::Keep => "keep",
            Self::Stop => "stop",
            Self::Setflag => "setflag",
            Self::Addflag => "addflag",
            Self::Removeflag => "removeflag",
        }
    }

    pub fn from_sieve(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fileinto" => Some(Self::Fileinto),
            "redirect" => Some(Self::Redirect),
            "reject" => Some(Self::Reject),
            "discard" => Some(Self::Discard),
            "keep" => Some(Self::Keep),
            "stop" => Some(Self::Stop),
            "setflag" => Some(Self::Setflag),
            "addflag" => Some(Self::Addflag),
            "removeflag" => Some(Self::Removeflag),
            _ => None,
        }
    }

    pub fn takes_argument(&self) -> bool {
        !matches!(self, Self::Discard | Self::Keep | Self::Stop)
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sieve())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionTest {
    Header,
    Address,
    Envelope,
    Size,
    Exists,
    True,
    False,
    Not,
    Body,
}

impl ConditionTest {
    pub fn as_sieve(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Address => "address",
            Self::Envelope => "envelope",
            Self::Size => "size",
            Self::Exists => "exists",
            Self::True => "true",
            Self::False => "false",
            Self::Not => "not",
            Self::Body => "body",
        }
    }

    pub fn from_sieve(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "header" => Some(Self::Header),
            "address" => Some(Self::Address),
            "envelope" => Some(Self::Envelope),
            "size" => Some(Self::Size),
            "exists" => Some(Self::Exists),
            "true" => Some(Self::True),
            "false" => Some(Self::False),
            "not" => Some(Self::Not),
            "body" => Some(Self::Body),
            _ => None,
        }
    }
}

impl fmt::Display for ConditionTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sieve())
    }
}
