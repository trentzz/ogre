use anyhow::{bail, Result};

/// Read an identifier (alphanumeric + underscore) from `chars` starting at `*i`.
pub fn take_identifier(chars: &[char], i: &mut usize) -> String {
    let mut s = String::new();
    while *i < chars.len() && (chars[*i].is_alphanumeric() || chars[*i] == '_') {
        s.push(chars[*i]);
        *i += 1;
    }
    s
}

/// Skip horizontal whitespace (spaces and tabs).
pub fn skip_spaces(chars: &[char], i: &mut usize) {
    while *i < chars.len() && (chars[*i] == ' ' || chars[*i] == '\t') {
        *i += 1;
    }
}

/// Skip all whitespace (including newlines).
pub fn skip_whitespace(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }
}

/// Read a double-quoted string, consuming both quotes.
pub fn take_quoted_string(chars: &[char], i: &mut usize) -> Result<String> {
    if *i >= chars.len() || chars[*i] != '"' {
        bail!("expected '\"', found {:?}", chars.get(*i));
    }
    *i += 1; // skip opening quote
    let mut s = String::new();
    loop {
        if *i >= chars.len() {
            bail!("unterminated string literal");
        }
        if chars[*i] == '"' {
            *i += 1; // skip closing quote
            break;
        }
        s.push(chars[*i]);
        *i += 1;
    }
    Ok(s)
}

/// Read everything up to the matching `}`, consuming the `}`.
/// The opening `{` has already been consumed by the caller.
pub fn take_brace_body(chars: &[char], i: &mut usize) -> Result<String> {
    let mut body = String::new();
    let mut depth = 1usize;

    while *i < chars.len() {
        match chars[*i] {
            '{' => {
                depth += 1;
                body.push('{');
                *i += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    *i += 1; // consume closing '}'
                    break;
                }
                body.push('}');
                *i += 1;
            }
            c => {
                body.push(c);
                *i += 1;
            }
        }
    }

    if depth > 0 {
        bail!("unclosed '{{'");
    }

    Ok(body)
}
