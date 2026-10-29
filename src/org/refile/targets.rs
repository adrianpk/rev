use super::*;
use crate::org::{
    Coverage, StructureKind,
    agenda::{AgendaSource, SourceSet},
    structure,
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Target {
    pub label: String,
    pub document: usize,
    pub bytes: Range<usize>,
    pub level: usize,
}
pub struct Targets {
    pub sources: SourceSet,
    pub headings: Vec<Target>,
}
impl Targets {
    pub fn observe(
        sources: SourceSet,
        limit: usize,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        if !sources.complete() {
            return Err(sources.feedback());
        }
        let mut result = Self {
            sources,
            headings: Vec::new(),
        };
        let mut labels = 0usize;
        for (index, source) in result.sources.documents.iter().enumerate() {
            let document = structure::observe(source.snapshot(), Limits::default(), cancel);
            if document.coverage != Coverage::Complete
                || document
                    .nodes
                    .iter()
                    .any(|n| n.kind == StructureKind::Malformed)
            {
                return Err("Refile target structure is incomplete or malformed".into());
            }
            for node in &document.nodes {
                let StructureKind::Heading { level } = node.kind else {
                    continue;
                };
                if cancel.is_cancelled() {
                    return Err("Refile target discovery cancelled".into());
                }
                if result.headings.len() >= limit.min(4096) {
                    return Err("Refile target limit reached".into());
                }
                let label = label(source, node)?;
                labels = labels.saturating_add(label.len());
                if labels > 1048576 {
                    return Err("Refile target labels exceed 1 MiB".into());
                }
                result.headings.push(Target {
                    label,
                    document: index,
                    bytes: node.source.clone(),
                    level,
                });
            }
        }
        Ok(result)
    }
    /// A number explicitly selects the displayed locator. Text must match one
    /// complete label; duplicate labels never choose an arbitrary destination.
    pub fn select(&self, input: &str) -> Result<usize, String> {
        if let Ok(number) = input.parse::<usize>() {
            return number
                .checked_sub(1)
                .filter(|i| *i < self.headings.len())
                .ok_or("Enter a displayed refile target number".into());
        }
        let mut matches = self
            .headings
            .iter()
            .enumerate()
            .filter(|(_, h)| h.label == input);
        let (index, _) = matches
            .next()
            .ok_or("Enter a complete refile target label or number")?;
        if matches.next().is_some() {
            return Err("Ambiguous refile target; select its displayed number".into());
        }
        Ok(index)
    }
}
fn label(source: &AgendaSource, node: &structure::Raw) -> Result<String, String> {
    let open = node
        .opening
        .as_ref()
        .ok_or("Refile heading has no opening")?;
    if open.len() > 4096 {
        return Err("Refile heading label exceeds 4096 bytes".into());
    }
    let text = source.snapshot();
    let title = text.byte_slice(open.clone()).to_string();
    let path = source
        .path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{:?}", source.version));
    Ok(format!(
        "{path}:{} {}",
        text.byte_to_line(open.start) + 1,
        title.trim()
    ))
}
