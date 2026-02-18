/// Recursive descent SIEVE parser.
///
/// Parses tokenized SIEVE scripts into an AST. Unrecognized constructs
/// are captured as `Command::Raw` for round-trip preservation.
use crate::sieve::ast::*;
use crate::sieve::lexer::{Token, tokenize};

pub fn parse(input: &str) -> Result<Script, String> {
    if input.trim().is_empty() {
        return Ok(Script { commands: Vec::new() });
    }

    let spans = tokenize(input)?;
    let tokens: Vec<&Token> = spans.iter().map(|s| &s.token).collect();
    let mut pos = 0;
    let mut commands = Vec::new();
    let mut pending_comment: Option<String> = None;
    let mut saw_valid_command = false;

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Comment(text) => {
                pending_comment = Some(text.clone());
                pos += 1;
            }
            Token::BlockComment(_) => {
                pos += 1;
            }
            Token::Identifier(ident) => {
                let lower = ident.to_lowercase();
                match lower.as_str() {
                    "require" => {
                        pos += 1;
                        let exts = parse_require_args(&tokens, &mut pos)?;
                        commands.push(Command::Require(exts));
                        saw_valid_command = true;
                    }
                    "if" => {
                        pos += 1;
                        let filter_name = extract_filter_name(&pending_comment);
                        let enabled = pending_comment
                            .as_ref()
                            .map(|c| !c.contains("[DISABLED]"))
                            .unwrap_or(true);
                        pending_comment = None;
                        let if_block = parse_if_block(&tokens, &mut pos, filter_name, enabled)?;
                        commands.push(Command::If(if_block));
                        saw_valid_command = true;
                    }
                    // Known top-level action commands
                    "keep" | "stop" | "discard" | "fileinto" | "redirect"
                    | "reject" | "setflag" | "addflag" | "removeflag" => {
                        pending_comment = None;
                        let action = parse_action_command(&tokens, &mut pos)?;
                        commands.push(Command::Action(action));
                        saw_valid_command = true;
                    }
                    _ => {
                        // Unknown identifier at top level — not valid SIEVE
                        return Err(format!("Unknown command '{}' at top level", ident));
                    }
                }
            }
            _ => {
                return Err(format!("Unexpected token {:?} at top level", tokens[pos]));
            }
        }
    }

    if !saw_valid_command && !tokens.is_empty() {
        // Only comments/whitespace — not really a valid script, but ok
        // (empty scripts handled above)
    }

    Ok(Script { commands })
}

fn extract_filter_name(comment: &Option<String>) -> Option<String> {
    comment.as_ref().and_then(|c| {
        let trimmed = c.trim();
        if let Some(name) = trimmed.strip_prefix("Filter:") {
            let name = name.trim();
            // Strip [DISABLED] suffix if present
            let name = name.strip_suffix("[DISABLED]").unwrap_or(name).trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        } else {
            None
        }
    })
}

fn parse_require_args(tokens: &[&Token], pos: &mut usize) -> Result<Vec<String>, String> {
    let mut exts = Vec::new();

    match tokens.get(*pos) {
        Some(Token::QuotedString(s)) => {
            exts.push(s.clone());
            *pos += 1;
        }
        Some(Token::LBracket) => {
            *pos += 1;
            loop {
                match tokens.get(*pos) {
                    Some(Token::QuotedString(s)) => {
                        exts.push(s.clone());
                        *pos += 1;
                    }
                    Some(Token::Comma) => {
                        *pos += 1;
                    }
                    Some(Token::RBracket) => {
                        *pos += 1;
                        break;
                    }
                    _ => break,
                }
            }
        }
        _ => {}
    }

    // Expect semicolon
    if matches!(tokens.get(*pos), Some(Token::Semicolon)) {
        *pos += 1;
    }

    Ok(exts)
}

fn parse_if_block(
    tokens: &[&Token],
    pos: &mut usize,
    name: Option<String>,
    enabled: bool,
) -> Result<IfBlock, String> {
    let condition = parse_test_expr(tokens, pos)?;
    let actions = parse_action_block(tokens, pos)?;
    let mut alternatives = Vec::new();

    // Parse elsif/else chain
    loop {
        match tokens.get(*pos) {
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("elsif") => {
                *pos += 1;
                let cond = parse_test_expr(tokens, pos)?;
                let acts = parse_action_block(tokens, pos)?;
                alternatives.push(Alternative::ElsIf {
                    condition: cond,
                    actions: acts,
                });
            }
            Some(Token::Identifier(s)) if s.eq_ignore_ascii_case("else") => {
                *pos += 1;
                let acts = parse_action_block(tokens, pos)?;
                alternatives.push(Alternative::Else { actions: acts });
                break;
            }
            _ => break,
        }
    }

    Ok(IfBlock {
        name,
        enabled,
        condition,
        actions,
        alternatives,
    })
}

fn parse_test_expr(tokens: &[&Token], pos: &mut usize) -> Result<TestExpr, String> {
    match tokens.get(*pos) {
        Some(Token::Identifier(ident)) => {
            let lower = ident.to_lowercase();
            match lower.as_str() {
                "allof" => {
                    *pos += 1;
                    let tests = parse_test_list(tokens, pos)?;
                    Ok(TestExpr::AllOf(tests))
                }
                "anyof" => {
                    *pos += 1;
                    let tests = parse_test_list(tokens, pos)?;
                    Ok(TestExpr::AnyOf(tests))
                }
                "not" => {
                    *pos += 1;
                    let inner = parse_test_expr(tokens, pos)?;
                    Ok(TestExpr::Not(Box::new(inner)))
                }
                "header" => {
                    *pos += 1;
                    parse_header_test(tokens, pos)
                }
                "address" => {
                    *pos += 1;
                    parse_address_test(tokens, pos, false)
                }
                "envelope" => {
                    *pos += 1;
                    parse_address_test(tokens, pos, true)
                }
                "size" => {
                    *pos += 1;
                    parse_size_test(tokens, pos)
                }
                "exists" => {
                    *pos += 1;
                    parse_exists_test(tokens, pos)
                }
                "body" => {
                    *pos += 1;
                    parse_body_test(tokens, pos)
                }
                "true" => {
                    *pos += 1;
                    Ok(TestExpr::True)
                }
                "false" => {
                    *pos += 1;
                    Ok(TestExpr::False)
                }
                _ => Err(format!("Unknown test '{ident}'")),
            }
        }
        Some(other) => Err(format!("Expected test expression, got {other:?}")),
        None => Err("Expected test expression, got end of input".to_string()),
    }
}

fn parse_test_list(tokens: &[&Token], pos: &mut usize) -> Result<Vec<TestExpr>, String> {
    // Expect '('
    if !matches!(tokens.get(*pos), Some(Token::LParen)) {
        return Err("Expected '(' in test list".to_string());
    }
    *pos += 1;

    let mut tests = Vec::new();
    loop {
        if matches!(tokens.get(*pos), Some(Token::RParen)) {
            *pos += 1;
            break;
        }
        if !tests.is_empty() && matches!(tokens.get(*pos), Some(Token::Comma)) {
            *pos += 1;
        }
        if matches!(tokens.get(*pos), Some(Token::RParen)) {
            *pos += 1;
            break;
        }
        tests.push(parse_test_expr(tokens, pos)?);
    }

    Ok(tests)
}

fn parse_header_test(tokens: &[&Token], pos: &mut usize) -> Result<TestExpr, String> {
    let mut match_type = ":is".to_string();

    // Parse optional tags
    while let Some(Token::Tag(tag)) = tokens.get(*pos) {
        if tag == ":comparator" {
            *pos += 1;
            // Skip comparator argument
            if matches!(tokens.get(*pos), Some(Token::QuotedString(_))) {
                *pos += 1;
            }
            continue;
        }
        match_type = tag.clone();
        *pos += 1;
    }

    let header_names = parse_string_or_list(tokens, pos)?;
    let keys = parse_string_or_list(tokens, pos)?;

    Ok(TestExpr::Header {
        match_type,
        header_names,
        keys,
    })
}

fn parse_address_test(
    tokens: &[&Token],
    pos: &mut usize,
    is_envelope: bool,
) -> Result<TestExpr, String> {
    let mut match_type = ":is".to_string();
    let mut address_part: Option<String> = None;

    // Parse optional tags (match_type and address_part can appear in any order)
    while let Some(Token::Tag(tag)) = tokens.get(*pos) {
        if tag == ":comparator" {
            *pos += 1;
            if matches!(tokens.get(*pos), Some(Token::QuotedString(_))) {
                *pos += 1;
            }
            continue;
        }
        match tag.as_str() {
            ":all" | ":localpart" | ":domain" => {
                address_part = Some(tag.clone());
                *pos += 1;
            }
            _ => {
                match_type = tag.clone();
                *pos += 1;
            }
        }
    }

    let header_names = parse_string_or_list(tokens, pos)?;
    let keys = parse_string_or_list(tokens, pos)?;

    if is_envelope {
        Ok(TestExpr::Envelope {
            address_part,
            match_type,
            header_names,
            keys,
        })
    } else {
        Ok(TestExpr::Address {
            address_part,
            match_type,
            header_names,
            keys,
        })
    }
}

fn parse_size_test(tokens: &[&Token], pos: &mut usize) -> Result<TestExpr, String> {
    let mut comparator = ":over".to_string();

    if let Some(Token::Tag(tag)) = tokens.get(*pos) {
        comparator = tag.clone();
        *pos += 1;
    }

    let limit = match tokens.get(*pos) {
        Some(Token::Number(n)) => {
            let n = n.clone();
            *pos += 1;
            n
        }
        Some(Token::QuotedString(s)) => {
            let s = s.clone();
            *pos += 1;
            s
        }
        _ => "0".to_string(),
    };

    Ok(TestExpr::Size { comparator, limit })
}

fn parse_exists_test(tokens: &[&Token], pos: &mut usize) -> Result<TestExpr, String> {
    let header_names = parse_string_or_list(tokens, pos)?;
    Ok(TestExpr::Exists { header_names })
}

fn parse_body_test(tokens: &[&Token], pos: &mut usize) -> Result<TestExpr, String> {
    let mut match_type = ":is".to_string();

    while let Some(Token::Tag(tag)) = tokens.get(*pos) {
        if tag == ":comparator" {
            *pos += 1;
            if matches!(tokens.get(*pos), Some(Token::QuotedString(_))) {
                *pos += 1;
            }
            continue;
        }
        match_type = tag.clone();
        *pos += 1;
    }

    let keys = parse_string_or_list(tokens, pos)?;

    Ok(TestExpr::Body { match_type, keys })
}

fn parse_string_or_list(tokens: &[&Token], pos: &mut usize) -> Result<Vec<String>, String> {
    match tokens.get(*pos) {
        Some(Token::QuotedString(s)) => {
            let s = s.clone();
            *pos += 1;
            Ok(vec![s])
        }
        Some(Token::LBracket) => {
            *pos += 1;
            let mut items = Vec::new();
            loop {
                match tokens.get(*pos) {
                    Some(Token::QuotedString(s)) => {
                        items.push(s.clone());
                        *pos += 1;
                    }
                    Some(Token::Comma) => {
                        *pos += 1;
                    }
                    Some(Token::RBracket) => {
                        *pos += 1;
                        break;
                    }
                    _ => break,
                }
            }
            Ok(items)
        }
        _ => Ok(Vec::new()),
    }
}

fn parse_action_block(tokens: &[&Token], pos: &mut usize) -> Result<Vec<ActionCommand>, String> {
    if !matches!(tokens.get(*pos), Some(Token::LBrace)) {
        return Err("Expected '{' to start action block".to_string());
    }
    *pos += 1;

    let mut actions = Vec::new();
    loop {
        // Skip comments inside blocks
        while matches!(tokens.get(*pos), Some(Token::Comment(_)) | Some(Token::BlockComment(_))) {
            *pos += 1;
        }

        if matches!(tokens.get(*pos), Some(Token::RBrace)) {
            *pos += 1;
            break;
        }
        if *pos >= tokens.len() {
            return Err("Unexpected end of input in action block".to_string());
        }
        actions.push(parse_action_command(tokens, pos)?);
    }

    Ok(actions)
}

fn parse_action_command(tokens: &[&Token], pos: &mut usize) -> Result<ActionCommand, String> {
    let name = match tokens.get(*pos) {
        Some(Token::Identifier(s)) => {
            let s = s.clone();
            *pos += 1;
            s
        }
        Some(other) => return Err(format!("Expected action name, got {other:?}")),
        None => return Err("Expected action name, got end of input".to_string()),
    };

    let mut arguments = Vec::new();

    // Collect arguments until semicolon
    loop {
        match tokens.get(*pos) {
            Some(Token::Semicolon) => {
                *pos += 1;
                break;
            }
            Some(Token::QuotedString(s)) => {
                arguments.push(Argument::QuotedString(s.clone()));
                *pos += 1;
            }
            Some(Token::Number(n)) => {
                arguments.push(Argument::Number(n.clone()));
                *pos += 1;
            }
            Some(Token::Tag(t)) => {
                arguments.push(Argument::Tag(t.clone()));
                *pos += 1;
            }
            Some(Token::LBracket) => {
                *pos += 1;
                let mut items = Vec::new();
                loop {
                    match tokens.get(*pos) {
                        Some(Token::QuotedString(s)) => {
                            items.push(s.clone());
                            *pos += 1;
                        }
                        Some(Token::Comma) => {
                            *pos += 1;
                        }
                        Some(Token::RBracket) => {
                            *pos += 1;
                            break;
                        }
                        _ => break,
                    }
                }
                arguments.push(Argument::StringList(items));
            }
            _ => break,
        }
    }

    Ok(ActionCommand { name, arguments })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let script = parse("").unwrap();
        assert!(script.commands.is_empty());
    }

    #[test]
    fn test_parse_require() {
        let script = parse("require \"fileinto\";").unwrap();
        assert_eq!(script.commands.len(), 1);
        match &script.commands[0] {
            Command::Require(exts) => assert_eq!(exts, &["fileinto"]),
            _ => panic!("Expected Require"),
        }
    }

    #[test]
    fn test_parse_require_list() {
        let script = parse("require [\"fileinto\", \"reject\"];").unwrap();
        match &script.commands[0] {
            Command::Require(exts) => assert_eq!(exts, &["fileinto", "reject"]),
            _ => panic!("Expected Require"),
        }
    }

    #[test]
    fn test_parse_simple_if() {
        let input = r#"
# Filter: Move spam
if header :contains "Subject" "SPAM" {
    fileinto "Junk";
    stop;
}
"#;
        let script = parse(input).unwrap();
        let if_cmd = script.commands.iter().find(|c| matches!(c, Command::If(_)));
        assert!(if_cmd.is_some());
        if let Command::If(block) = if_cmd.unwrap() {
            assert_eq!(block.name.as_deref(), Some("Move spam"));
            assert!(block.enabled);
            assert_eq!(block.actions.len(), 2);
        }
    }

    #[test]
    fn test_parse_allof() {
        let input = r#"
if allof (header :is "From" "boss@example.com", header :contains "Subject" "urgent") {
    fileinto "Important";
}
"#;
        let script = parse(input).unwrap();
        let if_cmd = script.commands.iter().find(|c| matches!(c, Command::If(_)));
        if let Some(Command::If(block)) = if_cmd {
            match &block.condition {
                TestExpr::AllOf(tests) => assert_eq!(tests.len(), 2),
                _ => panic!("Expected AllOf"),
            }
        }
    }

    #[test]
    fn test_parse_address_domain() {
        let input = r#"if address :is :domain "From" "hapimag.com" {
    fileinto "INBOX/Hapimag";
}"#;
        let script = parse(input).unwrap();
        let if_cmd = script.commands.iter().find(|c| matches!(c, Command::If(_)));
        if let Some(Command::If(block)) = if_cmd {
            match &block.condition {
                TestExpr::Address {
                    address_part,
                    match_type,
                    header_names,
                    keys,
                } => {
                    assert_eq!(address_part.as_deref(), Some(":domain"));
                    assert_eq!(match_type, ":is");
                    assert_eq!(header_names, &["From"]);
                    assert_eq!(keys, &["hapimag.com"]);
                }
                _ => panic!("Expected Address test"),
            }
        }
    }
}
