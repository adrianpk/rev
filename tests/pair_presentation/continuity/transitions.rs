use crossterm::event::{Event, KeyCode};
use rev::command::{Command, History, Motion, RegionCommand, Split};
use rev::display::Frame;
use std::time::{Duration, Instant};

use super::fixture::{Case, Editor, SOURCES};

fn case(language: &'static str) -> Case {
    Case {
        language,
        profile: "non-modal",
        style: "depth",
    }
}

// Boundary edits immediately discard old coordinates before any qualifier can
// run. Every later completed writer frame maps only current source endpoints.
#[test]
fn invalidating_edits() {
    let source = SOURCES[0].1;
    for inserted in ["(", "\"", "\\", "\n"] {
        let at = source.find("leaf").unwrap() + 2;
        let mut e = Editor::new(case("plain"), source, at);
        e.settle(|_, _| {});
        e.session.replace(at..at, inserted).unwrap();
        let expected = format!("{}{}{}", &source[..at], inserted, &source[at..]);
        assert_eq!(e.source(), expected);
        let frame = Frame::project(
            e.session.buffer().unwrap(),
            e.session.view(),
            rev::display::Size {
                width: 120,
                height: 10,
            },
            "",
        )
        .unwrap();
        assert!(
            crate::rendering::marks(&frame).is_empty(),
            "old facts cannot paint {inserted:?}"
        );
        e.settle(|frame, cells| current(frame, cells, &expected));
        e.session.execute(Command::History(History::Undo));
        e.settle(|frame, cells| current(frame, cells, source));
        assert_eq!(e.source(), source);
    }
}

fn current(frame: &Frame, cells: &super::observe::Cells, source: &str) {
    let chars: Vec<_> = source.chars().collect();
    for (at, row, column) in crate::rendering::marks(frame) {
        assert!(at < chars.len());
        assert!("()[]{}".contains(chars[at]));
        assert_eq!(cells[&(row, column)].character, chars[at]);
    }
}

// Shared source edits retain separate split policy and point state. Repeated
// redraw/resize reprojects current coordinates rather than a retained row cache.
#[test]
fn split_and_resize() {
    let source = SOURCES[1].1;
    let at = source.find("leaf").unwrap() + 2;
    let mut e = Editor::new(case("org"), source, at);
    e.settle(|_, _| {});
    e.session.execute(Command::Split(Split::Right));
    let ids = e.session.windows().order();
    e.session.select_view(ids[1]).unwrap();
    choose(&mut e, "off");
    e.session.select_view(ids[0]).unwrap();
    let sibling = e.session.windows().get(ids[1]).unwrap().point.position.0;
    e.handle(KeyCode::Char('x'));
    let expected = source.replacen("leaf", "lexaf", 1);
    for (width, height) in [(120, 10), (100, 12), (140, 14)] {
        e.session.handle(Event::Resize(width, height));
        let frame = e.settle(|frame, cells| current(frame, cells, &expected));
        assert_eq!(crate::rendering::marks(&frame).len(), 6);
    }
    assert_eq!(e.source(), expected);
    assert_eq!(
        e.session.windows().get(ids[1]).unwrap().point.position.0,
        sibling + 1
    );
    assert_eq!(e.session.remaining_input(super::now()), None);
}

fn choose(e: &mut Editor, style: &str) {
    e.session.execute(Command::PairHighlightStyle);
    e.session.handle(Event::Paste(style.into()));
    e.handle(KeyCode::Enter);
}

// Mark, source and cursor use canonical scalar/grapheme coordinates through a
// neutral edit with Unicode, CRLF and narrow wrapping. Read-only refusal is exact.
#[test]
fn fidelity_and_selection() {
    let source = "* Heading\r\nBody (界e\u{301}[leaf]) tail\r\n";
    let point = source.chars().position(|c| c == 'f').unwrap() - 1;
    let anchor = source.chars().position(|c| c == ')').unwrap() + 1;
    let mut e = Editor::new(case("org"), source, anchor);
    e.settle(|_, _| {});
    e.session.execute(Command::Region(RegionCommand::Set));
    for _ in point..anchor {
        e.session.execute(Command::Motion(Motion::BackwardChar));
    }
    let selected = e.settle(|_, _| {});
    assert!(selected.rows().iter().any(|row| row.selection().is_some()));
    let mark = e.session.buffer().unwrap().mark().unwrap();
    e.session.replace(point..point, "x").unwrap();
    let expected = source.replacen("leaf", "lexaf", 1);
    e.session.handle(Event::Resize(16, 20));
    e.settle(|frame, cells| current(frame, cells, &expected));
    assert_eq!(e.session.buffer().unwrap().mark().unwrap().0, mark.0 + 1);
    assert!(
        !e.session.buffer().unwrap().mark_active(),
        "ordinary source edits deactivate the transient region"
    );
    assert_eq!(e.session.view().point.position.0, point + 1);
    assert_eq!(e.source(), expected);
    let folded = "* H\n(hidden 界[leaf]\nbody\n)\n* Tail\n";
    let mut read_only = Editor::prepared(case("org"), folded, 0, |b, view| {
        let hidden = rev::org::Source::new(b)
            .range(rev::coordinates::Byte(4)..rev::coordinates::Byte(folded.find(')').unwrap()))
            .unwrap();
        view.set_folds(b, &[hidden]).unwrap();
        b.set_readonly(true);
    });
    let initial = read_only.settle(|_, _| {});
    read_only.handle(KeyCode::Char('x'));
    let next = read_only.settle(|frame, cells| current(frame, cells, folded));
    assert_eq!(crate::rendering::marks(&next).len(), 1);
    assert_eq!(
        crate::rendering::marks(&initial),
        crate::rendering::marks(&next)
    );
    assert_eq!(read_only.source(), folded);
    assert!(read_only.session.buffer().unwrap().history().is_empty());
}

// Reload can finish after several complete pending frames; no old source span
// survives the canonical publication or reappears after the source is replaced.
#[test]
fn source_reload() {
    let source = SOURCES[2].1;
    let mut e = Editor::new(case("markdown"), source, source.find("leaf").unwrap() + 2);
    e.settle(|_, _| {});
    let path = e
        .session
        .buffer()
        .unwrap()
        .file()
        .unwrap()
        .path()
        .to_owned();
    let replacement = source.replacen("leaf", "fresh", 1);
    std::fs::write(path, &replacement).unwrap();
    e.session.execute(Command::RevertBuffer);
    let end = Instant::now() + Duration::from_secs(10);
    while e.source() != replacement {
        assert!(Instant::now() < end, "owned source reload deadline");
        e.session.expire_input(super::now());
        let source = e.source();
        e.settle(|frame, cells| current(frame, cells, &source));
        std::thread::yield_now();
    }
    e.settle(|frame, cells| current(frame, cells, &replacement));
    assert_eq!(e.session.remaining_input(super::now()), None);
    assert!(!e.session.buffer().unwrap().is_modified());
}

// Isolate crossterm's cached NO_COLOR policy. Native complete updates preserve
// pretty headings and finite pair attributes through a neutral source change.
#[test]
fn no_color() {
    if std::env::var_os("REV_CONTINUITY_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "continuity::transitions::no_color",
                "--nocapture",
            ])
            .env("REV_CONTINUITY_COLOR_CHILD", "1")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let source = SOURCES[1].1;
    let mut c = case("org");
    c.style = "match";
    let mut e = Editor::new(c, source, source.find('(').unwrap());
    e.settle(|_, _| {});
    e.settle(|frame, cells| monochrome(frame, cells, source, true));
    let at = source.find("leaf").unwrap() + 2;
    e.session.replace(at..at, "x").unwrap();
    let expected = source.replacen("leaf", "lexaf", 1);
    e.settle(|frame, cells| monochrome(frame, cells, &expected, false));
    e.opening(&expected);
    e.settle(|frame, cells| monochrome(frame, cells, &expected, true));
    choose(&mut e, "off");
    let frame = e.settle(|_, _| {});
    assert!(crate::rendering::marks(&frame).is_empty());
    assert_eq!(e.session.remaining_input(super::now()), None);
}

fn monochrome(frame: &Frame, cells: &super::observe::Cells, source: &str, active: bool) {
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2") && !output.contains("48;2") && !output.contains("\x1b[5m"));
    assert!(frame.rows()[0].text().contains('◉'));
    let positions = crate::rendering::marks(frame);
    assert_eq!(positions.len(), if active { 2 } else { 0 });
    assert!(positions.iter().all(|(_, y, x)| cells[&(*y, *x)].underline));
    current(frame, cells, source);
}
