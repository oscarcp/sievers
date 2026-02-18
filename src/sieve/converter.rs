/// Bidirectional conversion between SIEVE script text and SieveScript models.
///
/// `text_to_script()` — parse text → AST → model
/// `script_to_text()` — model → AST → emit text
use crate::model::enums::*;
use crate::model::rule::{Action, Condition, SieveRule};
use crate::model::script::SieveScript;
use crate::sieve::ast::*;
use crate::sieve::emitter;
use crate::sieve::parser;

/// Parse SIEVE script text into a SieveScript model.
pub fn text_to_script(text: &str, script_name: &str) -> SieveScript {
    if text.trim().is_empty() {
        return SieveScript {
            name: script_name.to_string(),
            ..Default::default()
        };
    }

    let ast = match parser::parse(text) {
        Ok(ast) => ast,
        Err(_) => {
            return SieveScript {
                name: script_name.to_string(),
                rules: vec![SieveRule {
                    name: "(parse error)".to_string(),
                    raw_block: Some(text.to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            };
        }
    };

    let mut requires = Vec::new();
    let mut rules = Vec::new();

    for cmd in &ast.commands {
        match cmd {
            Command::Require(exts) => {
                for ext in exts {
                    if !requires.contains(ext) {
                        requires.push(ext.clone());
                    }
                }
            }
            Command::If(block) => {
                let rule = if_block_to_rule(block);
                rules.push(rule);
            }
            Command::Action(_) | Command::Comment(_) | Command::Raw(_) => {}
        }
    }

    SieveScript {
        name: script_name.to_string(),
        rules,
        requires,
        ..Default::default()
    }
}

fn if_block_to_rule(block: &IfBlock) -> SieveRule {
    let (logic, conditions) = extract_conditions(&block.condition);
    let actions = extract_actions(&block.actions);

    if conditions.is_empty() && actions.is_empty() {
        // Fall back to raw block
        let raw_ast = Script {
            commands: vec![Command::If(block.clone())],
        };
        return SieveRule {
            name: block.name.clone().unwrap_or_default(),
            enabled: block.enabled,
            raw_block: Some(emitter::emit(&raw_ast)),
            ..Default::default()
        };
    }

    SieveRule {
        name: block.name.clone().unwrap_or_default(),
        enabled: block.enabled,
        logic,
        conditions,
        actions,
        raw_block: None,
    }
}

fn extract_conditions(expr: &TestExpr) -> (LogicOperator, Vec<Condition>) {
    match expr {
        TestExpr::AllOf(tests) => {
            let conditions: Vec<Condition> = tests.iter().filter_map(single_test_to_condition).collect();
            (LogicOperator::AllOf, conditions)
        }
        TestExpr::AnyOf(tests) => {
            let conditions: Vec<Condition> = tests.iter().filter_map(single_test_to_condition).collect();
            (LogicOperator::AnyOf, conditions)
        }
        _ => {
            if let Some(c) = single_test_to_condition(expr) {
                (LogicOperator::AllOf, vec![c])
            } else {
                (LogicOperator::AllOf, vec![])
            }
        }
    }
}

fn single_test_to_condition(expr: &TestExpr) -> Option<Condition> {
    match expr {
        TestExpr::Header {
            match_type,
            header_names,
            keys,
        } => Some(Condition {
            test_type: ConditionTest::Header,
            header_names: header_names.clone(),
            keys: keys.clone(),
            match_type: MatchType::from_sieve(match_type).unwrap_or(MatchType::Contains),
            ..Default::default()
        }),
        TestExpr::Address {
            address_part,
            match_type,
            header_names,
            keys,
        } => Some(Condition {
            test_type: ConditionTest::Address,
            header_names: header_names.clone(),
            keys: keys.clone(),
            match_type: MatchType::from_sieve(match_type).unwrap_or(MatchType::Contains),
            address_part: address_part
                .as_deref()
                .and_then(AddressPartType::from_sieve)
                .unwrap_or(AddressPartType::All),
            ..Default::default()
        }),
        TestExpr::Envelope {
            address_part,
            match_type,
            header_names,
            keys,
        } => Some(Condition {
            test_type: ConditionTest::Envelope,
            header_names: header_names.clone(),
            keys: keys.clone(),
            match_type: MatchType::from_sieve(match_type).unwrap_or(MatchType::Contains),
            address_part: address_part
                .as_deref()
                .and_then(AddressPartType::from_sieve)
                .unwrap_or(AddressPartType::All),
            ..Default::default()
        }),
        TestExpr::Size { comparator, limit } => Some(Condition {
            test_type: ConditionTest::Size,
            size_comparator: SizeComparator::from_sieve(comparator).unwrap_or(SizeComparator::Over),
            size_value: limit.clone(),
            ..Default::default()
        }),
        TestExpr::Exists { header_names } => Some(Condition {
            test_type: ConditionTest::Exists,
            header_names: header_names.clone(),
            ..Default::default()
        }),
        TestExpr::Not(inner) => {
            single_test_to_condition(inner).map(|mut c| {
                c.negate = true;
                c
            })
        }
        _ => None,
    }
}

fn extract_actions(action_cmds: &[ActionCommand]) -> Vec<Action> {
    action_cmds
        .iter()
        .filter_map(|cmd| {
            let action_type = ActionType::from_sieve(&cmd.name)?;
            let argument = if action_type.takes_argument() {
                cmd.arguments.first().map(|a| match a {
                    Argument::QuotedString(s) => s.clone(),
                    Argument::Number(n) => n.clone(),
                    Argument::Tag(t) => t.clone(),
                    Argument::StringList(items) => items.join(", "),
                }).unwrap_or_default()
            } else {
                String::new()
            };
            Some(Action {
                action_type,
                argument,
            })
        })
        .collect()
}

/// Convert a SieveScript model back to SIEVE script text.
pub fn script_to_text(script: &SieveScript) -> String {
    let ast = script_to_ast(script);
    emitter::emit(&ast)
}

fn script_to_ast(script: &SieveScript) -> Script {
    let mut commands = Vec::new();

    // Compute requires from rules
    let requires = collect_requires(&script.rules);
    if !requires.is_empty() {
        commands.push(Command::Require(requires));
    }

    for rule in &script.rules {
        if let Some(raw) = &rule.raw_block {
            // Try to re-parse raw blocks
            if let Ok(parsed) = parser::parse(raw) {
                for cmd in parsed.commands {
                    if matches!(cmd, Command::If(_)) {
                        commands.push(cmd);
                        break;
                    }
                }
            } else {
                commands.push(Command::Raw(raw.clone()));
            }
            continue;
        }

        let condition = build_test_expr(rule);
        let actions = build_action_commands(rule);

        commands.push(Command::If(IfBlock {
            name: if rule.name.is_empty() {
                None
            } else {
                Some(rule.name.clone())
            },
            enabled: rule.enabled,
            condition,
            actions,
            alternatives: Vec::new(),
        }));
    }

    Script { commands }
}

fn build_test_expr(rule: &SieveRule) -> TestExpr {
    if rule.conditions.is_empty() {
        return TestExpr::True;
    }

    let tests: Vec<TestExpr> = rule.conditions.iter().map(condition_to_test_expr).collect();

    if tests.len() == 1 {
        tests.into_iter().next().unwrap()
    } else {
        match rule.logic {
            LogicOperator::AllOf => TestExpr::AllOf(tests),
            LogicOperator::AnyOf => TestExpr::AnyOf(tests),
        }
    }
}

fn condition_to_test_expr(cond: &Condition) -> TestExpr {
    let expr = match cond.test_type {
        ConditionTest::Header => TestExpr::Header {
            match_type: cond.match_type.as_sieve().to_string(),
            header_names: cond.header_names.clone(),
            keys: cond.keys.clone(),
        },
        ConditionTest::Address => TestExpr::Address {
            address_part: if cond.address_part == AddressPartType::All {
                None
            } else {
                Some(cond.address_part.as_sieve().to_string())
            },
            match_type: cond.match_type.as_sieve().to_string(),
            header_names: cond.header_names.clone(),
            keys: cond.keys.clone(),
        },
        ConditionTest::Envelope => TestExpr::Envelope {
            address_part: if cond.address_part == AddressPartType::All {
                None
            } else {
                Some(cond.address_part.as_sieve().to_string())
            },
            match_type: cond.match_type.as_sieve().to_string(),
            header_names: cond.header_names.clone(),
            keys: cond.keys.clone(),
        },
        ConditionTest::Size => TestExpr::Size {
            comparator: cond.size_comparator.as_sieve().to_string(),
            limit: cond.size_value.clone(),
        },
        ConditionTest::Exists => TestExpr::Exists {
            header_names: cond.header_names.clone(),
        },
        ConditionTest::True => TestExpr::True,
        ConditionTest::False => TestExpr::False,
        ConditionTest::Body => TestExpr::Body {
            match_type: cond.match_type.as_sieve().to_string(),
            keys: cond.keys.clone(),
        },
        ConditionTest::Not => TestExpr::True, // fallback
    };

    if cond.negate {
        TestExpr::Not(Box::new(expr))
    } else {
        expr
    }
}

fn build_action_commands(rule: &SieveRule) -> Vec<ActionCommand> {
    rule.actions
        .iter()
        .map(|action| {
            let arguments = if action.action_type.takes_argument() && !action.argument.is_empty() {
                vec![Argument::QuotedString(action.argument.clone())]
            } else {
                vec![]
            };
            ActionCommand {
                name: action.action_type.as_sieve().to_string(),
                arguments,
            }
        })
        .collect()
}

fn collect_requires(rules: &[SieveRule]) -> Vec<String> {
    let mut requires = std::collections::BTreeSet::new();

    for rule in rules {
        for action in &rule.actions {
            match action.action_type {
                ActionType::Fileinto => { requires.insert("fileinto".to_string()); }
                ActionType::Reject => { requires.insert("reject".to_string()); }
                ActionType::Setflag | ActionType::Addflag | ActionType::Removeflag => {
                    requires.insert("imap4flags".to_string());
                }
                _ => {}
            }
        }
        for cond in &rule.conditions {
            match cond.test_type {
                ConditionTest::Body => { requires.insert("body".to_string()); }
                ConditionTest::Envelope => { requires.insert("envelope".to_string()); }
                _ => {}
            }
            if cond.match_type == MatchType::Regex {
                requires.insert("regex".to_string());
            }
        }
    }

    requires.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Port of Python test_converter.py ---

    const SIMPLE_FILEINTO: &str = r#"require "fileinto";

# Filter: Move spam
if header :contains "Subject" "SPAM" {
    fileinto "Junk";
    stop;
}
"#;

    const MULTI_CONDITION: &str = r#"require "fileinto";

# Filter: VIP mail
if allof (header :is "From" "boss@example.com", header :contains "Subject" "urgent") {
    fileinto "Important";
}
"#;

    const ANYOF_SCRIPT: &str = r#"require "fileinto";

# Filter: Newsletters
if anyof (header :contains "From" "news@a.com", header :contains "From" "news@b.com") {
    fileinto "Newsletters";
}
"#;

    const ADDRESS_DOMAIN_SCRIPT: &str = r#"require ["reject", "fileinto", "body", "mailbox"];

# Filter: HAPIMAG emails
if address :is :domain "From" "hapimag.com" {
    fileinto "INBOX/Hapimag";
}
"#;

    #[test]
    fn test_parse_simple_fileinto() {
        let script = text_to_script(SIMPLE_FILEINTO, "test");
        assert_eq!(script.name, "test");
        assert_eq!(script.rules.len(), 1);

        let rule = &script.rules[0];
        assert_eq!(rule.name, "Move spam");
        assert!(rule.enabled);
        assert_eq!(rule.conditions.len(), 1);
        assert!(rule.actions.len() >= 1);

        let cond = &rule.conditions[0];
        assert_eq!(cond.test_type, ConditionTest::Header);
        assert_eq!(cond.match_type, MatchType::Contains);
        assert!(cond.header_names.contains(&"Subject".to_string()));
        assert!(cond.keys.contains(&"SPAM".to_string()));

        let fileinto = &rule.actions[0];
        assert_eq!(fileinto.action_type, ActionType::Fileinto);
        assert_eq!(fileinto.argument, "Junk");
    }

    #[test]
    fn test_parse_multi_condition_allof() {
        let script = text_to_script(MULTI_CONDITION, "");
        assert_eq!(script.rules.len(), 1);

        let rule = &script.rules[0];
        assert_eq!(rule.logic, LogicOperator::AllOf);
        assert_eq!(rule.conditions.len(), 2);

        assert_eq!(rule.conditions[0].header_names, vec!["From"]);
        assert_eq!(rule.conditions[0].keys, vec!["boss@example.com"]);
        assert_eq!(rule.conditions[0].match_type, MatchType::Is);

        assert_eq!(rule.conditions[1].header_names, vec!["Subject"]);
        assert_eq!(rule.conditions[1].keys, vec!["urgent"]);
    }

    #[test]
    fn test_parse_anyof() {
        let script = text_to_script(ANYOF_SCRIPT, "");
        let rule = &script.rules[0];
        assert_eq!(rule.logic, LogicOperator::AnyOf);
        assert_eq!(rule.conditions.len(), 2);
    }

    #[test]
    fn test_roundtrip_simple() {
        let script1 = text_to_script(SIMPLE_FILEINTO, "test");
        let text = script_to_text(&script1);
        let script2 = text_to_script(&text, "test");

        assert_eq!(script2.rules.len(), script1.rules.len());
        let (r1, r2) = (&script1.rules[0], &script2.rules[0]);
        assert_eq!(r1.conditions[0].test_type, r2.conditions[0].test_type);
        assert_eq!(r1.conditions[0].header_names, r2.conditions[0].header_names);
        assert_eq!(r1.conditions[0].keys, r2.conditions[0].keys);
        assert_eq!(r1.actions[0].action_type, r2.actions[0].action_type);
        assert_eq!(r1.actions[0].argument, r2.actions[0].argument);
    }

    #[test]
    fn test_parse_error_becomes_raw() {
        let script = text_to_script("this is not valid sieve {{{", "");
        assert_eq!(script.rules.len(), 1);
        assert!(script.rules[0].raw_block.is_some());
    }

    #[test]
    fn test_generate_requires() {
        let script = text_to_script(SIMPLE_FILEINTO, "");
        let text = script_to_text(&script);
        let first_line = text.lines().next().unwrap_or("");
        assert!(first_line.contains("fileinto"));
    }

    #[test]
    fn test_empty_script() {
        let script = text_to_script("", "");
        assert_eq!(script.rules.len(), 0);
    }

    #[test]
    fn test_parse_address_domain() {
        let script = text_to_script(ADDRESS_DOMAIN_SCRIPT, "");
        assert_eq!(script.rules.len(), 1);

        let rule = &script.rules[0];
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.actions.len(), 1);
        assert!(rule.raw_block.is_none());

        let cond = &rule.conditions[0];
        assert_eq!(cond.test_type, ConditionTest::Address);
        assert_eq!(cond.match_type, MatchType::Is);
        assert_eq!(cond.address_part, AddressPartType::Domain);
        assert_eq!(cond.header_names, vec!["From"]);
        assert_eq!(cond.keys, vec!["hapimag.com"]);

        assert_eq!(rule.actions[0].action_type, ActionType::Fileinto);
        assert_eq!(rule.actions[0].argument, "INBOX/Hapimag");
    }

    #[test]
    fn test_roundtrip_address_domain() {
        let script1 = text_to_script(ADDRESS_DOMAIN_SCRIPT, "");
        let text = script_to_text(&script1);
        let script2 = text_to_script(&text, "");

        assert_eq!(script2.rules.len(), 1);
        let r = &script2.rules[0];
        assert_eq!(r.conditions.len(), 1);
        assert_eq!(r.conditions[0].test_type, ConditionTest::Address);
        assert_eq!(r.conditions[0].address_part, AddressPartType::Domain);
        assert_eq!(r.conditions[0].header_names, vec!["From"]);
        assert_eq!(r.conditions[0].keys, vec!["hapimag.com"]);
    }
}
