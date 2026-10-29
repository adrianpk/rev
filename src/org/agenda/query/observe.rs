use super::*;
use crate::org::{
    Coverage, Limits, StructureKind,
    agenda::{CalendarQuery, CalendarSource, SourceIssue, SourceSet, SourceVersion},
    property::Properties,
    structure::{self, Raw},
    task::Workflow,
};
use ropey::Rope;
use std::{
    collections::BTreeSet,
    sync::Arc,
    time::{Duration, Instant},
};
mod metadata;
#[derive(Clone, Debug)]
pub struct QueryRow {
    pub title: Arc<str>,
    pub task: Option<String>,
    pub source: usize,
    pub byte: usize,
    pub stale: bool,
    pub fields: std::collections::BTreeMap<String, String>,
    pub block: Option<String>,
    pub block_index: usize,
}
pub struct QueryResult {
    pub query: AgendaQuery,
    pub context: CalendarQuery,
    pub sources: Vec<CalendarSource>,
    pub rows: Vec<QueryRow>,
    pub issues: Vec<SourceIssue>,
    pub truncated: bool,
    pub cancelled: bool,
    pub view: Option<crate::config::AgendaView>,
    pub baseline: Option<crate::config::AgendaView>,
    pub(in crate::org::agenda) bytes: usize,
}
impl QueryResult {
    pub fn observe(
        sources: &SourceSet,
        query: AgendaQuery,
        context: CalendarQuery,
        policy: &crate::config::Org,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        if !(1..=100000).contains(&policy.limits.agenda_rows)
            || !(1..=600).contains(&policy.limits.job_seconds)
        {
            return Err("Agenda query requires bounded row/time policies".into());
        }
        let bounded =
            cancel.bounded(Instant::now() + Duration::from_secs(policy.limits.job_seconds as u64));
        let mut result = Self {
            query,
            context,
            sources: sources
                .documents
                .iter()
                .map(|s| CalendarSource {
                    path: s.path.clone(),
                    version: s.version.clone(),
                })
                .collect(),
            rows: Vec::new(),
            issues: sources.issues.clone(),
            truncated: sources.truncated || sources.cancelled,
            cancelled: false,
            view: None,
            baseline: None,
            bytes: 0,
        };
        for (id, source) in sources.documents.iter().enumerate() {
            if bounded.is_cancelled() {
                break;
            }
            if let Err(e) = result.document(id, source.snapshot(), policy, &bounded) {
                result.issue(id, e);
            }
            if result.truncated && result.rows.len() == policy.limits.agenda_rows {
                break;
            }
        }
        result.cancelled = bounded.is_cancelled() && !bounded.expired();
        result.truncated |= bounded.expired();
        Ok(result)
    }
    pub fn complete(&self) -> bool {
        !self.truncated && !self.cancelled && self.issues.is_empty()
    }
    pub fn revalidate(&mut self, buffers: &crate::buffer::Buffers) {
        for row in &mut self.rows {
            if let SourceVersion::Buffer(stamp) = self.sources[row.source].version {
                row.stale = !buffers.get(stamp.buffer).is_ok_and(|b| b.stamp() == stamp);
            }
        }
        if self.rows.iter().any(|r| r.stale) {
            self.truncated = true;
        }
    }
    fn issue(&mut self, source: usize, message: String) {
        if self.issues.len() < 4096 {
            self.issues.push(SourceIssue {
                path: self.sources[source].path.clone(),
                message,
            });
        } else {
            self.truncated = true;
        }
    }
    fn document(
        &mut self,
        source: usize,
        text: Rope,
        policy: &crate::config::Org,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let doc = structure::observe(text.clone(), Limits::default(), cancel);
        if doc.coverage != Coverage::Complete {
            self.issue(source, "Incomplete structural source observation".into());
        }
        if doc.nodes.iter().any(|n| n.kind == StructureKind::Malformed) {
            self.issue(source, "Malformed Org structure".into());
        }
        let properties = Properties::observe_nodes(
            doc.nodes.iter(),
            doc.coverage == Coverage::Complete,
            &text,
            cancel,
        );
        let declarations = metadata::Declarations::observe(&doc.nodes, &text, policy)?;
        let heads: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, StructureKind::Heading { .. }))
            .collect();
        let file = self.sources[source].path.clone();
        let context = metadata::Context {
            properties: &properties,
            text: &text,
            declarations: &declarations,
            policy,
            file: file.as_deref(),
        };
        self.headings(source, &heads, &context, cancel)
    }
    fn headings(
        &mut self,
        source: usize,
        heads: &[&Raw],
        context: &metadata::Context<'_>,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let mut ancestors = Vec::<metadata::Heading>::new();
        for (i, node) in heads.iter().enumerate() {
            if cancel.is_cancelled() {
                break;
            }
            while ancestors.last().is_some_and(|h| h.end <= node.source.start) {
                ancestors.pop();
            }
            let end = heads
                .get(i + 1)
                .map_or(context.text.len_bytes(), |n| n.source.start);
            let heading = metadata::Heading::observe(
                node,
                context.text,
                ancestors.last(),
                context.declarations,
                context.policy,
            )?;
            if !heading.excluded {
                let data = heading.data(&self.query, node.source.start, context)?;
                if self.query.matches_with_policy(
                    &data,
                    context.text.byte_slice(node.source.start..end),
                    cancel,
                    context.policy,
                )? {
                    self.push(
                        (source, node),
                        &heading,
                        data,
                        context.policy.limits.agenda_rows,
                    )?;
                }
            }
            ancestors.push(heading);
        }
        Ok(())
    }
    fn retain_fields(&mut self, data: &MatchData, title: &str) -> Result<(), String> {
        let bytes = data
            .properties
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            + title.len();
        if self.bytes.saturating_add(bytes) > 8388608 {
            self.truncated = true;
            return Err("Agenda retained text exceeds 8 MiB".into());
        }
        self.bytes += bytes;
        Ok(())
    }
    fn push(
        &mut self,
        origin: (usize, &Raw),
        heading: &metadata::Heading,
        data: MatchData,
        limit: usize,
    ) -> Result<(), String> {
        let (source, node) = origin;
        self.retain_fields(&data, &heading.title)?;
        if self.rows.len() >= limit {
            self.truncated = true;
            return Err("Agenda row limit reached".into());
        }
        self.rows.push(QueryRow {
            title: Arc::from(heading.title.as_str()),
            task: heading.task.clone(),
            source,
            byte: node.source.start,
            stale: false,
            fields: data.properties,
            block: None,
            block_index: 0,
        });
        Ok(())
    }
}
