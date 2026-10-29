//! One bounded job observes a configured block view over canonical snapshots.
mod clocks;
mod rows;
use super::*;
use crate::{
    config::{AgendaBlock, AgendaKind, AgendaView, Org},
    org::Cancellation,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
pub struct CustomView;
impl CustomView {
    pub fn observe(
        sources: &SourceSet,
        view: AgendaView,
        context: CalendarQuery,
        policy: &Org,
        cancel: &Cancellation,
    ) -> Result<QueryResult, String> {
        view.validate()?;
        for block in &view.blocks {
            AgendaQuery::block(block, &policy.limits)?;
        }
        let bounded =
            cancel.bounded(Instant::now() + Duration::from_secs(policy.limits.job_seconds as u64));
        let query = AgendaQuery::parse(QueryKind::Tags, "LEVEL>=1", &policy.limits)?;
        let mut output = QueryResult {
            query,
            context,
            sources: Vec::new(),
            rows: Vec::new(),
            issues: sources.issues.clone(),
            truncated: sources.truncated,
            cancelled: sources.cancelled,
            view: Some(view.clone()),
        };
        blocks(&mut output, sources, &view, policy, &bounded)?;
        output.cancelled |= bounded.is_cancelled() && !bounded.expired();
        output.truncated |= bounded.expired();
        Ok(output)
    }
}
fn blocks(
    output: &mut QueryResult,
    sources: &SourceSet,
    view: &AgendaView,
    policy: &Org,
    bounded: &Cancellation,
) -> Result<(), String> {
    for (index, block) in view.blocks.iter().enumerate() {
        if bounded.is_cancelled() {
            break;
        }
        let subset = subset(sources, &block.sources);
        let block_context = match block.span {
            Some(span) => output.context.at(output.context.date, span)?,
            None => output.context.clone(),
        };
        let result = observe(&subset, block, block_context, policy, bounded)?;
        rows::append(output, result, (index, block), policy)?;
        if output.rows.len() >= policy.limits.agenda_rows {
            output.truncated = true;
            break;
        }
    }
    Ok(())
}

fn subset(sources: &SourceSet, paths: &[String]) -> SourceSet {
    let documents: Vec<_> = sources
        .documents
        .iter()
        .filter(|s| {
            paths.is_empty()
                || paths.iter().any(|root| {
                    let root = Path::new(root);
                    s.path.as_ref().is_some_and(|p| matches_path(p, root))
                        || s.aliases.iter().any(|p| matches_path(p, root))
                })
        })
        .cloned()
        .collect();
    let bytes = documents.iter().map(|s| s.text().len_bytes()).sum();
    SourceSet {
        documents,
        bytes,
        ..SourceSet::default()
    }
}
fn matches_path(path: &Path, root: &Path) -> bool {
    path == root || (path.parent() == Some(root) && path.extension().is_some_and(|s| s == "org"))
}
fn observe(
    sources: &SourceSet,
    block: &AgendaBlock,
    context: CalendarQuery,
    policy: &Org,
    cancel: &Cancellation,
) -> Result<QueryResult, String> {
    let query = AgendaQuery::block(block, &policy.limits)?;
    let mut result = QueryResult::observe(sources, query, context.clone(), policy, cancel)?;
    match block.kind {
        AgendaKind::Calendar => rows::calendar(
            &mut result,
            Calendar::observe(sources, context, policy, cancel)?,
        ),
        AgendaKind::Clock => clocks::observe(&mut result, sources, policy, cancel)?,
        _ => (),
    }
    rows::sort(&mut result.rows, &block.sort, &policy.duration.units);
    Ok(result)
}
