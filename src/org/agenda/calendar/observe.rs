use super::*;
use crate::org::{
    Coverage, InlineContext, Limits, StructureKind,
    structure::{self, Raw},
    task::Workflow,
};
use ropey::Rope;
use std::collections::BTreeMap;
struct Header {
    title: Arc<str>,
    byte: usize,
    done: bool,
    excluded: bool,
}
pub(super) fn sources(sources: &SourceSet, policy: &Org, cancel: &Cancellation) -> Facts {
    let mut facts = Facts {
        events: Vec::new(),
        sources: sources
            .documents
            .iter()
            .map(|s| CalendarSource {
                path: s.path.clone(),
                version: s.version.clone(),
            })
            .collect(),
        issues: sources.issues.clone(),
        truncated: sources.truncated || sources.cancelled,
        limit: policy.limits.agenda_rows,
        seconds: policy.limits.job_seconds,
    };
    for (id, source) in sources.documents.iter().enumerate() {
        if cancel.is_cancelled() {
            break;
        }
        let text = source.snapshot();
        let document = structure::observe(text.clone(), Limits::default(), cancel);
        if document.coverage != Coverage::Complete {
            issue(
                &mut facts,
                id,
                "Incomplete structural source observation".into(),
            );
        }
        match workflow(&document.nodes, &text, &policy.todo) {
            Ok(workflow) => document_events(
                &mut facts,
                id,
                (&document.nodes, &text),
                (&workflow, policy.agenda.include_archived),
                cancel,
            ),
            Err(error) => issue(&mut facts, id, error),
        }
        if facts.truncated && facts.events.len() == facts.limit {
            break;
        }
    }
    facts.truncated |= cancel.is_cancelled();
    facts
}
fn workflow(nodes: &[Raw], text: &Rope, policy: &crate::config::Todo) -> Result<Workflow, String> {
    let mut workflow = Workflow::default();
    for node in nodes.iter().filter(|n| n.kind == StructureKind::Keyword) {
        if let Some(name) = &node.name {
            let key = bounded(text, name.clone())?;
            if ["TODO", "SEQ_TODO", "TYP_TODO"]
                .iter()
                .any(|k| key.eq_ignore_ascii_case(k))
            {
                workflow.declaration(&bounded(text, node.content.clone())?);
            }
        }
    }
    workflow.effective(policy)
}
fn document_events(
    facts: &mut Facts,
    source: usize,
    data: (&[Raw], &Rope),
    context: (&Workflow, bool),
    cancel: &Cancellation,
) {
    let (nodes, text) = data;
    let (workflow, include_archived) = context;
    let mut owners = BTreeMap::<usize, Header>::new();
    let mut ancestry = Vec::<Option<usize>>::new();
    for (id, node) in nodes.iter().enumerate() {
        if cancel.is_cancelled() {
            break;
        }
        if node.kind == StructureKind::Malformed {
            issue(facts, source, "Malformed Org structure in source".into());
        }
        let parent = node.parent.and_then(|p| ancestry.get(p).copied().flatten());
        let owner = if matches!(node.kind, StructureKind::Heading { .. }) {
            match header(
                node,
                text,
                workflow,
                parent.and_then(|p| owners.get(&p)),
                include_archived,
            ) {
                Ok(h) => {
                    owners.insert(id, h);
                    Some(id)
                }
                Err(e) => {
                    issue(facts, source, e);
                    None
                }
            }
        } else {
            parent
        };
        ancestry.push(owner);
        if let Some(header) = owner.and_then(|p| owners.get(&p)).filter(|h| !h.excluded)
            && let Err(error) = node_events(facts, (source, node), text, header, cancel)
        {
            issue(facts, source, error);
        }
        if facts.events.len() == facts.limit && facts.truncated {
            break;
        }
        std::thread::yield_now();
    }
}
fn header(
    node: &Raw,
    text: &Rope,
    workflow: &Workflow,
    parent: Option<&Header>,
    include_archived: bool,
) -> Result<Header, String> {
    let title = bounded(text, node.content.clone())?;
    let word = title.split_whitespace().next().unwrap_or("");
    let archived = title
        .split_whitespace()
        .last()
        .is_some_and(|tags| tags.split(':').any(|t| t == "ARCHIVE"));
    Ok(Header {
        byte: node.source.start,
        done: workflow.state(word).is_some_and(|s| s.done),
        excluded: word == "COMMENT"
            || archived && !include_archived
            || parent.is_some_and(|h| h.excluded),
        title: Arc::from(title.trim()),
    })
}
fn node_events(
    facts: &mut Facts,
    origin: (usize, &Raw),
    text: &Rope,
    header: &Header,
    cancel: &Cancellation,
) -> Result<(), String> {
    let (source, node) = origin;
    if node.kind == StructureKind::Planning {
        return planning(
            facts,
            source,
            &bounded(text, node.content.clone())?,
            header,
            cancel,
        );
    }
    if node.context != Some(InlineContext::Standard) {
        return Ok(());
    }
    let value = text.byte_slice(node.content.clone()).to_string();
    timestamps(
        &value,
        |stamp| add(facts, source, header, stamp, CalendarKind::Appointment),
        cancel,
    )
}
fn planning(
    facts: &mut Facts,
    source: usize,
    text: &str,
    header: &Header,
    cancel: &Cancellation,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("Calendar observation cancelled or timed out".into());
    }
    for (name, kind) in [
        ("SCHEDULED:", CalendarKind::Scheduled),
        ("DEADLINE:", CalendarKind::Deadline),
    ] {
        let mut matches = text.match_indices(name);
        if let Some((at, _)) = matches.next() {
            if matches.next().is_some() {
                return Err(format!("Duplicate Org {name} source"));
            }
            let value = text[at + name.len()..].trim_start();
            let (value, _) = prefix(value)?;
            if value.start.active() {
                add(facts, source, header, value, kind)?;
            }
        }
    }
    Ok(())
}
fn timestamps(
    text: &str,
    mut add: impl FnMut(crate::org::time::TimestampRange) -> Result<(), String>,
    cancel: &Cancellation,
) -> Result<(), String> {
    let mut at = 0;
    while let Some(offset) = text[at..].find('<') {
        if cancel.is_cancelled() {
            return Err("Calendar observation cancelled or timed out".into());
        }
        let start = at + offset;
        let tail = &text[start..];
        if tail.as_bytes().get(1).is_some_and(u8::is_ascii_digit) {
            let (value, length) = prefix(tail)?;
            add(value)?;
            at = start + length;
        } else {
            at = start + 1;
        }
    }
    Ok(())
}
fn prefix(text: &str) -> Result<(crate::org::time::TimestampRange, usize), String> {
    let close = match text.as_bytes().first() {
        Some(b'<') => '>',
        Some(b'[') => ']',
        _ => return Err("Expected Org planning timestamp".into()),
    };
    let mut length = text.find(close).ok_or("Unclosed Org timestamp")? + 1;
    if text[length..].starts_with("--") {
        let tail = &text[length + 2..];
        length += 2 + tail.find(close).ok_or("Unclosed Org timestamp range")? + 1;
    }
    Ok((
        crate::org::time::TimestampRange::parse(&text[..length])?,
        length,
    ))
}

fn add(
    facts: &mut Facts,
    source: usize,
    header: &Header,
    value: crate::org::time::TimestampRange,
    kind: CalendarKind,
) -> Result<(), String> {
    if facts.events.len() >= facts.limit {
        facts.truncated = true;
        return Err("Calendar source fact limit reached".into());
    }
    facts.events.push(Event {
        source,
        byte: header.byte,
        title: header.title.clone(),
        done: header.done,
        value,
        kind,
    });
    Ok(())
}
fn bounded(text: &Rope, range: std::ops::Range<usize>) -> Result<String, String> {
    if range.len() > 4096 {
        return Err("Org agenda heading or metadata exceeds 4096 bytes".into());
    }
    Ok(text.byte_slice(range).to_string())
}
fn issue(facts: &mut Facts, source: usize, message: String) {
    if facts.issues.len() < 4096 {
        facts.issues.push(SourceIssue {
            path: facts.sources[source].path.clone(),
            message,
        });
    } else {
        facts.truncated = true;
    }
}
