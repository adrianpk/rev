//! Native source observations. External parser types stay inside the adapter.

mod parse;

use std::ops::Range;

use crate::buffer::Stamp;
use crate::syntax::{Capture, MarkdownRole, Style};

pub(crate) const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_EVENTS: usize = 200_000;
const MAX_DEPTH: usize = 256;

/// Complete source ranges belong to exactly one canonical buffer revision.
pub(crate) struct Observation {
    pub(crate) stamp: Stamp,
    nodes: Vec<Node>,
}

struct Node {
    bytes: Range<usize>,
    kind: Kind,
}

enum Kind {
    Heading(u8),
    List(usize),
    Literal,
    Semantic(MarkdownRole),
}

impl Kind {
    fn role(&self) -> Option<MarkdownRole> {
        match *self {
            Self::Heading(level) => {
                assert!((1..=6).contains(&level), "admitted Markdown heading level");
                None
            }
            Self::List(depth) => {
                assert!(depth <= MAX_DEPTH, "admitted Markdown list depth");
                Some(MarkdownRole::List)
            }
            Self::Literal => Some(MarkdownRole::Code),
            Self::Semantic(role) => Some(role),
        }
    }
}

impl Observation {
    pub(crate) fn captures(&self, bytes: Range<usize>) -> Vec<Capture> {
        self.nodes
            .iter()
            .filter(|node| node.bytes.start < bytes.end && node.bytes.end > bytes.start)
            .filter_map(|node| {
                Some(Capture {
                    bytes: node.bytes.clone(),
                    style: Style::Markdown(node.kind.role()?),
                })
            })
            .collect()
    }
}

/// A cheap rope snapshot moves to the worker. Its one contiguous source copy is
/// bounded and created there, never during painting or canonical publication.
pub(crate) struct Work {
    stamp: Stamp,
    text: ropey::Rope,
}

impl Work {
    pub(crate) fn new(stamp: Stamp, text: ropey::Rope) -> Self {
        Self { stamp, text }
    }

    pub(crate) fn run(self, cancel: &crate::process::Cancellation) -> Option<Observation> {
        let cancel = cancel.bounded(std::time::Instant::now() + std::time::Duration::from_secs(2));
        if self.text.len_bytes() > MAX_BYTES || cancel.is_cancelled() {
            return None;
        }
        let mut source = String::with_capacity(self.text.len_bytes());
        for chunk in self.text.chunks() {
            if cancel.is_cancelled() {
                return None;
            }
            source.push_str(chunk);
        }
        let nodes = parse::observe(&source, &cancel)?;
        Some(Observation {
            stamp: self.stamp,
            nodes,
        })
    }
}
