/// AST node types for SIEVE scripts (RFC 5228).
/// A complete SIEVE script is a list of commands.
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub commands: Vec<Command>,
}

/// A top-level command in a SIEVE script.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// `require ["ext1", "ext2"];`
    Require(Vec<String>),

    /// `if <test> { <actions> }` with optional elsif/else chain
    If(IfBlock),

    /// A simple action command like `keep;`, `stop;`, `fileinto "X";`
    Action(ActionCommand),

    /// A comment line: `# text`
    Comment(String),

    /// Unrecognized content preserved as raw text
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBlock {
    /// The filter name extracted from a preceding `# Filter: name` comment
    pub name: Option<String>,
    /// Whether the filter is enabled (disabled = `# Filter: name [DISABLED]` or wrapped in comment)
    pub enabled: bool,
    pub condition: TestExpr,
    pub actions: Vec<ActionCommand>,
    /// elsif/else chain
    pub alternatives: Vec<Alternative>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Alternative {
    ElsIf {
        condition: TestExpr,
        actions: Vec<ActionCommand>,
    },
    Else {
        actions: Vec<ActionCommand>,
    },
}

/// A test expression in an if/elsif condition.
#[derive(Debug, Clone, PartialEq)]
pub enum TestExpr {
    /// `allof (test1, test2, ...)`
    AllOf(Vec<TestExpr>),
    /// `anyof (test1, test2, ...)`
    AnyOf(Vec<TestExpr>),
    /// `not <test>`
    Not(Box<TestExpr>),
    /// `header :match_type "Header" "value"`
    Header {
        match_type: String,
        header_names: Vec<String>,
        keys: Vec<String>,
    },
    /// `address [:address_part] :match_type "Header" "value"`
    Address {
        address_part: Option<String>,
        match_type: String,
        header_names: Vec<String>,
        keys: Vec<String>,
    },
    /// `envelope [:address_part] :match_type "Header" "value"`
    Envelope {
        address_part: Option<String>,
        match_type: String,
        header_names: Vec<String>,
        keys: Vec<String>,
    },
    /// `size :over/:under <limit>`
    Size {
        comparator: String,
        limit: String,
    },
    /// `exists "Header"`
    Exists {
        header_names: Vec<String>,
    },
    /// `body :match_type "value"`
    Body {
        match_type: String,
        keys: Vec<String>,
    },
    /// `true`
    True,
    /// `false`
    False,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionCommand {
    pub name: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    QuotedString(String),
    Number(String),
    Tag(String),
    StringList(Vec<String>),
}
