use std::ops::Range;
use std::time::{Duration, Instant};

use crate::org::structure::Raw;
use crate::org::{
    BlockKind, Cancellation, Coverage, InlineContext, InlineKind, InlineScan, Limit, Limits,
    Structure, StructureKind,
};
use crate::syntax::{Capture, Style};

use super::{Highlights, Role};

struct Painter<'a> {
    text: &'a ropey::Rope,
    result: &'a mut Highlights,
    cancellation: &'a Cancellation,
    limits: Limits,
    yielded: Instant,
}

pub(super) fn project(
    text: &ropey::Rope,
    structure: &Structure,
    result: &mut Highlights,
    limits: Limits,
    cancellation: &Cancellation,
) {
    let mut painter = Painter {
        text,
        result,
        limits,
        cancellation,
        yielded: Instant::now(),
    };
    for node in structure.raw_nodes() {
        if !painter.tick() || !painter.node(node) {
            break;
        }
    }
    painter.finish();
}

impl Painter<'_> {
    fn finish(&mut self) {
        // Sorting has a finite capture bound; cancellation is checked on both
        // sides and throughout the prefix table before any result is exposed.
        if !self.tick() {
            self.result.captures.clear();
            return;
        }
        self.result
            .captures
            .sort_by_key(|capture| capture.bytes.start);
        let mut end = 0;
        for index in 0..self.result.captures.len() {
            if !self.tick() {
                self.result.captures.clear();
                self.result.ends.clear();
                return;
            }
            end = end.max(self.result.captures[index].bytes.end);
            self.result.ends.push(end);
        }
    }

    fn tick(&mut self) -> bool {
        if self.cancellation.is_cancelled() {
            self.result.coverage = Coverage::Cancelled;
            return false;
        }
        if self.yielded.elapsed() >= Duration::from_millis(10) {
            std::thread::yield_now();
            self.yielded = Instant::now();
        }
        true
    }

    fn push(&mut self, range: Range<usize>, role: Role) -> bool {
        if range.is_empty() {
            return true;
        }
        if self.result.captures.len() == self.limits.nodes {
            self.result.coverage = Coverage::Incomplete(Limit::SourceNodes);
            return false;
        }
        self.result.captures.push(Capture {
            bytes: range,
            style: Style::Org(role),
        });
        true
    }

    fn node(&mut self, node: &Raw) -> bool {
        if !self.structural(node) {
            return false;
        }
        let Some(context) = node.context else {
            return true;
        };
        if context == InlineContext::Literal {
            return true;
        }
        let remaining = self.limits.nodes - self.result.captures.len();
        if remaining == 0 {
            self.result.coverage = Coverage::Incomplete(Limit::SourceNodes);
            return false;
        }
        let limits = Limits {
            nodes: remaining,
            ..self.limits
        };
        let scan = InlineScan::from_snapshot(
            self.text.clone(),
            self.result.stamp,
            node.content.clone(),
            context,
            limits,
        );
        let inline = scan.run(self.cancellation);
        for (kind, source) in inline.raw_styles() {
            if !self.tick() || !self.push(source, inline_role(kind)) {
                return false;
            }
        }
        if inline.coverage() != Coverage::Complete {
            self.result.coverage = inline.coverage();
            return false;
        }
        true
    }

    fn structural(&mut self, node: &Raw) -> bool {
        match node.kind {
            StructureKind::Heading { .. } => self.optional(&node.opening, Role::Heading),
            StructureKind::Drawer | StructureKind::PropertyDrawer => {
                self.delimiters(node, Role::Drawer)
            }
            StructureKind::Item { .. } => self.optional(&node.opening, Role::List),
            StructureKind::Block(kind) => self.block(node, kind),
            _ => self.leaf(node),
        }
    }

    fn leaf(&mut self, node: &Raw) -> bool {
        let role = match node.kind {
            StructureKind::Property { .. } => Some(Role::Property),
            StructureKind::Planning => Some(Role::Planning),
            StructureKind::Keyword | StructureKind::AffiliatedKeyword => Some(Role::Keyword),
            StructureKind::TableRow { .. } => Some(Role::Table),
            StructureKind::TableFormula => Some(Role::Formula),
            _ => extra_role(node.kind),
        };
        role.is_none_or(|role| self.push(node.source.clone(), role))
    }

    fn block(&mut self, node: &Raw, kind: BlockKind) -> bool {
        let role = match kind {
            BlockKind::Source | BlockKind::Example | BlockKind::Export => Some(Role::Code),
            BlockKind::Comment => Some(Role::Comment),
            _ => None,
        };
        if !role.is_none_or(|role| self.push(node.content.clone(), role)) {
            return false;
        }
        self.delimiters(node, Role::Block)
    }

    fn optional(&mut self, range: &Option<Range<usize>>, role: Role) -> bool {
        range
            .as_ref()
            .is_none_or(|range| self.push(range.clone(), role))
    }

    fn delimiters(&mut self, node: &Raw, role: Role) -> bool {
        self.optional(&node.opening, role) && self.optional(&node.closing, role)
    }
}

fn extra_role(kind: StructureKind) -> Option<Role> {
    match kind {
        StructureKind::Comment => Some(Role::Comment),
        StructureKind::FixedWidth => Some(Role::Code),
        StructureKind::Malformed => Some(Role::Malformed),
        _ => None,
    }
}

fn inline_role(kind: InlineKind) -> Role {
    match kind {
        InlineKind::Code | InlineKind::Verbatim => Role::Code,
        InlineKind::Entity => Role::Entity,
        InlineKind::Link(_) => Role::Link,
        InlineKind::Target | InlineKind::RadioTarget => Role::Target,
        InlineKind::FootnoteReference | InlineKind::FootnoteInline => Role::Footnote,
        _ => Role::Emphasis,
    }
}
