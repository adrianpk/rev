//! Bounded value identity: named commands remain Copy without a global registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AiName {
    bytes: [u8; 67],
    len: u8,
}
impl AiName {
    /// Validate a definition key and reserve the common command/action namespace.
    pub fn new(name: &str) -> Result<Self, String> {
        if name.is_empty()
            || name.len() > 64
            || !name.as_bytes()[0].is_ascii_lowercase()
            || !name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("AI names require 1..=64 lowercase ASCII letters/digits/hyphens, starting with a letter".into());
        }
        let mut value = Self {
            bytes: [0; 67],
            len: u8::try_from(name.len() + 3).expect("AI command names fit 67 bytes"),
        };
        value.bytes[..3].copy_from_slice(b"ai-");
        value.bytes[3..usize::from(value.len)].copy_from_slice(name.as_bytes());
        if super::Binding::named(value.name()).is_some() {
            return Err("AI name collides with a compiled command or prompt action".into());
        }
        Ok(value)
    }
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).expect("validated ASCII name")
    }
    pub fn key(&self) -> &str {
        &self.name()[3..]
    }
}
