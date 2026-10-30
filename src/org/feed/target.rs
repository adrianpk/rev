//! One complete feed/status insertion in retained canonical target coordinates.
use super::*;
use crate::{
    config::{CaptureKind, CaptureTarget, CaptureTemplate, FeedSource, Org},
    org::{
        Coverage, Limits, StructureKind,
        capture::{CaptureEdit, Insertion, Target},
        io::DestinationSource,
        structure,
        time::Date,
    },
};
use std::ops::Range;
pub struct Plan {
    pub insertion: Insertion,
    pub added: usize,
    pub skipped: usize,
    pub point: usize,
    pub(crate) evidence: Vec<DestinationSource>,
}
pub(super) fn resolve(
    source: &DestinationSource,
    feed: &FeedSource,
    policy: &Org,
    cancel: &Cancellation,
) -> Result<Target, String> {
    let template = CaptureTemplate {
        key: "feed".into(),
        description: "Feed inbox".into(),
        kind: CaptureKind::Entry,
        target: CaptureTarget::Headline {
            file: feed.file.clone(),
            headline: feed.headline.clone(),
        },
        template: String::new(),
        prepend: false,
    };
    Target::resolve(source, &template, policy, Date::new(2000, 1, 1)?, cancel)
}
pub(super) fn plan(
    source: DestinationSource,
    target: Target,
    feed: &FeedSource,
    items: Feed,
    cancel: &Cancellation,
) -> Result<Plan, String> {
    let newline = newline(&source);
    let previous = drawer(&source, &target, &feed.drawer, cancel)?;
    let mut status = if let Some((_, value)) = &previous {
        super::status::Status::parse(value, cancel)?
    } else {
        super::status::Status::default()
    };
    let mut added = 0;
    let mut children = String::new();
    for item in &items.entries {
        xml::check(cancel)?;
        if status.handled(&item.id) {
            continue;
        }
        status.add(&item.id)?;
        children.push_str(&entry(item, target.level + 1, newline));
        added += 1;
    }
    let change = StatusChange {
        name: &feed.drawer,
        status: &status,
        old: previous,
    };
    let mut edits = change.edits(&source, &target, newline)?;
    append_entries(&source, &target, &mut edits, &children, newline);
    let inserted: usize = edits.iter().map(|e| e.text.len()).sum();
    let removed: usize = edits.iter().map(|e| e.bytes.len()).sum();
    if source.text().len_bytes() - removed + inserted > Limits::default().bytes {
        return Err("Imported feed exceeds native Org source byte limit".into());
    }
    let point = inbox_point(&source, &target, newline);
    Ok(Plan {
        insertion: Insertion { source, edits },
        added,
        skipped: items.entries.len() - added,
        point,
        evidence: Vec::new(),
    })
}
fn newline(source: &DestinationSource) -> &'static str {
    if source
        .text()
        .lines()
        .any(|l| l.to_string().ends_with("\r\n"))
    {
        "\r\n"
    } else {
        "\n"
    }
}
fn drawer(
    source: &DestinationSource,
    target: &Target,
    name: &str,
    cancel: &Cancellation,
) -> Result<Option<(Range<usize>, String)>, String> {
    if !target.missing.is_empty() {
        return Ok(None);
    }
    let document = structure::observe(source.snapshot(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete {
        return Err("Incomplete feed target structure".into());
    }
    let mut found = None;
    for node in &document.nodes {
        xml::check(cancel)?;
        if node.kind != StructureKind::Drawer
            || node.source.start < target.body.start.0
            || node.source.end > target.body.end.0
        {
            continue;
        }
        let Some(open) = &node.opening else {
            continue;
        };
        if !source
            .text()
            .byte_slice(open.clone())
            .to_string()
            .trim()
            .eq_ignore_ascii_case(&format!(":{name}:"))
        {
            continue;
        }
        if found.is_some() {
            return Err("Feed status drawer is ambiguous".into());
        }
        found = Some((
            node.content.clone(),
            source.text().byte_slice(node.content.clone()).to_string(),
        ));
    }
    Ok(found)
}
struct StatusChange<'a> {
    name: &'a str,
    status: &'a super::status::Status,
    old: Option<(Range<usize>, String)>,
}
impl StatusChange<'_> {
    fn edits(
        self,
        source: &DestinationSource,
        target: &Target,
        newline: &str,
    ) -> Result<Vec<CaptureEdit>, String> {
        let Self { name, status, old } = self;
        let value = status.render().replace('\n', newline);
        if let Some((bytes, previous)) = old {
            if value == previous {
                return Ok(Vec::new());
            }
            return Ok(vec![CaptureEdit { bytes, text: value }]);
        }
        let mut text = String::new();
        for heading in &target.missing {
            text.push_str(&format!(
                "{} {}{newline}",
                "*".repeat(heading.level),
                heading.title
            ));
        }
        text.push_str(&format!("  :{name}:{newline}{value}  :END:{newline}"));
        let at = if target.missing.is_empty() {
            target.body.end.0
        } else {
            target.subtree_end.0
        };
        separate(source, at, &mut text, newline);
        Ok(vec![CaptureEdit {
            bytes: at..at,
            text,
        }])
    }
}
fn append_entries(
    source: &DestinationSource,
    target: &Target,
    edits: &mut Vec<CaptureEdit>,
    children: &str,
    newline: &str,
) {
    if children.is_empty() {
        return;
    }
    let at = target.subtree_end.0;
    if let Some(edit) = edits
        .iter_mut()
        .find(|e| e.bytes.start == at && e.bytes.is_empty())
    {
        edit.text.push_str(children);
        return;
    }
    let mut text = children.into();
    separate(source, at, &mut text, newline);
    edits.push(CaptureEdit {
        bytes: at..at,
        text,
    });
}
fn separate(source: &DestinationSource, at: usize, text: &mut String, newline: &str) {
    if at > 0 && source.text().byte(at - 1) != b'\n' {
        text.insert_str(0, newline);
    }
}
fn inbox_point(source: &DestinationSource, target: &Target, newline: &str) -> usize {
    if let Some(parent) = target.parent {
        return parent.0;
    }
    let at = target.subtree_end.0;
    let separator = !target.missing.is_empty() && at > 0 && source.text().byte(at - 1) != b'\n';
    at + usize::from(separator) * newline.len()
}
fn entry(item: &Entry, level: usize, newline: &str) -> String {
    let mut text = format!("{} {}{newline}", "*".repeat(level), item.title);
    if let Some(link) = &item.link {
        let link = link.replace('[', "%5B").replace(']', "%5D");
        if link.starts_with("https://") || link.starts_with("http://") {
            text.push_str(&format!("[[{link}]]{newline}"));
        } else {
            text.push_str(&format!(": Link: {link}{newline}"));
        }
    }
    if let Some(date) = &item.published {
        text.push_str(&format!(
            ": Published: {}{newline}",
            date.split_whitespace().collect::<Vec<_>>().join(" ")
        ));
    }
    for line in item.body.lines() {
        text.push_str(&format!(": {line}{newline}"));
    }
    text
}

pub(super) fn admit(
    source: &DestinationSource,
    target: &Target,
    name: &str,
    cancel: &Cancellation,
) -> Result<(), String> {
    if let Some((_, value)) = drawer(source, target, name, cancel)? {
        super::status::Status::parse(&value, cancel)?;
    }
    Ok(())
}
pub(super) fn inbox(
    source: DestinationSource,
    target: Target,
    cancel: &Cancellation,
) -> Result<Plan, String> {
    xml::check(cancel)?;
    let newline = newline(&source);
    let at = target.subtree_end.0;
    let mut text = String::new();
    for heading in &target.missing {
        text.push_str(&format!(
            "{} {}{newline}",
            "*".repeat(heading.level),
            heading.title
        ));
    }
    separate(&source, at, &mut text, newline);
    let point = inbox_point(&source, &target, newline);
    let edits = if text.is_empty() {
        Vec::new()
    } else {
        vec![CaptureEdit {
            bytes: at..at,
            text,
        }]
    };
    Ok(Plan {
        insertion: Insertion { source, edits },
        added: 0,
        skipped: 0,
        point,
        evidence: Vec::new(),
    })
}
impl Plan {
    pub(crate) fn current(&self, buffers: &crate::buffer::Buffers) -> Result<(), String> {
        for source in &self.evidence {
            source.prepare_visit(buffers)?;
        }
        Ok(())
    }
}
