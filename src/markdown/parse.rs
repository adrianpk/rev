use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use super::{Kind, MAX_DEPTH, MAX_EVENTS, Node};
use crate::process::Cancellation;
use crate::syntax::MarkdownRole;

pub(super) fn observe(source: &str, cancel: &Cancellation) -> Option<Vec<Node>> {
    if cancel.is_cancelled() {
        return None;
    }
    let parser = Parser::new_ext(source, Options::ENABLE_TABLES);
    if cancel.is_cancelled() {
        return None;
    }
    let mut scan = Scan::default();
    for (count, (event, bytes)) in parser.into_offset_iter().enumerate() {
        if count >= MAX_EVENTS || cancel.is_cancelled() || !scan.event(&event, bytes, source) {
            return None;
        }
    }
    (!cancel.is_cancelled()).then_some(scan.nodes)
}

#[derive(Default)]
struct Scan {
    nodes: Vec<Node>,
    depth: usize,
    lists: usize,
}

impl Scan {
    fn event(&mut self, event: &Event<'_>, bytes: Range<usize>, source: &str) -> bool {
        match event {
            Event::Start(tag) => self.start(tag, bytes, source),
            Event::End(tag) => {
                self.depth -= 1;
                if *tag == TagEnd::List(false) || *tag == TagEnd::List(true) {
                    self.lists -= 1;
                }
                true
            }
            Event::Code(_) => {
                self.nodes.push(Node {
                    bytes,
                    kind: Kind::Literal,
                });
                true
            }
            _ => true,
        }
    }

    fn start(&mut self, tag: &Tag<'_>, bytes: Range<usize>, source: &str) -> bool {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return false;
        }
        let kind = match tag {
            Tag::Heading { level, .. } => Some(Kind::Heading {
                level: *level as u8,
                prefix: heading_prefix(source, &bytes, *level as u8),
            }),
            Tag::List(_) => {
                self.lists += 1;
                Some(Kind::List(self.lists))
            }
            Tag::CodeBlock(_) => Some(Kind::Literal),
            _ => semantic(tag).map(Kind::Semantic),
        };
        if let Some(kind) = kind {
            self.nodes.push(Node { bytes, kind });
        }
        true
    }
}

fn semantic(tag: &Tag<'_>) -> Option<MarkdownRole> {
    match tag {
        Tag::Emphasis | Tag::Strong => Some(MarkdownRole::Emphasis),
        Tag::Link { .. } | Tag::Image { .. } => Some(MarkdownRole::Link),
        Tag::BlockQuote(_) => Some(MarkdownRole::Quote),
        Tag::Table(_) => Some(MarkdownRole::Table),
        _ => None,
    }
}

// Only a parser-admitted heading can supply a prefix. Setext starts at title
// text; its underline never enters the substitution range.
fn heading_prefix(source: &str, bytes: &Range<usize>, level: u8) -> Option<Range<usize>> {
    let end = bytes.start.checked_add(usize::from(level))?;
    if source
        .as_bytes()
        .get(end)
        .is_some_and(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
    {
        return None;
    }
    let prefix = source.get(bytes.start..end)?;
    prefix
        .bytes()
        .all(|byte| byte == b'#')
        .then_some(bytes.start..end)
}
