//! Proposed result text has no edit authority until the source stamp is checked.
mod format;
mod previous;
use super::*;
use crate::{
    buffer::{Stamp, edit::Replacement},
    org::block::Headers,
};
use std::ops::Range;
#[derive(Clone, Debug)]
pub(crate) struct Site {
    stamp: Stamp,
    range: Range<usize>,
    policy: Policy,
    prefix: String,
    source_bytes: usize,
    old: String,
    newline: String,
    indent: String,
    inline: bool,
}
#[derive(Clone, Debug, Default)]
struct Policy {
    collection: Option<config::SourceResult>,
    table: bool,
    handling: Handling,
    format: String,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Handling {
    #[default]
    Replace,
    Append,
    Prepend,
    Silent,
}
impl Site {
    pub fn collection(headers: &Headers, result: config::SourceResult) -> Result<(), String> {
        let policy = Policy::parse(headers.get("results").unwrap_or("replace"))?;
        policy.admit(result)
    }
    pub fn stamp(&self) -> Stamp {
        self.stamp
    }
    pub fn block(scan: &Scan, definition: &Definition) -> Result<Self, String> {
        Self::new(
            scan,
            definition.end..definition.end,
            &definition.headers,
            definition.name.as_deref(),
        )
    }
    pub(in crate::org::execution) fn call(
        scan: &Scan,
        call: &super::input::calls::Call,
        definition: &Definition,
    ) -> Result<Self, String> {
        let mut headers = definition.headers.clone();
        headers.apply(&call.end_headers, &Cancellation::default())?;
        let mut end = Headers::default();
        end.apply(&call.end_headers, &Cancellation::default())?;
        if end.iter().any(|h| h.key != "results") {
            return Err("Call end headers support :results only".into());
        }
        let range = if call.inline {
            call.range.clone()
        } else {
            call.range.end..call.range.end
        };
        let mut site = Self::new(scan, range, &headers, Some(&call.name))?;
        site.inline = call.inline;
        if call.inline {
            site.inline_range(scan, call)?;
        }
        Ok(site)
    }
    fn new(
        scan: &Scan,
        anchor: Range<usize>,
        headers: &Headers,
        name: Option<&str>,
    ) -> Result<Self, String> {
        let text = scan.text();
        let line = text
            .line(text.byte_to_line(anchor.start.saturating_sub(1).min(text.len_bytes())))
            .to_string();
        let newline = if line.ends_with("\r\n") { "\r\n" } else { "\n" }.into();
        let indent: String = line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        let previous = previous::find(text, anchor.end, name)?;
        Ok(Self {
            stamp: scan.stamp(),
            range: previous.range,
            policy: Policy::parse(headers.get("results").unwrap_or("replace"))?,
            prefix: previous.prefix,
            source_bytes: text.len_bytes(),
            old: previous
                .body
                .lines()
                .map(|l| format!("{}\n", l.strip_prefix(&indent).unwrap_or(l)))
                .collect(),
            newline,
            indent,
            inline: false,
        })
    }
    fn inline_range(
        &mut self,
        scan: &Scan,
        call: &super::input::calls::Call,
    ) -> Result<(), String> {
        let tail = scan.text().byte_slice(call.range.end..).to_string();
        let end = if tail.starts_with(" {{{results(") {
            tail.find(")}}}")
                .map(|n| n + 4)
                .ok_or("Unclosed inline result macro")?
        } else {
            0
        };
        self.range = call.range.end..call.range.end + end;
        self.prefix.clear();
        self.old.clear();
        Ok(())
    }
    pub fn edit(&self, buffer: &Buffer, out: &Outcome) -> Result<Option<Replacement>, String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        if self.policy.handling == Handling::Silent {
            return Ok(None);
        }
        if buffer.is_readonly() {
            return Err("Source results publication is read-only".into());
        }
        let text = out
            .proposed
            .clone()
            .ok_or("Execution has no admitted result proposal")?;
        let source = buffer
            .byte_span(self.range.clone())
            .map_err(|e| e.to_string())?;
        Ok(Some(Replacement { source, text }))
    }
    pub fn prepare(&self, out: &Outcome, cancel: &Cancellation) -> Result<Option<String>, String> {
        super::check(cancel)?;
        if self.policy.handling == Handling::Silent {
            return Ok(None);
        }
        let value = format::render(&self.policy, out)?;
        let text = if self.inline {
            self.inline_text(&value)?
        } else {
            self.block_text(&value)
        };
        if self.source_bytes.saturating_sub(self.range.len()) + text.len()
            > crate::org::Limits::default().bytes
        {
            return Err("Updated results exceed native source byte limit".into());
        }
        super::check(cancel)?;
        Ok(Some(text))
    }
    fn inline_text(&self, value: &str) -> Result<String, String> {
        let value = value
            .trim_end_matches('\n')
            .strip_prefix(": ")
            .unwrap_or(value.trim());
        if value.contains(['\r', '\n', '=']) || value.contains(")}}}") || value.contains("{{{") {
            return Err("Inline results require a single verbatim scalar".into());
        }
        Ok(format!(" {{{{{{results(={value}=)}}}}}}"))
    }
    fn block_text(&self, value: &str) -> String {
        let body = match self.policy.handling {
            Handling::Append => format!("{}{value}", self.old),
            Handling::Prepend => format!("{value}{}", self.old),
            _ => value.into(),
        };
        let body: String = body
            .lines()
            .map(|l| format!("{}{l}{}", self.indent, self.newline))
            .collect();
        let prefix: String = self
            .prefix
            .lines()
            .map(|l| {
                if l.is_empty() {
                    self.newline.clone()
                } else {
                    format!("{}{l}{}", self.indent, self.newline)
                }
            })
            .collect();
        format!("{prefix}{body}")
    }
    pub fn admit(&self, result: config::SourceResult) -> Result<(), String> {
        self.policy.admit(result)
    }
}
impl Policy {
    fn admit(&self, result: config::SourceResult) -> Result<(), String> {
        if self
            .collection
            .is_some_and(|r| r != result && result != config::SourceResult::Table)
        {
            return Err(
                "Source :results collection requires a matching declared program result adapter"
                    .into(),
            );
        }
        Ok(())
    }
    fn parse(value: &str) -> Result<Self, String> {
        let mut p = Self::default();
        let mut handling = false;
        for word in value.split_whitespace() {
            p.option(word, &mut handling)?;
        }
        Ok(p)
    }
    fn option(&mut self, word: &str, handling: &mut bool) -> Result<(), String> {
        if word == "output" {
            return self.collect(config::SourceResult::Output);
        }
        if word == "value" {
            return self.collect(config::SourceResult::Value);
        }
        if ["table", "vector"].contains(&word) {
            self.table = true;
            return Ok(());
        }
        if ["replace", "append", "prepend", "silent", "none", "discard"].contains(&word) {
            return self.handling(word, handling);
        }
        self.format(word)
    }
    fn handling(&mut self, word: &str, seen: &mut bool) -> Result<(), String> {
        if *seen {
            return Err("Conflicting result handling".into());
        }
        *seen = true;
        self.handling = match word {
            "append" => Handling::Append,
            "prepend" => Handling::Prepend,
            "replace" => Handling::Replace,
            _ => Handling::Silent,
        };
        Ok(())
    }
    fn format(&mut self, word: &str) -> Result<(), String> {
        if ![
            "scalar", "verbatim", "drawer", "html", "latex", "org", "code",
        ]
        .contains(&word)
        {
            return Err(format!("Unsupported native :results option {word}"));
        }
        if !self.format.is_empty() {
            return Err("Conflicting result formats".into());
        }
        self.format = word.into();
        Ok(())
    }
    fn collect(&mut self, value: config::SourceResult) -> Result<(), String> {
        if self.collection.is_some() {
            return Err("Conflicting result collection".into());
        }
        self.collection = Some(value);
        Ok(())
    }
}
