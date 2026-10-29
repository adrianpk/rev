use super::*;
use crate::org::{
    priority::Priority,
    visibility::{query_file_tags, query_header},
};
pub(super) struct Declarations {
    pub workflow: Workflow,
    file_tags: Vec<String>,
    priority: Priority,
}
pub(super) struct Heading {
    pub end: usize,
    pub title: String,
    pub task: Option<String>,
    pub excluded: bool,
    level: usize,
    local: Vec<String>,
    tags: Vec<String>,
    priority: u8,
    finished: bool,
}
pub(super) struct Context<'a> {
    pub properties: &'a Properties,
    pub text: &'a Rope,
    pub policy: &'a crate::config::Org,
    pub declarations: &'a Declarations,
    pub file: Option<&'a std::path::Path>,
}
impl Declarations {
    pub fn observe(
        nodes: &[Raw],
        text: &Rope,
        policy: &crate::config::Org,
    ) -> Result<Self, String> {
        let mut value = Self {
            workflow: Workflow::default(),
            file_tags: Vec::new(),
            priority: Priority::configured(&policy.priority)?,
        };
        for node in nodes.iter().filter(|n| n.kind == StructureKind::Keyword) {
            let Some(name) = &node.name else {
                continue;
            };
            if node.content.len() > 4096 {
                return Err("Agenda declaration exceeds 4096 bytes".into());
            }
            let name = text
                .byte_slice(name.clone())
                .to_string()
                .to_ascii_uppercase();
            value.declaration(&name, &text.byte_slice(node.content.clone()).to_string())?;
        }
        value.workflow = value.workflow.effective(&policy.todo)?;
        Ok(value)
    }
    fn declaration(&mut self, name: &str, text: &str) -> Result<(), String> {
        match name {
            "TODO" | "SEQ_TODO" | "TYP_TODO" => self.workflow.declaration(text),
            "FILETAGS" => {
                let tags = query_file_tags(text)?;
                if self.file_tags.len() + tags.len() > 8192 {
                    return Err("Agenda file tags exceed 8192 records".into());
                }
                self.file_tags.extend(tags);
            }
            "PRIORITIES" => {
                let parts: Vec<_> = text.split_whitespace().collect();
                if parts.len() != 3 {
                    return Err("Org PRIORITIES requires highest lowest default".into());
                }
                self.priority = Priority::new(parts[0], parts[1], parts[2])?;
            }
            _ => (),
        }
        Ok(())
    }
}
impl Heading {
    pub fn observe(
        node: &Raw,
        text: &Rope,
        parent: Option<&Heading>,
        decl: &Declarations,
        policy: &crate::config::Org,
    ) -> Result<Self, String> {
        let header = query_header(node, text)?;
        let state = decl.workflow.state(&header.word);
        let task = state.map(|s| s.name.clone());
        let title = if task.is_some() {
            header.title[header.word.len()..].trim()
        } else {
            header.title.trim()
        };
        let (title, priority) = priority(title, decl.priority)?;
        let tags = membership(&header.tags, parent, &decl.file_tags, policy.tags.inherit)?;
        let level = match node.kind {
            StructureKind::Heading { level } => level,
            _ => unreachable!("raw heading"),
        };
        Ok(Self {
            end: node.source.end,
            title,
            task,
            excluded: header.word == "COMMENT"
                || !policy.agenda.include_archived && header.tags.iter().any(|t| t == "ARCHIVE")
                || parent.is_some_and(|h| h.excluded),
            level,
            local: header.tags,
            tags,
            priority,
            finished: state.is_some_and(|s| s.done),
        })
    }
    pub fn data(
        &self,
        query: &AgendaQuery,
        byte: usize,
        context: &Context<'_>,
    ) -> Result<MatchData, String> {
        let mut data = MatchData {
            tags: self.tags.clone(),
            finished: self.finished,
            ..MatchData::default()
        };
        for key in query.properties() {
            if let Some(value) = self.value(byte, key, context)? {
                data.properties.insert(key.into(), value);
            }
        }
        Ok(data)
    }
    fn value(&self, byte: usize, key: &str, c: &Context<'_>) -> Result<Option<String>, String> {
        match key {
            "TODO" => Ok(self.task.clone()),
            "LEVEL" => Ok(Some(self.level.to_string())),
            "PRIORITY" => Ok(Some(char::from(self.priority).to_string())),
            "ITEM" => Ok(Some(self.title.clone())),
            "TAGS" => Ok(tag_text(&self.local)),
            "ALLTAGS" => Ok(tag_text(&self.tags)),
            "FILE" => Ok(c
                .file
                .and_then(|p| p.file_name())
                .map(|p| p.to_string_lossy().into_owned())),
            _ => self.stored(byte, key, c),
        }
    }
    fn stored(&self, byte: usize, key: &str, c: &Context<'_>) -> Result<Option<String>, String> {
        if crate::org::derived_property(key) {
            return Err(format!("Unsupported derived agenda property {key}"));
        }
        let value = c
            .properties
            .query_value(c.text, byte, key, &c.policy.properties.inherit)?;
        if key == "EFFORT"
            && let Some(value) = &value
        {
            crate::org::time::Duration::parse(value, &c.policy.duration.units)?;
        }
        Ok(value)
    }
}
fn priority(title: &str, policy: Priority) -> Result<(String, u8), String> {
    let word = title.split_whitespace().next().unwrap_or("");
    if word.starts_with("[#") {
        let letter = word
            .strip_prefix("[#")
            .and_then(|s| s.strip_suffix(']'))
            .ok_or("Malformed priority cookie")?;
        let value = policy.admit(letter)?.ok_or("Empty priority cookie")?;
        return Ok((title[word.len()..].trim().into(), value));
    }
    Ok((title.into(), policy.default))
}
fn membership(
    local: &[String],
    parent: Option<&Heading>,
    file: &[String],
    inherit: bool,
) -> Result<Vec<String>, String> {
    let mut values: BTreeSet<_> = local.iter().cloned().collect();
    if inherit {
        values.extend(parent.map_or(file, |p| p.tags.as_slice()).iter().cloned());
    }
    if values.len() > 8192 {
        return Err("Agenda effective tags exceed 8192 records".into());
    }
    Ok(values.into_iter().collect())
}
fn tag_text(tags: &[String]) -> Option<String> {
    (!tags.is_empty()).then(|| format!(":{}:", tags.join(":")))
}
