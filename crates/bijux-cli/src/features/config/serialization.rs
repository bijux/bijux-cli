#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub(crate) fn decode_quoted_value(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

pub(crate) fn render_env(values: &BTreeMap<String, String>) -> String {
    let mut rendered = String::new();
    for (key, value) in values {
        rendered.push_str(&format!("BIJUXCLI_{}={}\n", key.to_ascii_uppercase(), value));
    }
    rendered
}
