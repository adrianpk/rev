mod edit;
mod matching;
mod model;
mod priority;
mod tasks;
mod timestamps;
pub(crate) use tasks::TaskChange;

use super::Outline;
use crate::buffer::{Buffer, edit::Replacement};
use crate::coordinates::{Coordinates, Scalar};
use crate::org::{Cancellation, Structure};
use std::collections::BTreeSet;
use std::ops::Range;

#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::org) struct Tags {
    complete: bool,
    headers: Vec<Header>,
    file: Vec<Tag>,
    candidates: BTreeSet<String>,
    contexts: Vec<Range<usize>>,
    count: usize,
    workflow: crate::org::task::Workflow,
    metadata: tasks::Metadata,
    priorities: Option<Result<crate::org::priority::Priority, String>>,
    repeat_log: Option<crate::org::task::Log>,
}
#[derive(Debug, PartialEq, Eq)]
struct Header {
    span: Range<usize>,
    valid: bool,
    title: Range<usize>,
    word: String,
    words: Vec<(Range<usize>, String)>,
    local: Vec<Tag>,
}
#[derive(Debug, PartialEq, Eq)]
struct Tag {
    name: String,
    source: Range<usize>,
}

pub(crate) fn valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 128
        && tag
            .chars()
            .all(|c| c.is_alphanumeric() || "_@#%".contains(c))
}
fn names(draft: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    for tag in draft
        .split(|c: char| c == ':' || c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
    {
        if !valid_tag(tag) {
            return Err(
                "Org tags must use letters, numbers, _, @, # or % (at most 128 UTF-8 bytes)".into(),
            );
        }
        if !result.iter().any(|name| name == tag) {
            result.push(tag.into());
        }
        if result.len() > 128 {
            return Err("Org heading exceeds 128 local tags".into());
        }
    }
    Ok(result)
}
impl Outline {
    pub(crate) fn tag_target(&self, buffer: &Buffer, point: Scalar) -> Result<usize, String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        if !self.tags.complete {
            return Err("Org tag observation is incomplete".into());
        }
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        let index = self
            .heading(byte)
            .ok_or("No current Org heading for tags")?;
        let admitted = self.tags.contexts.iter().any(|r| r.contains(&byte));
        if !admitted && self.unavailable(byte, byte == buffer.len_bytes(), byte > 0) {
            return Err("Org tags are unavailable in this source context".into());
        }
        if !self.tags.headers[index].valid {
            return Err("Malformed or excessive Org heading tags; correct source first".into());
        }
        Ok(index)
    }
    pub(crate) fn tag_header_at(&self, buffer: &Buffer, point: Scalar) -> bool {
        let Ok(c) = Coordinates::new(buffer) else {
            return false;
        };
        let Ok(byte) = c.to_byte(point) else {
            return false;
        };
        self.headings
            .iter()
            .any(|h| h.start <= byte.0 && byte.0 < h.body)
    }
    pub(crate) fn tag_edit(
        &self,
        buffer: &Buffer,
        index: usize,
        draft: &str,
        remove: bool,
    ) -> Result<Vec<Replacement>, String> {
        self.tags.edit(buffer, index, draft, remove)
    }
    pub(crate) fn tag_membership(&self, index: usize, inherit: bool) -> Vec<String> {
        self.tag_contributions(index, inherit)
            .into_iter()
            .map(|(tag, _)| tag.name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    fn tag_contributions(&self, index: usize, inherit: bool) -> Vec<(&Tag, Option<usize>)> {
        let mut result = Vec::new();
        let mut owner = Some(index);
        while let Some(current) = owner {
            result.extend(
                self.tags.headers[current]
                    .local
                    .iter()
                    .map(|t| (t, Some(current))),
            );
            owner = if inherit {
                self.headings[current].parent
            } else {
                None
            };
        }
        if inherit {
            result.extend(self.tags.file.iter().map(|t| (t, None)));
        }
        result
    }
    pub(crate) fn tag_inspection(
        &self,
        buffer: &Buffer,
        index: usize,
        inherit: bool,
    ) -> Result<String, String> {
        let mut parts = Vec::new();
        for (tag, owner) in self.tag_contributions(index, inherit).iter().take(32) {
            let kind = match owner {
                Some(i) if *i == index => "local",
                Some(_) => "inherited",
                None => "file",
            };
            let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
            let point = c
                .from_byte(crate::coordinates::Byte(tag.source.start))
                .map_err(|e| e.to_string())?;
            let row = c.to_line(point).map_err(|e| e.to_string())?.row + 1;
            parts.push(format!("{kind} [L{row}] {}", tag.name));
        }
        Ok(if parts.is_empty() {
            "Org tags: none".into()
        } else {
            format!("Org tags: {}", parts.join("; "))
        })
    }
    pub(crate) fn tag_completion(
        &self,
        draft: &str,
        cursor: usize,
    ) -> Result<(String, String), String> {
        let start = draft[..cursor]
            .rfind(|c: char| c == ':' || c == ',' || c.is_whitespace())
            .map_or(0, |n| {
                n + draft[n..].chars().next().expect("separator").len_utf8()
            });
        let prefix = &draft[start..cursor];
        let candidates: Vec<_> = self
            .tags
            .candidates
            .iter()
            .filter(|s| s.starts_with(prefix))
            .collect();
        let Some(first) = candidates.first() else {
            return Err("No matching Org tag".into());
        };
        let mut common = first.as_str();
        for candidate in &candidates {
            let shared: usize = common
                .chars()
                .zip(candidate.chars())
                .take_while(|(a, b)| a == b)
                .map(|(c, _)| c.len_utf8())
                .sum();
            common = &common[..shared];
        }
        let text = format!("{}{}{}", &draft[..start], common, &draft[cursor..]);
        let shown = candidates
            .iter()
            .take(16)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        Ok((
            text,
            format!("{} tag candidates: {shown}", candidates.len()),
        ))
    }
}
