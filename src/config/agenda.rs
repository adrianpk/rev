//! Data-only agenda definitions. Admission performs no source IO or evaluation.
use super::AgendaSpan;
use serde::Deserialize;
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgendaView {
    pub key: String,
    pub title: String,
    pub blocks: Vec<AgendaBlock>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgendaBlock {
    pub kind: AgendaKind,
    pub title: String,
    pub query: String,
    pub filter: String,
    pub span: Option<AgendaSpan>,
    pub sources: Vec<String>,
    pub columns: Vec<String>,
    pub sort: Vec<AgendaSort>,
    pub row_limit: usize,
}
impl Default for AgendaBlock {
    fn default() -> Self {
        Self {
            kind: AgendaKind::Todo,
            title: String::new(),
            query: String::new(),
            filter: String::new(),
            span: None,
            sources: Vec::new(),
            columns: Vec::new(),
            sort: Vec::new(),
            row_limit: 20000,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgendaKind {
    Calendar,
    #[default]
    Todo,
    Search,
    Tags,
    Properties,
    Columns,
    Clock,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgendaSort {
    TitleUp,
    TitleDown,
    TodoUp,
    PriorityDown,
    EffortUp,
    EffortDown,
    DateUp,
    ClockDown,
    SourceUp,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum NamedSpan {
    Day,
    Week,
    Month,
    Year,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum SpanInput {
    Name(NamedSpan),
    Days(u16),
}
impl<'de> Deserialize<'de> for AgendaSpan {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match SpanInput::deserialize(deserializer)? {
            SpanInput::Name(name) => Ok(match name {
                NamedSpan::Day => Self::Day,
                NamedSpan::Week => Self::Week,
                NamedSpan::Month => Self::Month,
                NamedSpan::Year => Self::Year,
            }),
            SpanInput::Days(days) if (1..=366).contains(&days) => Ok(Self::Days(days)),
            _ => Err(serde::de::Error::custom("Agenda day count requires 1..366")),
        }
    }
}
pub(super) fn views(views: &[AgendaView]) -> Result<(), String> {
    if views.len() > 64 {
        return Err("Agenda admits at most 64 custom views".into());
    }
    let mut keys = std::collections::BTreeSet::new();
    for view in views {
        view.validate()?;
        if !keys.insert(&view.key) {
            return Err("Agenda view keys must be unique".into());
        }
    }
    Ok(())
}
impl AgendaView {
    pub fn validate(&self) -> Result<(), String> {
        text(&self.key, 16)?;
        text(&self.title, 256)?;
        if self.key.is_empty() || self.blocks.is_empty() || self.blocks.len() > 32 {
            return Err("Agenda view requires a key and 1..32 blocks".into());
        }
        for block in &self.blocks {
            block.validate()?;
        }
        Ok(())
    }
}
impl AgendaBlock {
    pub fn validate(&self) -> Result<(), String> {
        text(&self.title, 256)?;
        text(&self.query, 65536)?;
        text(&self.filter, 65536)?;
        self.bounds()?;
        if matches!(self.span, Some(AgendaSpan::Days(n)) if !(1..=366).contains(&n)) {
            return Err("Agenda day count requires 1..366".into());
        }
        for column in &self.columns {
            column_name(column)?;
        }
        crate::org::agenda::SourceSelection::paths(&self.sources)?;
        Ok(())
    }
    fn bounds(&self) -> Result<(), String> {
        if self.columns.len() > 16 || self.sort.len() > 8 || !(1..=100000).contains(&self.row_limit)
        {
            return Err(
                "Agenda block requires at most 16 columns, 8 sort keys and 1..100000 rows".into(),
            );
        }
        Ok(())
    }
}
pub(crate) fn column_name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(
            "Agenda column requires a property name of 1..64 ASCII letters, digits or underscores"
                .into(),
        );
    }
    Ok(())
}
fn text(value: &str, maximum: usize) -> Result<(), String> {
    if value.len() > maximum || value.chars().any(char::is_control) {
        return Err(format!(
            "Agenda text exceeds {maximum} bytes or contains control characters"
        ));
    }
    Ok(())
}
