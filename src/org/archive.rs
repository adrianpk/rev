//! Native archive planning. Source fragments retain actual canonical spans;
//! destination observations and metadata do not confer publication authority.
mod location;
mod metadata;
use crate::{
    buffer::{Buffer, Buffers},
    config::{CaptureTarget, CaptureTemplate, Org},
    org::{
        Cancellation, Outline,
        agenda::{SourceScan, SourceSelection},
        capture::{Choice, Discovery},
        refile::{Destination, Plan, Subtree},
        time::Date,
    },
};
pub use location::location;
use ropey::Rope;
use std::collections::BTreeMap;

pub(crate) struct Context {
    pub subtree: Subtree,
    pub values: BTreeMap<String, String>,
    pub location: String,
    pub entry: usize,
}
pub(crate) struct Sibling {
    pub at: usize,
    pub level: usize,
    pub create: bool,
}
pub(crate) enum Archive {
    External {
        discovery: Box<Discovery>,
        subtree: Subtree,
        body: Rope,
    },
    Sibling {
        scan: Box<SourceScan>,
        subtree: Subtree,
        target: Sibling,
        newline: &'static str,
    },
}
impl Context {
    pub(crate) fn external(
        self,
        buffers: &Buffers,
        policy: &Org,
        date: Date,
        body: Rope,
    ) -> Result<Archive, String> {
        let from = buffers
            .get(self.subtree.stamp().buffer)
            .map_err(|e| e.to_string())?;
        let file = from
            .file()
            .ok_or("Save or associate the source file before external archiving")?;
        let (path, headline) = location::expand(&self.location, file.path())?;
        let mut template = CaptureTemplate::task();
        template.target = if headline.is_empty() {
            CaptureTarget::File {
                file: String::new(),
            }
        } else {
            CaptureTarget::Headline {
                file: String::new(),
                headline,
            }
        };
        let discovery = Choice {
            template,
            file: Some(path),
        }
        .discover(buffers, policy, date)?;
        Ok(Archive::External {
            discovery: Box::new(discovery),
            subtree: self.subtree,
            body,
        })
    }
    pub(crate) fn sibling(
        self,
        buffers: &Buffers,
        policy: &Org,
        target: Sibling,
    ) -> Result<Archive, String> {
        let source = buffers
            .get(self.subtree.stamp().buffer)
            .map_err(|e| e.to_string())?;
        let scan = SourceScan::new(
            buffers,
            SourceSelection {
                buffers: vec![source.id()],
                paths: Vec::new(),
            },
            &policy.limits,
        )?;
        Ok(Archive::Sibling {
            scan: Box::new(scan),
            subtree: self.subtree,
            target,
            newline: source.newline(),
        })
    }
}
impl Archive {
    pub(crate) fn stamp(&self) -> crate::buffer::Stamp {
        match self {
            Self::External { subtree, .. } | Self::Sibling { subtree, .. } => subtree.stamp(),
        }
    }
    pub(crate) fn run(self, cancel: &Cancellation) -> Result<Plan, String> {
        match self {
            Self::External {
                discovery,
                subtree,
                body,
            } => {
                let resolved = discovery.run(cancel)?;
                let insertion = resolved.entry_insertion(&body, cancel)?;
                let edit = insertion
                    .edits
                    .into_iter()
                    .next()
                    .ok_or("Archive insertion is empty")?;
                if edit.bytes.start != edit.bytes.end {
                    return Err("Archive destination must be an insertion".into());
                }
                let same = matches!(insertion.source.version, crate::org::io::DestinationVersion::Buffer(stamp) if stamp.buffer == subtree.stamp().buffer);
                if same {
                    return Err("External archive must use a different canonical document; use archive sibling".into());
                }
                Ok(Plan::prepared(
                    Destination::File(insertion.source),
                    subtree,
                    edit.bytes.start,
                    edit.text.trim_start_matches(['\r', '\n']).to_owned(),
                ))
            }
            Self::Sibling {
                scan,
                subtree,
                target,
                newline,
            } => sibling(*scan, subtree, target, newline, cancel),
        }
    }
}
fn sibling(
    scan: SourceScan,
    subtree: Subtree,
    target: Sibling,
    newline: &str,
    cancel: &Cancellation,
) -> Result<Plan, String> {
    let sources = scan.run(cancel);
    if !sources.complete() {
        return Err(sources.feedback());
    }
    let destination = sources
        .documents
        .into_iter()
        .next()
        .ok_or("Archive source unavailable")?;
    let mut body = String::new();
    if target.create {
        body.push_str(&format!(
            "{} Archive :ARCHIVE:{newline}",
            "*".repeat(target.level)
        ));
    }
    body.push_str(&crate::org::structure::entry::relevel(
        subtree.text(),
        target.level + 1,
        newline,
        cancel,
    )?);
    if !body.ends_with('\n') {
        body.push_str(newline);
    }
    Ok(Plan::prepared(
        Destination::Agenda(destination),
        subtree,
        target.at,
        body,
    ))
}
