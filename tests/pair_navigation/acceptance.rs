use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, ModalCommand, Split};
use rev::coordinates::Scalar;
use rev::display::Frame;
use rev::session::Session;
use rev::view::{Size, View};

use super::groups::{keys, marked};
use crate::editor::{Editor, settle};
use crate::fixture::Fixture;
use crate::surface::{self, Cell};

const SIZE: Size = Size {
    width: 40,
    height: 22,
};

// Enhanced decoded repeat/release metadata retains the canonical expression
// chord in both profiles. Release cannot repeat an ordinary source motion.
#[test]
fn decoded_keys() {
    for profile in ["non-modal", "modal"] {
        let config = format!("editor.interaction_profile='{profile}'");
        let mut e = Editor::build(&config, "plain", "|one two three", true);
        if profile == "modal" {
            e.session.execute(Command::Modal(ModalCommand::Insert));
        }
        keys(&mut e, "C-M-f");
        for (kind, expected) in [(KeyEventKind::Repeat, 7), (KeyEventKind::Release, 7)] {
            let mut event = KeyEvent::new_with_kind(
                KeyCode::Char('f'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
                kind,
            );
            event.state = KeyEventState::CAPS_LOCK;
            e.session.handle(Event::Key(event));
            settle(&mut e.session);
            e.expect(&marked("one two three", expected));
        }
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Pending navigation keeps completed source heading cells exact in both views.
// Synchronous source projection exposes a literal-heading fallback instead of
// hiding it behind a loading frame or a replacement source edit.
#[test]
fn frame_continuity() {
    for (suffix, marker) in [("org", "**"), ("md", "##")] {
        let source = format!("{}\n{marker} Title\n(a)\n", "x".repeat(8192));
        let (_fixture, mut session) = editor(suffix, &source);
        settle(&mut session);
        session.execute(Command::Split(Split::Right));
        settle(&mut session);
        exact_heading(&mut session, marker);
        let baseline = samples(&session);
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point.position;
        let started = Instant::now();
        session.execute(Command::ForwardPair);
        assert_eq!(session.view().point.position, point);
        let (turns, maximum) = drain(&mut session, &baseline);
        assert!(turns > 0);
        assert_eq!(session.view().point.position, Scalar(point.0 + 3));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(session.buffer().unwrap().is_readonly());
        assert_eq!(samples(&session), baseline);
        println!(
            "NAVIGATION_FRAMES mode={suffix} source_bytes={} views=2 turns={turns} elapsed_us={} max_host_turn_us={}",
            source.len(),
            started.elapsed().as_micros(),
            maximum.as_micros()
        );
    }
}

fn editor(suffix: &str, source: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let config = "editor.soft_wrap=false\norg.indent=false\norg.headings.pretty=true\nmarkdown.pretty=true\npairs.enabled=false\ngutter.visible=false";
    let startup = rev::config::prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers
        .visit(&fixture.write(&format!("source.{suffix}"), source))
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let mut view = View::with_settings(buffer, &startup.snapshot);
    view.point.position = Scalar(source.find("(a)").unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    (fixture, session)
}

fn exact_heading(session: &mut Session, marker: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(Instant::now() < deadline, "exact heading deadline");
        let frame = session.frame().unwrap();
        let heading = frame
            .rows()
            .iter()
            .find(|row| row.text().contains("Title"))
            .unwrap();
        if !heading.text().contains(marker) {
            return;
        }
        session.expire_input(Instant::now());
        std::thread::yield_now();
    }
}

fn samples(session: &Session) -> Vec<Vec<Cell>> {
    let buffer = session.buffer().unwrap();
    session
        .windows()
        .iter()
        .map(|(_, view)| {
            let frame = Frame::project(buffer, view, SIZE, "").unwrap();
            let row = frame
                .rows()
                .iter()
                .position(|r| r.text().contains("Title"))
                .unwrap();
            let cells = surface::draw(&frame);
            cells
                .into_iter()
                .filter_map(|((y, _), cell)| (y == row).then_some(cell))
                .collect()
        })
        .collect()
}

fn drain(session: &mut Session, baseline: &[Vec<Cell>]) -> (usize, Duration) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut turns = 0;
    let mut maximum = Duration::ZERO;
    while session.remaining_input(Instant::now()) == Some(Duration::ZERO) {
        assert!(Instant::now() < deadline, "navigation frame deadline");
        let start = Instant::now();
        assert_eq!(samples(session), baseline);
        session.expire_input(start);
        maximum = maximum.max(start.elapsed());
        turns += 1;
    }
    (turns, maximum)
}
