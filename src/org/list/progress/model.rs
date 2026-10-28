use super::{Cookie, Data, Heading, Lists};
use crate::org::{StructureKind, structure::Raw};
use std::collections::HashMap;
use std::ops::Range;

pub(in crate::org::list) fn cookies(
    text: &ropey::Rope,
    range: Range<usize>,
    allowance: usize,
) -> Vec<Cookie> {
    let line = text.byte_slice(range.clone()).to_string();
    let mut cookies = Vec::new();
    let mut position = 0;
    while let Some(start) = line[position..].find('[').map(|start| position + start) {
        let Some(end) = line[start..].find(']').map(|end| start + end) else {
            break;
        };
        if let Some(percent) = kind(&line[start + 1..end]) {
            cookies.push(Cookie {
                source: range.start + start..range.start + end + 1,
                percent,
            });
        }
        if cookies.len() > allowance {
            break;
        }
        position = end + 1;
    }
    cookies
}

fn kind(body: &str) -> Option<bool> {
    if let Some(number) = body.strip_suffix('%') {
        return number
            .bytes()
            .all(|byte| byte.is_ascii_digit())
            .then_some(true);
    }
    let (a, b) = body.split_once('/')?;
    a.bytes()
        .chain(b.bytes())
        .all(|byte| byte.is_ascii_digit())
        .then_some(false)
}

impl Lists {
    pub(in crate::org::list) fn progress_node(
        &mut self,
        node: &Raw,
        text: &ropey::Rope,
        lists: &HashMap<usize, usize>,
    ) {
        match node.kind {
            StructureKind::Heading { level } => self.heading_node(node, text, level),
            StructureKind::Property { .. } => self.property_node(node, text),
            StructureKind::AffiliatedKeyword => self.radio_node(node, text, lists),
            _ => (),
        }
    }

    fn heading_node(&mut self, node: &Raw, text: &ropey::Rope, level: usize) {
        let title = text.byte_slice(node.content.clone()).to_string();
        let word = title.split_whitespace().next().unwrap_or("");
        let task = if word.len() <= 128 {
            word.into()
        } else {
            String::new()
        };
        let cookies = cookies(
            text,
            node.content.clone(),
            4096 - self.cookie_count.min(4096),
        );
        self.cookie_count += cookies.len();
        let parent = self
            .headings
            .iter()
            .rposition(|heading| heading.level < level);
        self.headings.push(Heading {
            source: node.source.clone(),
            title: node.content.clone(),
            parent,
            level,
            task,
            ordered: false,
            data: Data::Auto,
            cookies,
        });
    }

    fn property_node(&mut self, node: &Raw, text: &ropey::Rope) {
        let Some(heading) = self.headings.last_mut() else {
            return;
        };
        let Some(name) = &node.name else {
            return;
        };
        let name = text.byte_slice(name.clone()).to_string();
        let value = text.byte_slice(node.content.clone()).to_string();
        if name.eq_ignore_ascii_case("COOKIE_DATA") {
            heading.data = match value.trim() {
                "todo" => Data::Todo,
                "checkbox" => Data::Checkbox,
                _ => Data::Auto,
            };
        }
        if name.eq_ignore_ascii_case("ORDERED") {
            heading.ordered = value.trim() == "t";
        }
    }

    fn radio_node(&mut self, node: &Raw, text: &ropey::Rope, lists: &HashMap<usize, usize>) {
        let Some(list) = node.parent.and_then(|parent| lists.get(&parent)).copied() else {
            return;
        };
        let Some(name) = &node.name else {
            return;
        };
        if !text
            .byte_slice(name.clone())
            .to_string()
            .eq_ignore_ascii_case("ATTR_ORG")
        {
            return;
        }
        let value = text.byte_slice(node.content.clone()).to_string();
        let mut words = value.split_whitespace();
        while let Some(word) = words.next() {
            if word == ":radio" {
                self.lists[list].radio = words.next() == Some("t");
            }
        }
    }
}
