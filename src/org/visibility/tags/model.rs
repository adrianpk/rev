use super::*;
use crate::org::{Coverage, StructureKind};
use ropey::Rope;

impl Tags {
    pub(in crate::org) fn observe(
        structure: &Structure,
        text: &Rope,
        cancel: &Cancellation,
    ) -> Self {
        let mut tags = Self {
            complete: structure.coverage() == Coverage::Complete,
            workflow: vec![("TODO".into(), false), ("DONE".into(), true)],
            workflow_valid: true,
            ..Self::default()
        };
        let sections: BTreeSet<_> = structure
            .raw_nodes()
            .enumerate()
            .filter(|(_, n)| n.kind == StructureKind::Section)
            .map(|(i, _)| i)
            .collect();
        for node in structure.raw_nodes() {
            if cancel.is_cancelled() || tags.at_limit() {
                tags.complete = false;
                break;
            }
            tags.collect(node, text, &sections);
        }
        tags.complete &= !tags.at_limit();
        tags
    }
    fn at_limit(&self) -> bool {
        self.count > 8192 || self.headers.len() > 4096 || self.contexts.len() > 8192
    }
    fn collect(
        &mut self,
        node: &crate::org::structure::Raw,
        text: &Rope,
        sections: &BTreeSet<usize>,
    ) {
        match node.kind {
            StructureKind::Heading { .. } => self.header(node, text),
            StructureKind::Keyword if node.parent.is_some_and(|p| sections.contains(&p)) => {
                self.keyword(node, text)
            }
            StructureKind::PropertyDrawer => self.contexts.push(node.source.clone()),
            StructureKind::Planning | StructureKind::Comment
                if node.parent.is_some_and(|p| sections.contains(&p)) =>
            {
                self.contexts.push(node.source.clone())
            }
            _ => (),
        }
    }
    fn header(&mut self, node: &crate::org::structure::Raw, text: &Rope) {
        let title = text.byte_slice(node.content.clone()).to_string();
        let suffix = title
            .rsplit_once(char::is_whitespace)
            .map_or(title.as_str(), |(_, s)| s);
        let start = node.content.end - suffix.len();
        let looks = suffix.starts_with(':') && suffix.ends_with(':');
        let (span, local) = if looks {
            (start..node.content.end, parse_suffix(suffix, start))
        } else {
            (node.content.end..node.content.end, Some(Vec::new()))
        };
        let local = local.map(|local| {
            for tag in &local {
                self.candidates.insert(tag.name.clone());
            }
            local
        });
        let valid = local.is_some();
        let local = local.unwrap_or_default();
        self.count += local.len();
        let mut words = title.split_whitespace();
        let first = words.next().unwrap_or("");
        let word = if first.len() <= 128 {
            first.into()
        } else {
            String::new()
        };
        let priority = priority(first).or_else(|| words.next().and_then(priority));
        let content = node.content.start..span.start;
        self.headers.push(Header {
            span,
            valid,
            local,
            title: content,
            word,
            priority,
        });
    }
    fn keyword(&mut self, node: &crate::org::structure::Raw, text: &Rope) {
        let Some(name) = &node.name else {
            return;
        };
        let name = text.byte_slice(name.clone()).to_string();
        let value = text.byte_slice(node.content.clone()).to_string();
        match name.to_ascii_uppercase().as_str() {
            "FILETAGS" => self.file_tags(&value, node.content.start),
            "TAGS" => {
                for token in value.split_whitespace().take(8193) {
                    self.candidate(token);
                }
            }
            "TODO" | "SEQ_TODO" | "TYP_TODO" => self.workflow(&value),
            _ => (),
        }
    }
    fn file_tags(&mut self, value: &str, offset: usize) {
        if let Some(tags) = parse_suffix(value.trim(), offset) {
            self.count += tags.len();
            self.candidates.extend(tags.iter().map(|t| t.name.clone()));
            self.file.extend(tags);
        }
    }
    fn workflow(&mut self, value: &str) {
        if !self.workflow_declared {
            self.workflow.clear();
            self.workflow_declared = true;
        }
        let mut finished = false;
        let explicit = value.split_whitespace().any(|w| w == "|");
        for token in value.split_whitespace().take(66) {
            if token == "|" {
                finished = true;
                continue;
            }
            let word = token.split('(').next().unwrap_or(token);
            if !valid_tag(word) || self.workflow.len() == 64 {
                self.workflow_valid = false;
                return;
            }
            self.workflow.push((word.into(), finished));
        }
        if !explicit && let Some(last) = self.workflow.last_mut() {
            last.1 = true;
        }
    }
    fn candidate(&mut self, token: &str) {
        let token = token.split('(').next().unwrap_or(token);
        if valid_tag(token) {
            self.candidates.insert(token.into());
            self.count += 1;
        }
    }
}
fn parse_suffix(value: &str, offset: usize) -> Option<Vec<Tag>> {
    let inner = value.strip_prefix(':')?.strip_suffix(':')?;
    let mut tags = Vec::new();
    let mut start = offset + 1;
    for name in inner.split(':').take(129) {
        if !valid_tag(name) || tags.len() == 128 {
            return None;
        }
        tags.push(Tag {
            name: name.into(),
            source: start..start + name.len(),
        });
        start += name.len() + 1;
    }
    Some(tags)
}

fn priority(word: &str) -> Option<String> {
    let letter = word.strip_prefix("[#")?.strip_suffix(']')?;
    (letter.len() == 1 && letter.as_bytes()[0].is_ascii_uppercase()).then(|| letter.into())
}
