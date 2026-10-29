use std::path::PathBuf;
/// A closed native destination. Source never selects an executable or shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Internal(String),
    File {
        path: PathBuf,
        search: String,
        external: bool,
    },
    Id {
        id: String,
        search: String,
    },
    Attachment {
        path: PathBuf,
        search: String,
    },
    External(String),
}
impl Target {
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
            return Err("Link target requires 1..4096 bytes without controls".into());
        }
        let value = unescape(value);
        let (kind, body) = value.split_once(':').unwrap_or(("", &value));
        typed(&kind.to_ascii_lowercase(), body, &value)
    }
}
fn typed(kind: &str, body: &str, value: &str) -> Result<Target, String> {
    if ["file", "file+emacs", "file+sys"].contains(&kind) {
        let (path, search) = parts(body)?;
        return Ok(Target::File {
            path: PathBuf::from(path),
            search,
            external: kind == "file+sys",
        });
    }
    match kind {
        "id" => {
            let (id, search) = parts(body)?;
            Ok(Target::Id { id, search })
        }
        "attachment" => {
            let (path, search) = parts(body)?;
            Ok(Target::Attachment {
                path: PathBuf::from(path),
                search,
            })
        }
        _ => resource(kind, body, value),
    }
}
fn resource(kind: &str, body: &str, value: &str) -> Result<Target, String> {
    if ["https", "http", "ftp", "mailto", "news"].contains(&kind) {
        if body.is_empty() || value.chars().any(char::is_whitespace) {
            return Err("External link requires a nonempty encoded resource".into());
        }
        return Ok(Target::External(value.into()));
    }
    if ["shell", "elisp"].contains(&kind) {
        return Err("Executable Org link forms are not admitted".into());
    }
    implicit(value)
}

fn implicit(value: &str) -> Result<Target, String> {
    if value.starts_with('/') || value.starts_with("./") || value.starts_with("../") {
        let (path, search) = parts(value)?;
        return Ok(Target::File {
            path: PathBuf::from(path),
            search,
            external: false,
        });
    }
    Ok(Target::Internal(decode(value)?))
}
fn parts(value: &str) -> Result<(String, String), String> {
    let (name, search) = value.split_once("::").unwrap_or((value, ""));
    if name.is_empty() || search.contains("::") {
        return Err("Invalid or empty file/ID link target".into());
    }
    Ok((decode(name)?, decode(search)?))
}
fn unescape(value: &str) -> String {
    let mut chars = value.chars().peekable();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek().is_some_and(|c| "[]\\".contains(*c)) {
            out.push(chars.next().expect("escaped link character"));
        } else {
            out.push(c);
        }
    }
    out
}
/// Decode exact UTF-8 percent escapes once. No environment or shell expansion.
pub fn decode(value: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let mut at = 0;
    while at < value.len() {
        if value.as_bytes()[at] == b'%' {
            let pair = value
                .get(at + 1..at + 3)
                .ok_or("Incomplete percent escape")?;
            bytes.push(u8::from_str_radix(pair, 16).map_err(|_| "Invalid percent escape")?);
            at += 3;
        } else {
            bytes.push(value.as_bytes()[at]);
            at += 1;
        }
    }
    let text = String::from_utf8(bytes).map_err(|_| "Link path is not UTF-8")?;
    if text.chars().any(char::is_control) {
        return Err("Decoded link contains control characters".into());
    }
    Ok(text)
}
/// Encode a literal local path or target without changing its scalar bytes.
pub fn encode(value: &str) -> String {
    let mut result = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            result.push(char::from(b));
        } else {
            use std::fmt::Write;
            write!(result, "%{b:02X}").expect("write string");
        }
    }
    result
}
