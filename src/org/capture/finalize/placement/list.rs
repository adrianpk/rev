use super::*;
use crate::org::{Coverage, Limits, StructureKind, structure};
pub(super) fn insert(
    resolved: &Resolved,
    draft: &Rope,
    newline: &str,
    cancel: &Cancellation,
) -> Result<Vec<CaptureEdit>, String> {
    let Container::List { bytes, ordered } = &resolved.target.container else {
        unreachable!("selected list container")
    };
    let source = resolved.source.snapshot();
    let old = source.byte_slice(bytes.start.0..bytes.end.0).to_string();
    let (start, marker) = style(&old, cancel)?;
    let mut payload = item(draft, &resolved.target.container, newline)?;
    payload = crate::org::list::capture_style(&Rope::from_str(&payload), &marker, cancel)?;
    if !payload.ends_with('\n') {
        payload.push_str(newline)
    }
    let at = if resolved.template.prepend {
        start
    } else {
        content_end(&old)
    };
    let (mut payload, changes) = if *ordered {
        crate::org::list::capture_markers(
            &Rope::from_str(&old),
            &Rope::from_str(&payload),
            resolved.template.prepend,
            cancel,
        )?
    } else {
        (payload, Vec::new())
    };
    if at > 0 && !old[..at].ends_with('\n') {
        payload.insert_str(0, newline)
    }
    Ok(edits(bytes.start.0, at, payload, changes))
}
fn edits(
    start: usize,
    at: usize,
    payload: String,
    changes: Vec<(std::ops::Range<usize>, String)>,
) -> Vec<CaptureEdit> {
    let absolute = start + at;
    let mut edits = vec![CaptureEdit {
        bytes: absolute..absolute,
        text: payload,
    }];
    for (range, text) in changes {
        let range = start + range.start..start + range.end;
        if range.start == absolute {
            edits[0].bytes.end = range.end;
            edits[0].text.push_str(&text);
        } else {
            edits.push(CaptureEdit { bytes: range, text })
        }
    }
    edits.sort_by_key(|edit| edit.bytes.start);
    edits
}
fn style(text: &str, cancel: &Cancellation) -> Result<(usize, String), String> {
    let rope = Rope::from_str(text);
    let document = structure::observe(rope.clone(), Limits::default(), cancel);
    if document.coverage != Coverage::Complete {
        return Err("Capture list is incomplete".into());
    }
    let item = document
        .nodes
        .iter()
        .find(|n| matches!(n.kind, StructureKind::Item { .. }))
        .ok_or("Capture list item is unavailable")?;
    let opening = rope
        .byte_slice(item.source.start..item.content.start)
        .to_string();
    Ok((item.source.start, opening.trim_end().to_owned()))
}
