use crate::config::StartupVisibility;
use crate::org::{Cancellation, Structure, StructureKind};

use super::{Level, Outline, Visibility};

// Reuse structural ownership and its finite source/node limits. Keywords in
// drawers can be parsed as children, but never become document startup metadata.
pub(super) fn declaration(
    model: &Outline,
    structure: &Structure,
    text: &ropey::Rope,
    cancel: &Cancellation,
) -> Option<StartupVisibility> {
    // A keyword's own value is literal text for coloring. Protect enclosing
    // literal forms here, rather than accidentally excluding every keyword.
    let mut protected: Vec<_> = structure
        .raw_nodes()
        .filter(|node| {
            node.kind == StructureKind::Malformed
                || matches!(node.kind, StructureKind::Block(_))
                    && node.context == Some(crate::org::InlineContext::Literal)
        })
        .map(|node| node.source.clone())
        .collect();
    protected.extend(model.drawers.iter().map(|d| d.start..d.end));
    super::model::merge(&mut protected);
    let mut found = None;
    let mut last = 0;
    for node in structure
        .raw_nodes()
        .filter(|n| n.kind == StructureKind::Keyword)
    {
        if cancel.is_cancelled() {
            return None;
        }
        let index = protected.partition_point(|range| range.end <= node.source.start);
        if protected
            .get(index)
            .is_some_and(|r| r.contains(&node.source.start))
        {
            continue;
        }
        let Some(name) = &node.name else { continue };
        if !text
            .byte_slice(name.clone())
            .to_string()
            .eq_ignore_ascii_case("STARTUP")
        {
            continue;
        }
        for word in text
            .byte_slice(node.content.clone())
            .to_string()
            .split_ascii_whitespace()
        {
            if cancel.is_cancelled() {
                return None;
            }
            let policy = [
                ("overview", StartupVisibility::Overview),
                ("content", StartupVisibility::Content),
                ("showall", StartupVisibility::Showall),
                ("showeverything", StartupVisibility::Showeverything),
            ]
            .into_iter()
            .find_map(|(name, policy)| word.eq_ignore_ascii_case(name).then_some(policy));
            if policy.is_some() && node.source.start >= last {
                found = policy;
                last = node.source.start;
            }
        }
    }
    found
}

impl Outline {
    pub(crate) fn startup_ready(&self) -> bool {
        self.complete().is_ok() && self.malformed.is_empty()
    }
}

impl Visibility {
    pub(crate) fn startup(&self, configured: StartupVisibility) -> Result<Self, &'static str> {
        if !self.model.startup_ready() {
            return Err("Org startup observation is incomplete or malformed");
        }
        let mut next = Self::new(self.model.clone());
        match self.model.startup.unwrap_or(configured) {
            StartupVisibility::Overview => {
                next.levels.fill(Level::Closed);
                next.drawers.fill(true);
                next.global = 1;
            }
            StartupVisibility::Content => {
                next.levels.fill(Level::Children);
                next.drawers.fill(true);
                next.global = 2;
            }
            StartupVisibility::Showall => next.drawers.fill(true),
            StartupVisibility::Showeverything => {}
        }
        Ok(next)
    }
}
