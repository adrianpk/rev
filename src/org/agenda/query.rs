mod observe;
pub use observe::{QueryResult, QueryRow};
// Typed query plans parse before source discovery or worker launch.
use crate::{
    config::OrgLimits,
    org::{Cancellation, MatchData, MatchQuery, SearchQuery},
};
use ropey::RopeSlice;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryKind {
    Todo,
    Search,
    Tags,
}
impl QueryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Search => "Search",
            Self::Tags => "Tags/properties",
        }
    }
}
#[derive(Clone, Debug)]
pub struct AgendaQuery {
    pub kind: QueryKind,
    pub source: String,
    plan: Plan,
    filter: Option<MatchQuery>,
    fields: std::collections::BTreeSet<String>,
}
#[derive(Clone, Debug)]
enum Plan {
    Todo(Vec<String>),
    Search(SearchQuery),
    Tags(MatchQuery),
}
impl AgendaQuery {
    pub fn parse(kind: QueryKind, source: &str, limits: &OrgLimits) -> Result<Self, String> {
        if source.len() > limits.query_bytes || source.chars().any(char::is_control) {
            return Err(
                "Agenda query exceeds its text limit or contains control characters".into(),
            );
        }
        let plan = match kind {
            QueryKind::Todo => Plan::Todo(todo(source)?),
            QueryKind::Search => Plan::Search(SearchQuery::parse(
                source,
                limits.query_bytes,
                limits.query_depth,
            )?),
            QueryKind::Tags => Plan::Tags(MatchQuery::bounded(
                source,
                limits.query_bytes,
                limits.query_depth,
            )?),
        };
        Ok(Self {
            kind,
            source: source.into(),
            plan,
            filter: None,
            fields: std::collections::BTreeSet::new(),
        })
    }
    pub fn properties(&self) -> std::collections::BTreeSet<&str> {
        let mut fields: std::collections::BTreeSet<_> =
            self.fields.iter().map(String::as_str).collect();
        if let Plan::Tags(query) = &self.plan {
            fields.extend(query.properties());
        }
        if let Some(filter) = &self.filter {
            fields.extend(filter.properties());
        }
        fields.retain(|s| !matches!(*s, "DATE" | "TIME" | "CLOCKSUM" | "CLOCKOPEN"));
        fields.insert("TODO");
        fields
    }
    pub fn block(block: &crate::config::AgendaBlock, limits: &OrgLimits) -> Result<Self, String> {
        use crate::config::AgendaKind as K;
        block.validate()?;
        let kind = match block.kind {
            K::Todo => QueryKind::Todo,
            K::Search => QueryKind::Search,
            _ => QueryKind::Tags,
        };
        let source = if block.query.is_empty() && kind == QueryKind::Tags {
            "LEVEL>=1"
        } else {
            &block.query
        };
        let mut query = Self::parse(kind, source, limits)?;
        if !block.filter.is_empty() {
            query.filter = Some(MatchQuery::bounded(
                &block.filter,
                limits.query_bytes,
                limits.query_depth,
            )?);
        }
        query
            .fields
            .extend(block.columns.iter().map(|s| s.to_ascii_uppercase()));
        query.sort_fields(&block.sort);
        Ok(query)
    }

    fn sort_fields(&mut self, sort: &[crate::config::AgendaSort]) {
        use crate::config::AgendaSort as S;
        for sort in sort {
            let key = match sort {
                S::PriorityDown => "PRIORITY",
                S::EffortUp | S::EffortDown => "EFFORT",
                _ => continue,
            };
            self.fields.insert(key.into());
        }
    }

    pub(crate) fn matches_with_policy(
        &self,
        data: &MatchData,
        text: RopeSlice<'_>,
        cancel: &Cancellation,
        policy: &crate::config::Org,
    ) -> Result<bool, String> {
        if let Some(filter) = &self.filter
            && !filter.matches_with_units(data, &policy.duration.units)?
        {
            return Ok(false);
        }
        match &self.plan {
            Plan::Tags(query) => query.matches_with_units(data, &policy.duration.units),
            _ => self.matches(data, text, cancel),
        }
    }
    pub fn matches(
        &self,
        data: &MatchData,
        text: RopeSlice<'_>,
        cancel: &Cancellation,
    ) -> Result<bool, String> {
        match &self.plan {
            Plan::Todo(states) => Ok(if states.is_empty() {
                !data.finished && data.properties.get("TODO").is_some_and(|s| !s.is_empty())
            } else {
                data.properties
                    .get("TODO")
                    .is_some_and(|s| states.contains(s))
            }),
            Plan::Search(query) => query.matches(text, cancel),
            Plan::Tags(query) => query.matches(data),
        }
    }
}
fn todo(source: &str) -> Result<Vec<String>, String> {
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }
    let states: Vec<_> = source
        .split('|')
        .map(str::trim)
        .map(str::to_owned)
        .collect();
    if states.len() > 64 || states.iter().any(|s| !crate::org::query::task_keyword(s)) {
        return Err("TODO selection requires at most 64 keywords separated by |".into());
    }
    Ok(states)
}
