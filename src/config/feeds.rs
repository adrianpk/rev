//! Declared feed data and optional transport argv. Loading performs no IO.
use serde::Deserialize;
use std::collections::BTreeSet;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Feeds {
    pub sources: Vec<FeedSource>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialFeeds {
    pub sources: Option<Vec<FeedSource>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedSource {
    pub name: String,
    pub source: String,
    pub file: String,
    pub headline: String,
    #[serde(default = "drawer")]
    pub drawer: String,
    #[serde(default)]
    pub transport: Option<Vec<String>>,
}
fn drawer() -> String {
    "FEEDSTATUS".into()
}
pub(crate) fn validate(sources: &[FeedSource]) -> Result<(), String> {
    if sources.len() > 64 {
        return Err("Feeds permit at most 64 declared sources".into());
    }
    let mut names = BTreeSet::new();
    for source in sources {
        source.validate()?;
        if !names.insert(&source.name) {
            return Err("Feed names must be unique".into());
        }
    }
    Ok(())
}
impl FeedSource {
    pub(crate) fn validate(&self) -> Result<(), String> {
        text(&self.name, 128)?;
        text(&self.headline, 256)?;
        text(&self.source, 4096)?;
        text(&self.file, 4096)?;
        text(&self.drawer, 64)?;
        if !self
            .drawer
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b == b'_')
        {
            return Err("Feed drawer requires uppercase ASCII letters or underscore".into());
        }
        if let Some(argv) = &self.transport {
            argv_check(argv)?;
        }
        Ok(())
    }
    pub fn remote(&self) -> bool {
        self.source.starts_with("https://") || self.source.starts_with("http://")
    }
}
fn text(value: &str, max: usize) -> Result<(), String> {
    super::capture::path(value)?;
    if value.trim().is_empty() || value.len() > max {
        return Err(format!("Feed values require 1..{max} bytes"));
    }
    Ok(())
}
pub(crate) fn argv_check(argv: &[String]) -> Result<(), String> {
    if argv.len() > 64 {
        return Err("Feed transport permits at most 64 argv tokens".into());
    }
    for token in argv {
        text(token, 4096)?;
    }
    if argv.is_empty() {
        return Ok(());
    }
    substitutions(argv)
}
fn substitutions(argv: &[String]) -> Result<(), String> {
    let count = argv.iter().filter(|s| s.as_str() == "{target}").count();
    if count > 1 || argv[0].contains(['{', '}']) || count == 0 && argv.len() == 64 {
        return Err("Feed target must be one non-executable whole token or appended once".into());
    }
    for token in &argv[1..] {
        if token.contains(['{', '}']) && token != "{target}" {
            return Err("Feed transport admits whole {target} substitution only".into());
        }
    }
    Ok(())
}
