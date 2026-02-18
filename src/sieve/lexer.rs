/// SIEVE script tokenizer (RFC 5228).

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A `:tag` like `:is`, `:contains`, `:over`, `:domain`, etc.
    Tag(String),
    /// An unquoted identifier like `if`, `header`, `allof`, `fileinto`, etc.
    Identifier(String),
    /// A double-quoted string.
    QuotedString(String),
    /// A multi-line string `text:\r\n...\r\n.\r\n`
    MultiLineString(String),
    /// A numeric value, possibly with K/M/G suffix.
    Number(String),
    /// A `# ...` single-line comment.
    Comment(String),
    /// A `/* ... */` block comment.
    BlockComment(String),
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub token: Token,
    pub offset: usize,
    pub len: usize,
}

pub fn tokenize(input: &str) -> Result<Vec<Span>, String> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip whitespace
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        let start = i;

        match bytes[i] {
            b';' => {
                tokens.push(Span { token: Token::Semicolon, offset: start, len: 1 });
                i += 1;
            }
            b',' => {
                tokens.push(Span { token: Token::Comma, offset: start, len: 1 });
                i += 1;
            }
            b'(' => {
                tokens.push(Span { token: Token::LParen, offset: start, len: 1 });
                i += 1;
            }
            b')' => {
                tokens.push(Span { token: Token::RParen, offset: start, len: 1 });
                i += 1;
            }
            b'{' => {
                tokens.push(Span { token: Token::LBrace, offset: start, len: 1 });
                i += 1;
            }
            b'}' => {
                tokens.push(Span { token: Token::RBrace, offset: start, len: 1 });
                i += 1;
            }
            b'[' => {
                tokens.push(Span { token: Token::LBracket, offset: start, len: 1 });
                i += 1;
            }
            b']' => {
                tokens.push(Span { token: Token::RBracket, offset: start, len: 1 });
                i += 1;
            }

            // Single-line comment: # ...
            b'#' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                let text = &input[start + 1..i];
                tokens.push(Span {
                    token: Token::Comment(text.trim().to_string()),
                    offset: start,
                    len: i - start,
                });
            }

            // Block comment: /* ... */
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                let comment_start = i;
                loop {
                    if i + 1 >= bytes.len() {
                        return Err(format!("Unterminated block comment at offset {start}"));
                    }
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        break;
                    }
                    i += 1;
                }
                let text = &input[comment_start..i];
                i += 2; // skip */
                tokens.push(Span {
                    token: Token::BlockComment(text.trim().to_string()),
                    offset: start,
                    len: i - start,
                });
            }

            // Quoted string
            b'"' => {
                i += 1;
                let mut s = String::new();
                loop {
                    if i >= bytes.len() {
                        return Err(format!("Unterminated string at offset {start}"));
                    }
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        // Escape sequence
                        s.push(bytes[i + 1] as char);
                        i += 2;
                    } else if bytes[i] == b'"' {
                        i += 1;
                        break;
                    } else {
                        s.push(bytes[i] as char);
                        i += 1;
                    }
                }
                tokens.push(Span {
                    token: Token::QuotedString(s),
                    offset: start,
                    len: i - start,
                });
            }

            // Multi-line string: text:
            b't' | b'T'
                if i + 4 < bytes.len()
                    && input[i..i + 5].eq_ignore_ascii_case("text:") =>
            {
                i += 5;
                // Skip to end of line
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1; // skip \n
                }
                let body_start = i;
                // Read until a line that is just "."
                loop {
                    if i >= bytes.len() {
                        return Err(format!("Unterminated multi-line string at offset {start}"));
                    }
                    // Check if current line is ".\r\n" or ".\n"
                    if bytes[i] == b'.' {
                        let next = i + 1;
                        if next >= bytes.len()
                            || bytes[next] == b'\n'
                            || (bytes[next] == b'\r'
                                && next + 1 < bytes.len()
                                && bytes[next + 1] == b'\n')
                        {
                            let body = &input[body_start..i];
                            // Skip past the dot and newline
                            i += 1;
                            if i < bytes.len() && bytes[i] == b'\r' {
                                i += 1;
                            }
                            if i < bytes.len() && bytes[i] == b'\n' {
                                i += 1;
                            }
                            tokens.push(Span {
                                token: Token::MultiLineString(body.to_string()),
                                offset: start,
                                len: i - start,
                            });
                            break;
                        }
                    }
                    // Skip to next line
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                }
            }

            // Tag: :identifier
            b':' => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let tag = input[start..i].to_lowercase();
                tokens.push(Span {
                    token: Token::Tag(tag),
                    offset: start,
                    len: i - start,
                });
            }

            // Number
            b'0'..=b'9' => {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                // Optional K/M/G suffix
                if i < bytes.len() && matches!(bytes[i], b'K' | b'k' | b'M' | b'm' | b'G' | b'g')
                {
                    i += 1;
                }
                let num = input[start..i].to_string();
                tokens.push(Span {
                    token: Token::Number(num),
                    offset: start,
                    len: i - start,
                });
            }

            // Identifier
            _ if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' => {
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                let ident = input[start..i].to_string();
                tokens.push(Span {
                    token: Token::Identifier(ident),
                    offset: start,
                    len: i - start,
                });
            }

            _ => {
                return Err(format!(
                    "Unexpected character '{}' at offset {start}",
                    bytes[i] as char
                ));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let tokens = tokenize("require \"fileinto\";").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].token, Token::Identifier(s) if s == "require"));
        assert!(matches!(&tokens[1].token, Token::QuotedString(s) if s == "fileinto"));
        assert!(matches!(&tokens[2].token, Token::Semicolon));
    }

    #[test]
    fn test_tags_and_strings() {
        let tokens = tokenize("header :contains \"Subject\" \"SPAM\"").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[1].token, Token::Tag(s) if s == ":contains"));
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize("# Filter: test\nkeep;").unwrap();
        assert!(matches!(&tokens[0].token, Token::Comment(s) if s == "Filter: test"));
    }

    #[test]
    fn test_string_list() {
        let tokens = tokenize("[\"a\", \"b\"]").unwrap();
        assert_eq!(tokens.len(), 5); // [ "a" , "b" ]
    }

    #[test]
    fn test_number_with_suffix() {
        let tokens = tokenize("100K").unwrap();
        assert!(matches!(&tokens[0].token, Token::Number(s) if s == "100K"));
    }
}
