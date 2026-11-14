use super::*;
use rev::buffer::edit::Replacement;
use rev::coordinates::{Coordinates, LineByte, Scalar};
use rev::syntax::Style;

const SOURCE: &str = "* Root\nRoot body.\n** Child\nChild body.\n*** Deep\nDeep body.\n";

fn setup_policies(config: &str) -> (Fixture, Buffers, View) {
    setup_source(config, SOURCE)
}

fn setup_source(config: &str, source: &str) -> (Fixture, Buffers, View) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&fixture.write("notes.org", source)).unwrap();
    let mut view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    view.gutter.apply(rev::gutter::Change::Visible(false));
    view.bookmark_markers = false;
    observe(buffers.get_mut(id).unwrap(), Limits::default());
    (fixture, buffers, view)
}

fn change(buffer: &mut Buffer, view: &mut View, row: usize, value: &str) {
    let coordinates = Coordinates::new(buffer).unwrap();
    let start = coordinates.from_line(LineByte { row, column: 4 }).unwrap();
    let edit = Replacement {
        source: buffer.span(start.0..start.0).unwrap(),
        text: value.into(),
    };
    let mut points = [view.point];
    let prepared = buffer.prepare(&[edit], &points, None).unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    view.point = points[0];
}

fn frame(buffer: &Buffer, view: &View) -> Frame {
    Frame::project(buffer, view, SIZE, "").unwrap()
}

fn samples(frame: &Frame) -> Vec<(usize, usize, surface::Cell)> {
    let cells = surface::draw(frame);
    let mut result = Vec::new();
    for (row, first) in [(0, 'R'), (2, 'C'), (4, 'D')] {
        let title = frame.rows()[row]
            .text()
            .chars()
            .position(|ch| ch == first)
            .unwrap();
        let marker = frame.rows()[row]
            .text()
            .chars()
            .position(|ch| !ch.is_whitespace())
            .unwrap();
        result.push((row, marker, cells[&(row, marker)]));
        result.push((row, title, cells[&(row, title)]));
    }
    result
}

// These frames are deliberately rendered without launching a worker: every
// edit remains pending, exposing any literal/default frame before exact data.
#[test]
fn continuity_frames() {
    for (indent, pretty) in [(true, true), (true, false), (false, true), (false, false)] {
        for colors in [true, false] {
            let config = format!(
                "org.indent={indent}\norg.headings.pretty={pretty}\nsyntax.enabled={colors}"
            );
            check_frames(&config);
        }
    }
}

fn check_frames(config: &str) {
    let (_fixture, mut buffers, mut view) = setup_policies(config);
    let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
    let baseline = samples(&frame(buffer, &view));
    let base = buffer.stamp();
    for (row, inserted) in [(3, "x"), (2, "界"), (3, "e\u{301}"), (2, "z"), (3, "q")] {
        change(buffer, &mut view, row, inserted);
        assert_eq!(
            buffer.org_highlights().map(|h| h.stamp()),
            if config.contains("syntax.enabled=true") {
                Some(base)
            } else {
                None
            }
        );
        assert_eq!(
            samples(&frame(buffer, &view)),
            baseline,
            "{config}, row {row}"
        );
    }
    let pending = frame(buffer, &view);
    let source = text(buffer);
    observe(buffer, Limits::default());
    assert_eq!(samples(&frame(buffer, &view)), baseline, "exact {config}");
    assert_eq!(text(buffer), source);
    assert_eq!(
        pending.rows()[4].text(),
        frame(buffer, &view).rows()[4].text()
    );
    assert_eq!(buffer.history().len(), 5);
}

// Before and after classification reject boundary changes, unsupported literal
// contexts and the native ASCII-space separator. Fresh exact evidence restores
// decoration without granting the carried display structural action authority.
#[test]
fn continuity_refusals() {
    for (name, source, range, inserted) in [
        ("star", SOURCE, 0..1, ""),
        ("separator", SOURCE, 1..2, ""),
        ("newline", SOURCE, 8..8, "\n"),
        (
            "drawer",
            "* Root\n:PROPERTIES:\n:END:\n** Child\n",
            7..8,
            "x",
        ),
        (
            "block",
            "* Root\n#+begin_src text\n,* Literal\n#+end_src\n** Child\n",
            7..8,
            "x",
        ),
    ] {
        let (_f, mut buffers, mut view) = setup_policies("org.indent=false");
        let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
        let edit = Replacement {
            source: buffer.span(0..buffer.len_chars()).unwrap(),
            text: source.into(),
        };
        let prepared = buffer.prepare(&[edit], &[view.point], None).unwrap();
        let mut points = [view.point];
        buffer.publish(prepared, &mut points).unwrap();
        view.point = points[0];
        observe(buffer, Limits::default());
        let edit = Replacement {
            source: buffer.span(range).unwrap(),
            text: inserted.into(),
        };
        let prepared = buffer.prepare(&[edit], &points, None).unwrap();
        buffer.publish(prepared, &mut points).unwrap();
        view.point = points[0];
        let pending = frame(buffer, &view);
        assert!(
            !pending.rows().iter().any(|r| r.text().contains('◉')),
            "{name}"
        );
    }
}

// A pending title keeps its structural face while uncertain lexical captures
// on unchanged body rows disappear. Current source bytes supply heading paint.
#[test]
fn continuity_faces() {
    let source = SOURCE.replace("Child body.", "Child *body*.");
    let (_f, mut buffers, mut view) = setup_source("org.indent=true", &source);
    let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
    assert!(!frame(buffer, &view).rows()[3].syntax().is_empty());
    change(buffer, &mut view, 2, "*");
    let current = frame(buffer, &view);
    assert!(
        current.rows()[2]
            .syntax()
            .iter()
            .all(|p| p.style == Style::Org(rev::org::Role::Heading))
    );
    assert!(
        current.rows()[2]
            .syntax()
            .iter()
            .any(|p| p.style == Style::Org(rev::org::Role::Heading))
    );
    assert!(current.rows()[3].syntax().is_empty());
    assert_eq!(view.point.position, Scalar(0));
}

// Carried structure uses the active level palette rather than inheriting a
// retained capture color or requiring a current lexical highlight stamp.
#[test]
fn continuity_palette() {
    let (fixture, mut buffers, mut view) = setup_policies("palette.org.heading_2='#123456'");
    let settings = prepare(&fixture.0.join("config.toml")).snapshot;
    let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
    change(buffer, &mut view, 3, "z");
    let current = frame(buffer, &view).with_palette(&settings.settings().palette);
    let sampled = samples(&current);
    assert_eq!(
        sampled[2].2.foreground,
        Some(rev::config::Rgb(0x12, 0x34, 0x56))
    );
    assert_eq!(
        sampled[3].2.foreground,
        Some(rev::config::Rgb(0x12, 0x34, 0x56))
    );
}

// The native carry displays Unicode edits without changing copied source,
// BOM/CRLF output, read-only admission or the single canonical undo entry.
#[test]
fn continuity_source() {
    let source = "* Root\r\nbody 界e\u{301}\r\n** Child\r\n";
    let (fixture, mut buffers, mut view) = setup_source("", &format!("\u{feff}{source}"));
    let id = view.point.stamp.buffer;
    let buffer = buffers.get_mut(id).unwrap();
    change(buffer, &mut view, 1, "z");
    let current = source.replacen("body", "bodyz", 1);
    assert_eq!(text(buffer), current);
    assert!(frame(buffer, &view).rows()[2].text().contains('○'));
    assert_eq!(buffer.history().len(), 1);
    let stamp = buffer.stamp();
    buffer.set_readonly(true);
    let forbidden = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "x".into(),
    };
    assert!(buffer.prepare(&[forbidden], &[view.point], None).is_err());
    assert_eq!(buffer.stamp(), stamp);
    assert!(frame(buffer, &view).rows()[2].text().contains('○'));
    buffer.set_readonly(false);
    buffers.save(id).unwrap();
    assert_eq!(
        std::fs::read(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{current}").as_bytes()
    );
    let buffer = buffers.get_mut(id).unwrap();
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.history().len(), 1);
    let mut points = [view.point];
    buffer.navigate(History::Undo, &mut points, None).unwrap();
    view.point = points[0];
    assert_eq!(text(buffer), source);
    assert!(!frame(buffer, &view).rows()[2].text().contains('○'));
}

// A delimiter outside either viewport edge can affect a visible cross-row
// capture. Carry drops the whole uncertain lexical context, including that row.
#[test]
fn continuity_viewport_context() {
    let source = "** Root\nordinary *open\ncontinued* tail\n*** Deep\nbody\n";
    for (top, needle) in [(2, "*open"), (1, "* tail")] {
        let (_f, mut buffers, mut view) = setup_source("org.indent=true", source);
        view.top = top;
        let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
        let size = Size {
            width: 40,
            height: 3,
        };
        let before = Frame::project(buffer, &view, size, "").unwrap();
        assert!(
            before.rows()[0]
                .syntax()
                .iter()
                .any(|paint| paint.style == Style::Org(rev::org::Role::Emphasis))
        );
        let at = source.find(needle).unwrap();
        let edit = rev::buffer::edit::Replacement {
            source: buffer.span(at..at + 1).unwrap(),
            text: String::new(),
        };
        let mut points = [view.point];
        let prepared = buffer.prepare(&[edit], &points, None).unwrap();
        buffer.publish(prepared, &mut points).unwrap();
        view.point = points[0];
        let current = Frame::project(buffer, &view, size, "").unwrap();
        assert!(current.rows()[0].syntax().is_empty());
        let mut full = view.clone();
        full.top = 0;
        assert!(frame(buffer, &full).rows()[3].text().contains('✸'));
        observe(buffer, Limits::default());
        assert!(
            Frame::project(buffer, &view, size, "").unwrap().rows()[0]
                .syntax()
                .is_empty()
        );
    }
}
