mod headings;
use crate::{
    config::{CaptureKind, CaptureTarget, CaptureTemplate, Org},
    coordinates::Byte,
    org::{
        Cancellation, Coverage, Limits, StructureKind, io::DestinationSource, structure, time::Date,
    },
};
use headings::Headings;
use std::ops::Range;

/// Discovery offsets belong only to the retained source observation. Finalize
/// must revalidate actual canonical text and evidence before preparing edits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub parent: Option<Byte>,
    pub level: usize,
    pub body: Range<Byte>,
    pub subtree_end: Byte,
    pub missing: Vec<PlannedHeading>,
    pub missing_at: Option<Byte>,
    pub container: Container,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedHeading {
    pub level: usize,
    pub title: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Container {
    Body,
    List { bytes: Range<Byte>, ordered: bool },
    Table { bytes: Range<Byte> },
    MissingList,
    MissingTable,
}
impl Target {
    pub(in crate::org) fn resolve(
        source: &DestinationSource,
        template: &CaptureTemplate,
        policy: &Org,
        date: Date,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        let text = source.snapshot();
        let document = structure::observe(text.clone(), Limits::default(), cancel);
        if document.coverage != Coverage::Complete {
            return Err("Capture destination structural observation is incomplete".into());
        }
        if document
            .nodes
            .iter()
            .any(|n| n.kind == StructureKind::Malformed)
        {
            return Err("Malformed capture destination structure".into());
        }
        let headings = Headings::new(&document.nodes, &text, &policy.todo, cancel)?;
        let mut target = headings.target(&template.target, date)?;
        if target.level > 256 || template.kind == CaptureKind::Entry && target.level == 256 {
            return Err("Capture target exceeds the native 256-level structural profile".into());
        }
        target.container = container(&document.nodes, &target, template.kind, cancel)?;
        Ok(target)
    }
}
fn container(
    nodes: &[structure::Raw],
    target: &Target,
    kind: CaptureKind,
    cancel: &Cancellation,
) -> Result<Container, String> {
    let wanted = match kind {
        CaptureKind::Item | CaptureKind::Checkitem => Container::MissingList,
        CaptureKind::TableLine => Container::MissingTable,
        _ => return Ok(Container::Body),
    };
    if !target.missing.is_empty() {
        return Ok(wanted);
    }
    for node in nodes
        .iter()
        .filter(|n| target.body.start.0 <= n.source.start && n.source.end <= target.body.end.0)
    {
        if cancel.is_cancelled() {
            return Err("Capture destination discovery cancelled".into());
        }
        if let Some(found) = observed_container(kind, node) {
            return Ok(found);
        }
    }
    Ok(wanted)
}

fn observed_container(kind: CaptureKind, node: &structure::Raw) -> Option<Container> {
    let bytes = Byte(node.source.start)..Byte(node.source.end);
    match (kind, node.kind) {
        (CaptureKind::Item | CaptureKind::Checkitem, StructureKind::List { ordered }) => {
            Some(Container::List { bytes, ordered })
        }
        (CaptureKind::TableLine, StructureKind::Table) => Some(Container::Table { bytes }),
        _ => None,
    }
}
impl Target {
    fn plan(&mut self, title: String) {
        self.missing_at.get_or_insert(self.subtree_end);
        self.level += 1;
        self.missing.push(PlannedHeading {
            level: self.level,
            title,
        });
    }
}
