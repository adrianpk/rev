//! Column values are derived observations. They never update property drawers.
mod format;
mod summary;
mod table;
use crate::{
    buffer::{Span, edit::Replacement},
    config::Org,
    org::{
        Cancellation,
        agenda::{AgendaQuery, CalendarQuery, ClockScan, QueryLine, QueryResult, SourceScan},
        time::Zone,
    },
};
pub(crate) use format::{Format, Options};
pub(crate) struct Scan {
    pub sources: SourceScan,
    pub clocks: ClockScan,
    pub format: Format,
    pub context: CalendarQuery,
    pub policy: Org,
    pub maxlevel: usize,
    pub update: Option<Update>,
}
pub(crate) struct Update {
    pub source: Span,
    pub prefix: String,
    pub suffix: String,
    pub newline: String,
}
pub(crate) enum Output {
    View(Box<QueryResult>),
    Update(Replacement),
}
impl Scan {
    pub(crate) fn run(self, zone: &Zone, cancel: &Cancellation) -> Result<Output, String> {
        let bounded = cancel.bounded(
            std::time::Instant::now()
                + std::time::Duration::from_secs(self.policy.limits.job_seconds as u64),
        );
        self.observe(zone, &bounded)
    }
    fn observe(self, zone: &Zone, cancel: &Cancellation) -> Result<Output, String> {
        let sources = self.sources.run(cancel);
        let clocks = self.clocks.run(zone, cancel)?;
        let view = self.format.view();
        let query = AgendaQuery::block(&view.blocks[0], &self.policy.limits)?;
        let result = QueryResult::observe(&sources, query, self.context, &self.policy, cancel)?;
        let mut result = project(
            result,
            &clocks,
            (&self.format, &self.policy, self.maxlevel),
            cancel,
        )?;
        let diagnostic = diagnostic(&clocks);
        if let Some(update) = self.update {
            return update
                .render(&result, &self.format, &diagnostic, cancel)
                .map(Output::Update);
        }
        if !diagnostic.is_empty() {
            result.issues.push(crate::org::agenda::SourceIssue {
                path: result.sources.first().and_then(|s| s.path.clone()),
                message: diagnostic,
            });
        }
        result.view = Some(view);
        result.lines = std::iter::once(QueryLine::Block(0))
            .chain((0..result.rows.len()).map(QueryLine::Entry))
            .collect();
        Ok(Output::View(Box::new(result)))
    }
}

fn diagnostic(summary: &crate::org::agenda::ClockSummary) -> String {
    if summary.open == 0 && summary.unresolved == 0 && summary.overlaps == 0 {
        return String::new();
    }
    format!(
        "Clock records: open {}; unresolved {}; overlaps {}",
        summary.open, summary.unresolved, summary.overlaps
    )
}
impl Update {
    fn render(
        self,
        result: &QueryResult,
        format: &Format,
        diagnostic: &str,
        cancel: &Cancellation,
    ) -> Result<Replacement, String> {
        let body = table::render(result, format, &self.newline, cancel)?;
        let mut text = format!("{}{body}", self.prefix);
        if !diagnostic.is_empty() {
            text.push_str(&format!("# {diagnostic}{}", self.newline));
        }
        text.push_str(&self.suffix);
        if text.len() > 1048576 {
            return Err("Column report exceeds 1048576 bytes".into());
        }
        Ok(Replacement {
            source: self.source,
            text,
        })
    }
}

fn project(
    mut result: QueryResult,
    clocks: &crate::org::agenda::ClockSummary,
    data: (&Format, &Org, usize),
    cancel: &Cancellation,
) -> Result<QueryResult, String> {
    if !result.complete() {
        return Err("Column source observation is incomplete; source unchanged".into());
    }
    let selected: std::collections::BTreeSet<_> = clocks.rows.iter().map(|r| r.byte).collect();
    result
        .rows
        .retain(|r| !cancel.is_cancelled() && selected.contains(&r.byte));
    summary::clocks(&mut result, &clocks.rows, cancel)?;
    summary::properties(&mut result, data.0, data.1, cancel)?;
    result.rows.retain(|r| summary::level(r) <= data.2);
    if cancel.is_cancelled() {
        return Err("Column report cancelled; source unchanged".into());
    }
    Ok(result)
}
