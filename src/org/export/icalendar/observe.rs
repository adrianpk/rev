use super::*;
mod stamps;
use stamps::*;
pub(super) fn entries(snapshot: &Snapshot, cancel: &Cancellation) -> Result<Vec<Entry>, String> {
    let properties = Properties::observe_nodes(snapshot.nodes.iter(), true, &snapshot.text, cancel);
    let mut entries: Vec<Entry> = Vec::new();
    let mut owners = Vec::with_capacity(snapshot.nodes.len());
    for (id, node) in snapshot.nodes.iter().enumerate() {
        document::check(cancel)?;
        let parent = node.parent.and_then(|p| owners[p]);
        let owner = if matches!(node.kind, StructureKind::Heading { .. }) {
            add_heading(&mut entries, snapshot, (id, node), parent)?;
            Some(entries.len() - 1)
        } else {
            parent
        };
        owners.push(owner);
        if let Some(entry) = owner
            .and_then(|p| entries.get_mut(p))
            .filter(|e| !e.excluded)
        {
            observe(entry, snapshot, node, cancel)?;
        }
    }
    metadata(&mut entries, snapshot, &properties, cancel)?;
    Ok(entries)
}
fn add_heading(
    entries: &mut Vec<Entry>,
    snapshot: &Snapshot,
    node: (usize, &structure::Raw),
    parent: Option<usize>,
) -> Result<(), String> {
    if entries.len() >= 4096 {
        return Err("Calendar export exceeds 4096 headings".into());
    }
    let (id, node) = node;
    let raw = snapshot.text.byte_slice(node.content.clone()).to_string();
    if raw.len() > 4096 {
        return Err("Calendar heading exceeds 4096 bytes".into());
    }
    if let Some(last) = entries.last_mut() {
        last.body.end = node.source.start;
    }
    let range = super::super::render::headings::title_range(
        &snapshot.text,
        node,
        &snapshot.options,
        &snapshot.todo,
    );
    let word = raw.split_whitespace().next().unwrap_or("");
    let excluded =
        word == "COMMENT" || archived(&raw) || parent.is_some_and(|p| entries[p].excluded);
    entries.push(Entry {
        node: id,
        summary: snapshot.text.byte_slice(range).to_string(),
        task: snapshot.workflow.state(word).map(|s| s.done),
        excluded,
        raw,
        body: node.opening.as_ref().expect("native heading").end..snapshot.text.len_bytes(),
        properties: BTreeMap::new(),
        drawers: Vec::new(),
        scheduled: None,
        deadline: None,
        appointments: Vec::new(),
    });
    Ok(())
}
fn archived(raw: &str) -> bool {
    raw.split_whitespace()
        .last()
        .is_some_and(|s| s.starts_with(':') && s.split(':').any(|t| t == "ARCHIVE"))
}
fn metadata(
    entries: &mut [Entry],
    snapshot: &Snapshot,
    properties: &Properties,
    cancel: &Cancellation,
) -> Result<(), String> {
    for entry in entries.iter_mut().filter(|e| !e.excluded) {
        document::check(cancel)?;
        let at = snapshot.nodes[entry.node].source.start;
        for key in [
            "ID",
            "SUMMARY",
            "DESCRIPTION",
            "LOCATION",
            "CLASS",
            "TIMEZONE",
        ] {
            if let Some(value) = properties.query_value(&snapshot.text, at, key, &[])? {
                entry.properties.insert(key, value);
            }
        }
    }
    Ok(())
}
fn observe(
    entry: &mut Entry,
    snapshot: &Snapshot,
    node: &structure::Raw,
    cancel: &Cancellation,
) -> Result<(), String> {
    if node.kind == StructureKind::PropertyDrawer {
        entry.drawers.push(node.source.clone());
    }
    if node.kind == StructureKind::Planning {
        return planning(
            entry,
            &snapshot.text.byte_slice(node.content.clone()).to_string(),
        );
    }
    if node.context == Some(InlineContext::Standard) {
        appointments(entry, snapshot, node.content.clone(), cancel)?;
    }
    Ok(())
}
