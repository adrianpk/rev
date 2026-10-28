//! Immutable bounded TZif/POSIX observations, independent of source text.
mod observe;
pub use observe::{ZoneEnvironment, ZoneSelection};
mod posix;
mod tzif;
use posix::Posix;

#[derive(Clone, Debug)]
pub struct Zone {
    offsets: Vec<Option<i32>>,
    transitions: Vec<Transition>,
    future: Option<Posix>,
}
#[derive(Clone, Debug)]
struct Transition {
    at: i64,
    kind: usize,
}
impl Zone {
    /// Parse at most 1 MiB, 65536 transitions, 256 types and 4096 designation
    /// bytes. Leap-second TZif data is explicitly outside the Unix-time profile.
    pub fn from_tzif(bytes: &[u8]) -> Result<Self, String> {
        tzif::parse(bytes)
    }
    /// Parse a bounded POSIX zone with explicit DST rules, including signed
    /// transition hours. Implementation-defined implicit DST rules reject.
    pub fn from_posix(text: &str) -> Result<Self, String> {
        let future = Posix::parse(text, true)?;
        Ok(Self {
            offsets: vec![Some(future.standard)],
            transitions: Vec::new(),
            future: Some(future),
        })
    }
    pub fn utc() -> Self {
        Self {
            offsets: vec![Some(0)],
            transitions: Vec::new(),
            future: None,
        }
    }
    pub fn offset_at(&self, seconds: i64) -> Result<i32, String> {
        let index = self.transitions.partition_point(|t| t.at <= seconds);
        if index == self.transitions.len() {
            return self.after_transitions(seconds);
        }
        let kind = if index == 0 {
            0
        } else {
            self.transitions[index - 1].kind
        };
        self.known_offset(kind)
    }
    fn after_transitions(&self, seconds: i64) -> Result<i32, String> {
        if let Some(future) = &self.future {
            return future.offset_at(seconds);
        }
        if !self.transitions.is_empty() {
            return Err("Org time exceeds known TZif coverage".into());
        }
        self.known_offset(0)
    }
    fn known_offset(&self, kind: usize) -> Result<i32, String> {
        self.offsets[kind].ok_or_else(|| "TZif marks this local time as unspecified".into())
    }
}
