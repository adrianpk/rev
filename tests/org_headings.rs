#![forbid(unsafe_code)]

#[path = "org_headings/colors.rs"]
mod colors;
#[path = "org_headings/controls.rs"]
mod controls;
#[path = "org_headings/editing.rs"]
mod editing;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "org_headings/geometry.rs"]
mod geometry;
#[path = "org_headings/lifecycle.rs"]
mod lifecycle;
#[path = "org_headings/observations.rs"]
mod observations;
#[path = "support/surface.rs"]
mod surface;

use std::time::{Duration, Instant};

use rev::buffer::{Buffer, Buffers};
use rev::command::{Command, History};
use rev::config::prepare;
use rev::display::{Frame, Size};
use rev::org::{Cancellation, ColorScan, Coverage, Limits};
use rev::session::Session;
use rev::view::View;

use fixture::Fixture;

const SIZE: Size = Size {
    width: 80,
    height: 24,
};

fn setup(config: &str, source: &str) -> (Fixture, Buffers, View) {
    let fixture = Fixture::new();
    // Exercise heading presentation without additional structural indentation.
    fixture.write("config.toml", config);
    let startup =
        prepare(&fixture.write("geometry.toml", "include=['config.toml']\norg.indent=false"));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&fixture.write("notes.org", source)).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    (fixture, buffers, view)
}

fn editor(config: &str, source: &str) -> (Fixture, Session) {
    let (fixture, buffers, view) = setup(config, source);
    (fixture, Session::new(buffers, view, SIZE).unwrap())
}

fn observe(buffer: &mut Buffer, limits: Limits) -> Coverage {
    let coloring = ColorScan::new(buffer, limits)
        .unwrap()
        .run(&Cancellation::default());
    let coverage = coloring.highlights().coverage();
    buffer.install_org_coloring(coloring).unwrap();
    coverage
}

fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "native heading observation did not settle"
        );
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn body(frame: &Frame, row: usize) -> String {
    frame.rows()[row].text().chars().skip(7).collect()
}

// Real session workers observe all eight levels under both profiles, including
// disabled coloring. Presentation leaves BOM/CRLF bytes, state and history intact.
#[test]
fn reference_levels() {
    let source: String = (1..=8)
        .map(|level| format!("{} Heading 界e\u{301}\r\n", "*".repeat(level)))
        .collect();
    for profile in ["non-modal", "modal"] {
        for colored in [true, false] {
            let config =
                format!("editor.interaction_profile='{profile}'\nsyntax.enabled={colored}");
            let (fixture, mut session) = editor(&config, &format!("\u{feff}{source}"));
            let stamp = session.buffer().unwrap().stamp();
            let state = session.buffer().unwrap().state();
            settle(&mut session);
            let frame = session.frame().unwrap();
            for (row, marker) in ["◉", "○", "✸", "✿", "◆", "▶", "◉", "○"].iter().enumerate()
            {
                let expected = format!("{marker} Heading 界e\u{301}");
                assert!(
                    body(&frame, row).starts_with(&expected),
                    "{profile}, colored={colored}, row={row}: {:?}",
                    body(&frame, row)
                );
                assert_eq!(frame.rows()[row].syntax().is_empty(), !colored);
            }
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().state(), state);
            assert_eq!(text(session.buffer().unwrap()), source);
            assert!(!session.buffer().unwrap().is_modified());
            assert!(session.buffer().unwrap().history().is_empty());
            session.execute(Command::SaveBuffer);
            assert_eq!(
                std::fs::read(fixture.0.join("notes.org")).unwrap(),
                format!("\u{feff}{source}").as_bytes()
            );
            assert!(session.buffer().unwrap().history().is_empty());
            assert_eq!(session.remaining_input(Instant::now()), None);
        }
    }
}

// The startup choices compose independently; custom graphemes and duplicate
// markers preserve geometry, and a disabled Org owner remains plain text.
#[test]
fn startup_choices() {
    for (name, config, expected) in [
        ("cycle", "", "○ Tail"),
        ("final", "org.headings.cycle_bullets=false", "▶ Tail"),
        (
            "visible",
            "org.headings.hide_leading_stars=false",
            "*******○ Tail",
        ),
        ("one marker", "org.headings.bullets=['+']", "+ Tail"),
        (
            "combining",
            "org.headings.bullets=['e\u{301}']",
            "e\u{301} Tail",
        ),
        ("source", "org.headings.pretty=false", "******** Tail"),
        (
            "source visible",
            "org.headings.pretty=false\norg.headings.hide_leading_stars=false",
            "******** Tail",
        ),
        ("disabled", "org.enabled=false", "******** Tail"),
        ("color cutoff", "syntax.max_bytes=1", "○ Tail"),
    ] {
        let (_fixture, mut session) = editor(config, "******** Tail\nbody");
        settle(&mut session);
        let frame = session.frame().unwrap();
        assert!(
            body(&frame, 0).starts_with(expected),
            "{name}: {:?}",
            body(&frame, 0)
        );
        assert!(body(&frame, 1).starts_with("body"), "{name}");
        assert_eq!(text(session.buffer().unwrap()), "******** Tail\nbody");
        if name == "disabled" {
            assert!(session.buffer().unwrap().org_highlights().is_none());
        }
    }
}

// A source edit retires the old prefix evidence immediately. Fresh observation
// and undo recompute from canonical stars without adding presentation history.
#[test]
fn edited_level() {
    let (_fixture, mut session) = editor("syntax.enabled=false", "** Heading\n");
    settle(&mut session);
    assert!(body(&session.frame().unwrap(), 0).starts_with("○ Heading"));
    session.insert('*');
    let stale = Frame::project(session.buffer().unwrap(), session.view(), SIZE, "").unwrap();
    assert!(body(&stale, 0).starts_with("*** Heading"));
    settle(&mut session);
    assert!(body(&session.frame().unwrap(), 0).starts_with("✸ Heading"));
    assert_eq!(session.view().point.position.0, 1);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert!(body(&session.frame().unwrap(), 0).starts_with("○ Heading"));
    assert_eq!(text(session.buffer().unwrap()), "** Heading\n");
    assert!(!session.buffer().unwrap().is_modified());
}
