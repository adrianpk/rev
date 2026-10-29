use super::*;
use crate::org::{task::Workflow, visibility::query_header};
use ropey::Rope;
struct Heading {
    id: usize,
    parent: Option<usize>,
    level: usize,
    start: usize,
    body: Range<usize>,
    end: usize,
    title: String,
}
pub(super) struct Headings<'a> {
    values: Vec<Heading>,
    size: usize,
    cancel: &'a Cancellation,
}
impl<'a> Headings<'a> {
    pub fn new(
        nodes: &[structure::Raw],
        text: &Rope,
        todo: &crate::config::Todo,
        cancel: &'a Cancellation,
    ) -> Result<Self, String> {
        let workflow = workflow(nodes, text, todo, cancel)?;
        let mut result = Self {
            values: Vec::new(),
            size: text.len_bytes(),
            cancel,
        };
        for (id, node) in nodes.iter().enumerate() {
            if cancel.is_cancelled() {
                return Err("Capture destination discovery cancelled".into());
            }
            if let StructureKind::Heading { level } = node.kind {
                let title = title(node, text, &workflow)?;
                let body = node.opening.as_ref().expect("structural heading").end;
                result.values.push(Heading {
                    id,
                    parent: node.parent,
                    level,
                    start: node.source.start,
                    body: body..node.source.end,
                    end: node.source.end,
                    title,
                });
            }
        }
        result.sections();
        Ok(result)
    }
    fn sections(&mut self) {
        for index in 0..self.values.len() {
            if let Some(child) = self.values.get(index + 1) {
                let next = child.start;
                self.values[index].body.end = self.values[index].body.end.min(next);
            }
        }
    }
    pub fn target(&self, selection: &CaptureTarget, date: Date) -> Result<Target, String> {
        let mut result = self.scope(None);
        match selection {
            CaptureTarget::File { .. } => (),
            CaptureTarget::Headline { headline, .. } => {
                if let Some(id) = self.find(None, headline, true)? {
                    result = self.scope(Some(id));
                } else {
                    result.plan(headline.clone());
                }
            }
            CaptureTarget::Outline { outline, .. } | CaptureTarget::DateTree { outline, .. } => {
                self.path(&mut result, outline)?
            }
        }
        if matches!(selection, CaptureTarget::DateTree { .. }) {
            self.date_tree(&mut result, date)?;
        }
        Ok(result)
    }
    fn path(&self, target: &mut Target, names: &[String]) -> Result<(), String> {
        for name in names {
            let id = target
                .parent
                .and_then(|p| self.values.iter().find(|h| h.start == p.0))
                .map(|h| h.id);
            let found = if target.missing.is_empty() {
                self.find(id, name, false)?
            } else {
                None
            };
            if let Some(found) = found {
                *target = self.scope(Some(found));
            } else {
                target.plan(name.clone());
            }
        }
        Ok(())
    }
    fn find(
        &self,
        parent: Option<usize>,
        title: &str,
        global: bool,
    ) -> Result<Option<usize>, String> {
        let mut found = None;
        for h in &self.values {
            if self.cancel.is_cancelled() {
                return Err("Capture destination discovery cancelled".into());
            }
            if (global || h.parent == parent) && h.title == title {
                if found.is_some() {
                    return Err(
                        "Ambiguous capture heading; select an exact unique outline path".into(),
                    );
                }
                found = Some(h.id);
            }
        }
        Ok(found)
    }
    fn scope(&self, id: Option<usize>) -> Target {
        match id.and_then(|id| self.values.iter().find(|h| h.id == id)) {
            Some(h) => Target {
                parent: Some(Byte(h.start)),
                level: h.level,
                body: Byte(h.body.start)..Byte(h.body.end),
                subtree_end: Byte(h.end),
                missing: Vec::new(),
                missing_at: None,
                container: Container::Body,
            },
            None => Target {
                parent: None,
                level: 0,
                body: Byte(0)..Byte(self.size),
                subtree_end: Byte(self.size),
                missing: Vec::new(),
                missing_at: None,
                container: Container::Body,
            },
        }
    }
    fn date_tree(&self, target: &mut Target, date: Date) -> Result<(), String> {
        let names = date_titles(date);
        for name in names {
            let prefix = name.split_whitespace().next().expect("date prefix");
            let parent = target
                .parent
                .and_then(|p| self.values.iter().find(|h| h.start == p.0))
                .map(|h| h.id);
            let found = if target.missing.is_empty() {
                self.date_heading(parent, prefix)?
            } else {
                None
            };
            if let Some(found) = found {
                *target = self.scope(Some(found));
            } else {
                if target.missing.is_empty() {
                    target.missing_at = self.next_date(parent, prefix)?;
                }
                target.plan(name);
            }
        }
        Ok(())
    }
    fn next_date(&self, parent: Option<usize>, prefix: &str) -> Result<Option<Byte>, String> {
        for h in &self.values {
            if self.cancel.is_cancelled() {
                return Err("Capture destination discovery cancelled".into());
            }
            let token = h.title.split_whitespace().next().unwrap_or("");
            if h.parent == parent && date_token(token, prefix.len()) && token > prefix {
                return Ok(Some(Byte(h.start)));
            }
        }
        Ok(None)
    }
    fn date_heading(&self, parent: Option<usize>, prefix: &str) -> Result<Option<usize>, String> {
        let mut found = None;
        for h in &self.values {
            if self.cancel.is_cancelled() {
                return Err("Capture destination discovery cancelled".into());
            }
            if h.parent == parent && h.title.split_whitespace().next() == Some(prefix) {
                if found.is_some() {
                    return Err("Ambiguous capture date-tree heading".into());
                }
                found = Some(h.id);
            }
        }
        Ok(found)
    }
}
fn date_titles(date: Date) -> [String; 3] {
    let month = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][usize::from(date.month() - 1)];
    let day = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ][usize::from(date.weekday() - 1)];
    [
        format!("{:04}", date.year()),
        format!("{:04}-{:02} {month}", date.year(), date.month()),
        format!("{date} {day}"),
    ]
}
fn workflow(
    nodes: &[structure::Raw],
    text: &Rope,
    todo: &crate::config::Todo,
    cancel: &Cancellation,
) -> Result<Workflow, String> {
    let mut value = Workflow::default();
    for node in nodes.iter().filter(|n| n.kind == StructureKind::Keyword) {
        if cancel.is_cancelled() {
            return Err("Capture destination discovery cancelled".into());
        }
        if is_workflow(node, text) {
            if node.content.len() > 4096 {
                return Err("Capture task declaration exceeds 4096 bytes".into());
            }
            value.declaration(&text.byte_slice(node.content.clone()).to_string());
        }
    }
    value.effective(todo)
}
fn is_workflow(node: &structure::Raw, text: &Rope) -> bool {
    let Some(name) = &node.name else {
        return false;
    };
    if name.len() > 16 {
        return false;
    }
    let name = text.byte_slice(name.clone()).to_string();
    ["TODO", "SEQ_TODO", "TYP_TODO"]
        .iter()
        .any(|k| name.eq_ignore_ascii_case(k))
}

fn title(node: &structure::Raw, text: &Rope, workflow: &Workflow) -> Result<String, String> {
    let header = query_header(node, text)?;
    let mut title = header.title.trim();
    if workflow.state(&header.word).is_some() {
        title = title
            .strip_prefix(&header.word)
            .expect("observed word")
            .trim_start();
    }
    if title.starts_with("[#") && title.as_bytes().get(3) == Some(&b']') {
        title = title[4..].trim_start();
    }
    if let Some(rest) = title.strip_prefix("COMMENT ") {
        title = rest;
    }
    Ok(title.to_owned())
}

fn date_token(value: &str, length: usize) -> bool {
    if value.len() != length {
        return false;
    }
    match length {
        4 => value.parse::<u16>().is_ok_and(|n| (1..=9999).contains(&n)),
        7 => crate::org::time::Timestamp::parse(&format!("<{value}-01>")).is_ok(),
        10 => crate::org::time::Timestamp::parse(&format!("<{value}>")).is_ok(),
        _ => false,
    }
}
