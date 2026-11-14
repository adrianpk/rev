use super::tests::edit;
use super::{Mapping, ROW_LIMIT};
use crate::buffer::{Buffer, Buffers, Mode, edit::History};
use crate::org::{Cancellation, ColorScan, Coverage, Limits};
use ropey::Rope;

fn observe(buffer: &mut Buffer) {
    if buffer.is_org() {
        super::tests::analyze(buffer);
    } else {
        let work = crate::markdown::Work::new(buffer.stamp(), buffer.text.clone());
        buffer.install_markdown(work.run(&Cancellation::default()).unwrap());
    }
}

fn observed(mode: Mode) -> Buffer {
    let marker = if mode == Mode::Org { "**" } else { "##" };
    let source = format!("{marker} Heading\n{}", "ordinary body\n".repeat(10));
    let mut buffer = Buffer::from_rope(Rope::from_str(&source)).unwrap();
    buffer.select_mode(mode).unwrap();
    observe(&mut buffer);
    buffer
}

fn carry(buffer: &mut Buffer) {
    let at = buffer.text.line_to_char(1) + 4;
    edit(buffer, at..at, "界");
    assert!(buffer.presentation.is_some());
}

fn transition(buffer: &mut Buffer, name: &str) {
    match name {
        "undo" => buffer.navigate(History::Undo, &mut [], None).unwrap(),
        "redo" => {
            buffer.navigate(History::Undo, &mut [], None).unwrap();
            observe(buffer);
            buffer.navigate(History::Redo, &mut [], None).unwrap();
        }
        "reload" => {
            let text = Rope::from_str(&buffer.text.to_string().replace("body", "next"));
            let prepared = buffer.prepare_reload_text(&text, false, &[]).unwrap();
            buffer.publish(prepared, &mut []).unwrap();
        }
        "mode" => buffer.select_mode(Mode::Text).unwrap(),
        "syntax policy" => {
            let settings = crate::config::Snapshot::default();
            buffer.syntax_policy.enabled = !settings.settings().syntax.enabled;
            buffer.apply_settings(&settings);
        }
        "boundary" => edit(buffer, 0..0, "\n"),
        _ => unreachable!("named retirement case"),
    }
}

// A retired chain cannot bridge the next key. Both native formats require a
// fresh complete base after history, reload, mode, policy or boundary changes.
#[test]
fn retirement_matrix() {
    for mode in [Mode::Org, Mode::Markdown] {
        for name in [
            "undo",
            "redo",
            "reload",
            "mode",
            "syntax policy",
            "boundary",
        ] {
            let mut buffer = observed(mode);
            carry(&mut buffer);
            transition(&mut buffer, name);
            assert!(buffer.presentation.is_none(), "{mode:?}/{name}");
            buffer.select_mode(mode).unwrap();
            let at = buffer.text.line_to_char(2) + 4;
            edit(&mut buffer, at..at, "x");
            assert!(buffer.presentation.is_none(), "gap: {mode:?}/{name}");
            observe(&mut buffer);
            carry(&mut buffer);
        }
    }
}

// Distinct-row exhaustion is format admission, not merely a synthetic map
// operation: all eight ordinary body rows carry and the ninth publishes literal.
#[test]
fn native_row_ceiling() {
    for mode in [Mode::Org, Mode::Markdown] {
        let mut buffer = observed(mode);
        let base = buffer.stamp();
        for row in 1..=ROW_LIMIT {
            let at = buffer.text.line_to_char(row) + 4;
            edit(&mut buffer, at..at, "界");
            let mapping = buffer.presentation.unwrap();
            assert_eq!(mapping.base, base);
            assert_eq!(mapping.rows.iter().flatten().count(), row);
        }
        let at = buffer.text.line_to_char(ROW_LIMIT + 1) + 4;
        edit(&mut buffer, at..at, "界");
        assert!(buffer.presentation.is_none());
        assert_eq!(buffer.history().len(), ROW_LIMIT + 1);
        edit(&mut buffer, at..at, "z");
        assert!(buffer.presentation.is_none());
    }
    println!("Native row ceiling: accepted=16, rejected=2, no gap reuse=2");
}

// Replacing an exact result can clear a carry but an obsolete result cannot.
// Partial/canceled Org coverage is unusable as a future presentation base.
#[test]
fn observation_retirement() {
    for (name, limits, cancel) in [
        ("byte limit", Limits::new(4, 200, 20).unwrap(), false),
        ("node limit", Limits::new(4096, 1, 20).unwrap(), false),
        ("cancelled", Limits::default(), true),
    ] {
        let mut buffer = observed(Mode::Org);
        let stale = ColorScan::new(&buffer, Limits::default())
            .unwrap()
            .run(&Cancellation::default());
        carry(&mut buffer);
        let mapping = buffer.presentation;
        assert!(buffer.install_org_coloring(stale).is_err());
        assert_eq!(buffer.presentation, mapping);
        let cancellation = Cancellation::default();
        if cancel {
            cancellation.cancel();
        }
        let coloring = ColorScan::new(&buffer, limits).unwrap().run(&cancellation);
        assert_ne!(
            coloring.highlights().coverage(),
            Coverage::Complete,
            "{name}"
        );
        buffer.install_org_coloring(coloring).unwrap();
        assert!(buffer.presentation.is_none());
        let at = buffer.text.line_to_char(1) + 4;
        edit(&mut buffer, at..at, "x");
        assert!(buffer.presentation.is_none(), "{name}");
    }
    let mut buffer = observed(Mode::Markdown);
    let stale = crate::markdown::Work::new(buffer.stamp(), buffer.text.clone())
        .run(&Cancellation::default())
        .unwrap();
    carry(&mut buffer);
    let mapping = buffer.presentation;
    buffer.install_markdown(stale);
    assert_eq!(buffer.presentation, mapping);
    buffer.retire_markdown_presentation();
    assert!(buffer.markdown.observation.is_none());
    assert!(buffer.presentation.is_none());
    observe(&mut buffer);
    assert!(buffer.markdown_observation().is_some());
}

// Explicit cancellation suppresses automatic retries, while a new explicit
// request can start exactly one existing work slot. Closing drops the old ID.
#[test]
fn cancel_and_close() {
    for mode in [Mode::Org, Mode::Markdown] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.insert(observed(mode));
        carry(buffers.get_mut(id).unwrap());
        buffers.retire_org_presentations();
        buffers.retire_markdown_presentations();
        let buffer = buffers.get_mut(id).unwrap();
        assert!(buffer.presentation.is_none());
        retry(buffer, mode);
        let kill = buffers.prepare_kill(id).unwrap();
        buffers.kill(kill).unwrap();
        assert!(buffers.get(id).is_err());
        let next = buffers.insert(observed(mode));
        assert_ne!(next, id);
        assert!(buffers.get(next).unwrap().presentation.is_none());
    }
}

fn retry(buffer: &mut Buffer, mode: Mode) {
    if mode == Mode::Org {
        assert!(!buffer.needs_org_outline());
        buffer.request_org_outline();
        assert!(buffer.needs_org_outline());
        buffer.attempted_org_colors();
        assert!(!buffer.needs_org_outline());
    } else {
        assert!(buffer.markdown_work().is_none());
        buffer.request_markdown();
        assert!(buffer.markdown_work().is_some());
        assert!(buffer.markdown_work().is_none());
    }
}

// Zero-net Unicode replacements still dirty the row. Hundreds of real edits
// retain one complete base and two fixed slots, with no per-key map allocation.
#[test]
fn sustained_bounds() {
    for mode in [Mode::Org, Mode::Markdown] {
        let mut buffer = observed(mode);
        let base = buffer.stamp();
        for step in 0..512 {
            let row = 1 + step % 2;
            let at = buffer.text.line_to_char(row) + 4;
            if step % 4 < 2 {
                edit(&mut buffer, at..at, "界");
            } else {
                edit(&mut buffer, at..at + 1, "");
            }
            let mapping = buffer.presentation.unwrap();
            assert_eq!(mapping.base, base);
            assert_eq!(mapping.current, buffer.stamp());
            assert!(mapping.rows.iter().flatten().count() <= 2);
        }
        let mapping = buffer.presentation.unwrap();
        assert_eq!(mapping.rows.iter().flatten().count(), 2);
        assert!(mapping.rows.iter().flatten().all(|row| row.delta == 0));
        assert_eq!(
            mapping.base_byte(10, buffer.text.line_to_byte(10)),
            Some(buffer.text.line_to_byte(10))
        );
        println!(
            "Carry storage {mode:?}: edits=512, dirty_rows=2, capacity={ROW_LIMIT}, mapping_bytes={}, optional_mapping_bytes={}",
            std::mem::size_of::<Mapping>(),
            std::mem::size_of::<Option<Mapping>>()
        );
    }
}
