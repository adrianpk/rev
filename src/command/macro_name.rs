//! Session command identity with the same bounded value semantics as AI names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MacroName {
    bytes: [u8; 64],
    len: u8,
}

impl MacroName {
    /// Validate syntax and the fixed command/action namespace. The session
    /// separately checks its currently admitted dynamic command identities.
    pub fn new(name: &str) -> Result<Self, String> {
        if name.is_empty()
            || name.len() > 64
            || !name.as_bytes()[0].is_ascii_lowercase()
            || !name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("Macro names require 1..=64 lowercase ASCII letters/digits/hyphens, starting with a letter".into());
        }
        if super::Binding::named(name).is_some() || name == "kmacro-bound-input" {
            return Err("Macro name collides with a compiled command or prompt action".into());
        }
        let mut value = Self {
            bytes: [0; 64],
            len: name.len() as u8,
        };
        value.bytes[..name.len()].copy_from_slice(name.as_bytes());
        Ok(value)
    }

    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)])
            .expect("validated ASCII macro name")
    }
}
