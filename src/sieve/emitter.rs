/// Emit SIEVE script text from AST nodes.
use crate::sieve::ast::*;

pub fn emit(script: &Script) -> String {
    let mut out = String::new();
    let mut first = true;

    // Collect all requires into a single statement
    let mut all_requires: Vec<String> = Vec::new();
    for cmd in &script.commands {
        if let Command::Require(exts) = cmd {
            for ext in exts {
                if !all_requires.contains(ext) {
                    all_requires.push(ext.clone());
                }
            }
        }
    }

    if !all_requires.is_empty() {
        if all_requires.len() == 1 {
            out.push_str(&format!("require \"{}\";\n", all_requires[0]));
        } else {
            let list = all_requires
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("require [{list}];\n"));
        }
        first = false;
    }

    for cmd in &script.commands {
        match cmd {
            Command::Require(_) => {} // Already handled above
            Command::If(block) => {
                if !first {
                    out.push('\n');
                }
                emit_if_block(&mut out, block);
                first = false;
            }
            Command::Action(action) => {
                emit_action(&mut out, action, 0);
                first = false;
            }
            Command::Comment(text) => {
                out.push_str(&format!("# {text}\n"));
                // Don't set first=false so we don't get extra blank lines
            }
            Command::Raw(text) => {
                if !first {
                    out.push('\n');
                }
                out.push_str(text);
                out.push('\n');
                first = false;
            }
        }
    }

    out
}

fn emit_if_block(out: &mut String, block: &IfBlock) {
    // Emit filter name comment
    if let Some(name) = &block.name {
        if block.enabled {
            out.push_str(&format!("# Filter: {name}\n"));
        } else {
            out.push_str(&format!("# Filter: {name} [DISABLED]\n"));
        }
    }

    out.push_str("if ");
    emit_test_expr(out, &block.condition);
    out.push_str(" {\n");

    for action in &block.actions {
        emit_action(out, action, 1);
    }

    out.push('}');

    for alt in &block.alternatives {
        match alt {
            Alternative::ElsIf { condition, actions } => {
                out.push_str(" elsif ");
                emit_test_expr(out, condition);
                out.push_str(" {\n");
                for action in actions {
                    emit_action(out, action, 1);
                }
                out.push('}');
            }
            Alternative::Else { actions } => {
                out.push_str(" else {\n");
                for action in actions {
                    emit_action(out, action, 1);
                }
                out.push('}');
            }
        }
    }

    out.push('\n');
}

fn emit_test_expr(out: &mut String, expr: &TestExpr) {
    match expr {
        TestExpr::AllOf(tests) => {
            out.push_str("allof (");
            for (i, test) in tests.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_test_expr(out, test);
            }
            out.push(')');
        }
        TestExpr::AnyOf(tests) => {
            out.push_str("anyof (");
            for (i, test) in tests.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_test_expr(out, test);
            }
            out.push(')');
        }
        TestExpr::Not(inner) => {
            out.push_str("not ");
            emit_test_expr(out, inner);
        }
        TestExpr::Header {
            match_type,
            header_names,
            keys,
        } => {
            out.push_str("header ");
            out.push_str(match_type);
            out.push(' ');
            emit_string_or_list(out, header_names);
            out.push(' ');
            emit_string_or_list(out, keys);
        }
        TestExpr::Address {
            address_part,
            match_type,
            header_names,
            keys,
        } => {
            out.push_str("address ");
            out.push_str(match_type);
            if let Some(ap) = address_part {
                if ap != ":all" {
                    out.push(' ');
                    out.push_str(ap);
                }
            }
            out.push(' ');
            emit_string_or_list(out, header_names);
            out.push(' ');
            emit_string_or_list(out, keys);
        }
        TestExpr::Envelope {
            address_part,
            match_type,
            header_names,
            keys,
        } => {
            out.push_str("envelope ");
            out.push_str(match_type);
            if let Some(ap) = address_part {
                if ap != ":all" {
                    out.push(' ');
                    out.push_str(ap);
                }
            }
            out.push(' ');
            emit_string_or_list(out, header_names);
            out.push(' ');
            emit_string_or_list(out, keys);
        }
        TestExpr::Size { comparator, limit } => {
            out.push_str("size ");
            out.push_str(comparator);
            out.push(' ');
            out.push_str(limit);
        }
        TestExpr::Exists { header_names } => {
            out.push_str("exists ");
            emit_string_or_list(out, header_names);
        }
        TestExpr::Body { match_type, keys } => {
            out.push_str("body ");
            out.push_str(match_type);
            out.push(' ');
            emit_string_or_list(out, keys);
        }
        TestExpr::True => out.push_str("true"),
        TestExpr::False => out.push_str("false"),
    }
}

fn emit_string_or_list(out: &mut String, items: &[String]) {
    if items.len() == 1 {
        out.push_str(&format!("\"{}\"", escape_sieve_string(&items[0])));
    } else {
        out.push('[');
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("\"{}\"", escape_sieve_string(item)));
        }
        out.push(']');
    }
}

fn escape_sieve_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn emit_action(out: &mut String, action: &ActionCommand, indent: usize) {
    let prefix = "    ".repeat(indent);
    out.push_str(&prefix);
    out.push_str(&action.name);
    for arg in &action.arguments {
        out.push(' ');
        match arg {
            Argument::QuotedString(s) => {
                out.push_str(&format!("\"{}\"", escape_sieve_string(s)));
            }
            Argument::Number(n) => out.push_str(n),
            Argument::Tag(t) => out.push_str(t),
            Argument::StringList(items) => {
                emit_string_or_list(out, items);
            }
        }
    }
    out.push_str(";\n");
}

/// Compute what `require` extensions a script's AST needs.
pub fn compute_requires(script: &Script) -> Vec<String> {
    let mut requires = std::collections::BTreeSet::new();

    for cmd in &script.commands {
        match cmd {
            Command::If(block) => {
                collect_test_requires(&block.condition, &mut requires);
                collect_action_requires(&block.actions, &mut requires);
                for alt in &block.alternatives {
                    match alt {
                        Alternative::ElsIf { condition, actions } => {
                            collect_test_requires(condition, &mut requires);
                            collect_action_requires(actions, &mut requires);
                        }
                        Alternative::Else { actions } => {
                            collect_action_requires(actions, &mut requires);
                        }
                    }
                }
            }
            Command::Action(action) => {
                collect_single_action_require(action, &mut requires);
            }
            _ => {}
        }
    }

    requires.into_iter().collect()
}

fn collect_test_requires(expr: &TestExpr, requires: &mut std::collections::BTreeSet<String>) {
    match expr {
        TestExpr::AllOf(tests) | TestExpr::AnyOf(tests) => {
            for t in tests {
                collect_test_requires(t, requires);
            }
        }
        TestExpr::Not(inner) => collect_test_requires(inner, requires),
        TestExpr::Envelope { .. } => {
            requires.insert("envelope".to_string());
        }
        TestExpr::Body { .. } => {
            requires.insert("body".to_string());
        }
        TestExpr::Header { match_type, .. }
        | TestExpr::Address { match_type, .. } => {
            if match_type == ":regex" {
                requires.insert("regex".to_string());
            }
        }
        _ => {}
    }
}

fn collect_action_requires(actions: &[ActionCommand], requires: &mut std::collections::BTreeSet<String>) {
    for action in actions {
        collect_single_action_require(action, requires);
    }
}

fn collect_single_action_require(action: &ActionCommand, requires: &mut std::collections::BTreeSet<String>) {
    match action.name.to_lowercase().as_str() {
        "fileinto" => { requires.insert("fileinto".to_string()); }
        "reject" => { requires.insert("reject".to_string()); }
        "setflag" | "addflag" | "removeflag" => { requires.insert("imap4flags".to_string()); }
        _ => {}
    }
}
