use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use super::tables::{Alignment, Table};
use super::{Item, Kind, MAX_DEPTH, MAX_EVENTS, Node};
use crate::process::Cancellation;
use crate::syntax::MarkdownRole;

pub(super) struct Parsed {
    pub(super) nodes: Vec<Node>,
    pub(super) items: Vec<Item>,
    pub(super) tables: Vec<Table>,
}

pub(super) fn observe(source: &str, cancel: &Cancellation) -> Option<Parsed> {
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
    (!cancel.is_cancelled()).then_some(Parsed {
        nodes: scan.nodes,
        items: scan.items,
        tables: scan.tables,
    })
}

#[derive(Default)]
struct Scan {
    nodes: Vec<Node>,
    depth: usize,
    lists: Vec<usize>,
    items: Vec<Item>,
    open_items: Vec<usize>,
    tables: Vec<Table>,
    table: Option<TableSource>,
}

struct TableSource {
    bytes: Range<usize>,
    rows: Vec<Range<usize>>,
    alignments: Vec<Alignment>,
}

impl Scan {
    fn event(&mut self, event: &Event<'_>, bytes: Range<usize>, source: &str) -> bool {
        match event {
            Event::Start(tag) => self.start(tag, bytes, source),
            Event::End(tag) => {
                self.depth -= 1;
                if *tag == TagEnd::Table {
                    let table = self.table.take().expect("admitted table source");
                    self.tables.push(Table::observe(
                        source,
                        table.bytes,
                        &table.rows,
                        table.alignments,
                    ));
                }
                if *tag == TagEnd::List(false) || *tag == TagEnd::List(true) {
                    self.lists.pop();
                }
                if *tag == TagEnd::Item {
                    self.open_items.pop();
                }
                true
            }
            Event::Code(_) => {
                self.content();
                self.nodes.push(Node {
                    bytes,
                    kind: Kind::Literal,
                });
                true
            }
            _ => {
                self.content();
                true
            }
        }
    }

    fn start(&mut self, tag: &Tag<'_>, bytes: Range<usize>, source: &str) -> bool {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return false;
        }
        self.content();
        self.table_source(tag, &bytes);
        let kind = match tag {
            Tag::Heading { level, .. } => Some(Kind::Heading {
                level: *level as u8,
                prefix: heading_prefix(source, &bytes, *level as u8),
            }),
            Tag::List(_) => {
                self.lists.push(bytes.start);
                Some(Kind::List(self.lists.len()))
            }
            Tag::Item => {
                return self.item(source, bytes);
            }
            Tag::CodeBlock(_) => Some(Kind::LiteralBlock),
            _ => semantic(tag).map(Kind::Semantic),
        };
        if let Some(kind) = kind {
            self.nodes.push(Node { bytes, kind });
        }
        true
    }
    fn table_source(&mut self, tag: &Tag<'_>, bytes: &Range<usize>) {
        match tag {
            Tag::Table(alignments) => {
                self.table = Some(TableSource {
                    bytes: bytes.clone(),
                    rows: Vec::new(),
                    alignments: alignments
                        .iter()
                        .map(|alignment| match alignment {
                            pulldown_cmark::Alignment::None => Alignment::None,
                            pulldown_cmark::Alignment::Left => Alignment::Left,
                            pulldown_cmark::Alignment::Center => Alignment::Center,
                            pulldown_cmark::Alignment::Right => Alignment::Right,
                        })
                        .collect(),
                });
            }
            Tag::TableHead | Tag::TableRow => {
                self.table
                    .as_mut()
                    .expect("parser row belongs to table")
                    .rows
                    .push(bytes.clone());
            }
            _ => {}
        }
    }
    fn content(&mut self) {
        if let Some(index) = self.open_items.last() {
            self.items[*index].nonempty = true;
        }
    }

    fn item(&mut self, source: &str, bytes: Range<usize>) -> bool {
        let Some(item) = Item::parse(source, bytes, self.lists.len()) else {
            return false;
        };
        let mut item = item;
        item.parent = self.open_items.last().copied();
        item.group = *self
            .lists
            .last()
            .expect("an admitted item belongs to a list");
        self.open_items.push(self.items.len());
        self.items.push(item);
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
