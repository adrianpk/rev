//! The approved single-character native priority profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Priority {
    pub highest: u8,
    pub lowest: u8,
    pub default: u8,
}
impl Priority {
    pub fn new(highest: &str, lowest: &str, default: &str) -> Result<Self, String> {
        let value = Self {
            highest: letter(highest)?,
            lowest: letter(lowest)?,
            default: letter(default)?,
        };
        if value.highest > value.default || value.default > value.lowest {
            return Err("Org priority requires highest <= default <= lowest".into());
        }
        Ok(value)
    }
    pub fn configured(policy: &crate::config::Priority) -> Result<Self, String> {
        Self::new(&policy.highest, &policy.lowest, &policy.default)
    }
    pub fn admit(self, draft: &str) -> Result<Option<u8>, String> {
        if draft.trim().is_empty() {
            return Ok(None);
        }
        let value = letter(draft.trim())?;
        if !(self.highest..=self.lowest).contains(&value) {
            return Err("Org priority is outside the configured range".into());
        }
        Ok(Some(value))
    }
    pub fn step(self, current: Option<u8>, up: bool) -> Option<u8> {
        match current {
            None => Some(if up { self.lowest } else { self.highest }),
            Some(v) if up => (v > self.highest).then(|| v - 1),
            Some(v) => (v < self.lowest).then(|| v + 1),
        }
    }
}
fn letter(text: &str) -> Result<u8, String> {
    if text.len() == 1 && text.as_bytes()[0].is_ascii_uppercase() {
        Ok(text.as_bytes()[0])
    } else {
        Err("Org priority requires one uppercase A-Z character".into())
    }
}
