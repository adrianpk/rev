#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "support/surface.rs"]
mod surface;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, CompletionCommand as C, History, Motion, WindowCommand};
use rev::coordinates::Scalar;
use rev::display::Size;
use rev::session::Session;
use rev::view::View;

const SOURCE: &str = "fn config() {}\nfn cfg_value() {}\nfn main() { cfg| }\n";

fn source(text: &str) -> (fixture::Fixture, Session) {
    configured_source(text, "")
}

fn configured_source(text: &str, config: &str) -> (fixture::Fixture, Session) {
    named_source("source.rs", text, config)
}

fn named_source(name: &str, text: &str, config: &str) -> (fixture::Fixture, Session) {
    let fixture = fixture::Fixture::new();
    let offset = text.find('|').unwrap();
    let position = text[..offset].chars().count();
    let path = fixture.write(name, &text.replace('|', ""));
    let config = fixture.write("config.toml", config);
    let snapshot = rev::config::resolve(&rev::config::load(&config).unwrap()).unwrap();
    let mut buffers = Buffers::with_settings(snapshot).unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = Scalar(position);
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

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn key(session: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    session.handle(Event::Key(KeyEvent::new(code, modifiers)));
    if code == KeyCode::Tab && modifiers == KeyModifiers::ALT {
        settle(session);
    }
}

// Native M-TAB and named invocation preselect without editing. Both acceptance
// keys insert literal names once and restore the full prior source in one undo.
#[test]
fn manual_edit() {
    for accept in [KeyCode::Tab, KeyCode::Enter] {
        let (_fixture, mut session) = source(SOURCE);
        let original = text(&session);
        let point = session.view().point;
        key(&mut session, KeyCode::Tab, KeyModifiers::ALT);
        let completion = session.completion().unwrap();
        assert_eq!(completion.selected(), Some(0));
        assert_eq!(completion.candidates()[0].text(), "cfg_value");
        assert_eq!(text(&session), original);
        assert_eq!(session.view().point, point);
        assert!(session.buffer().unwrap().history().is_empty());
        key(&mut session, accept, KeyModifiers::NONE);
        assert_eq!(text(&session), original.replace("{ cfg }", "{ cfg_value }"));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert!(session.completion().is_none());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original);
        session.execute(Command::History(History::Redo));
        assert!(text(&session).contains("{ cfg_value }"));
    }
}

// Popup navigation never moves source point. Cancellation keys consume their
// own event, leave no transaction, and restore ordinary Enter outside the list.
#[test]
fn navigation_and_cancel() {
    for cancel in [KeyCode::Esc, KeyCode::Char('g')] {
        let (_fixture, mut session) = source(SOURCE);
        let original = text(&session);
        let point = session.view().point;
        complete(&mut session);
        key(&mut session, KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(session.completion().unwrap().selected(), Some(1));
        key(&mut session, KeyCode::Char('p'), KeyModifiers::CONTROL);
        assert_eq!(session.completion().unwrap().selected(), Some(0));
        assert_eq!(session.view().point, point);
        let modifiers = if cancel == KeyCode::Esc {
            KeyModifiers::NONE
        } else {
            KeyModifiers::CONTROL
        };
        key(&mut session, cancel, modifiers);
        assert!(session.completion().is_none());
        assert_eq!(text(&session), original);
        assert!(session.buffer().unwrap().history().is_empty());
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(text(&session), original.replacen("{ cfg }", "{ cfg\n }", 1));
    }
}

// Unicode and raw identifiers replace the whole admitted token, including its
// suffix, while literal label annotations never enter the canonical document.
#[test]
fn literal_ranges() {
    for (name, document, expected) in [
        (
            "unicode",
            "fn café_value() {}\nfn main() { café|_old }",
            "{ café_value }",
        ),
        ("raw", "fn r#type() {}\nfn main() { r#ty| }", "{ r#type }"),
        ("subsequence", SOURCE, "{ config }"),
    ] {
        let (_fixture, mut session) = source(document);
        complete(&mut session);
        if name == "subsequence" {
            session.execute(Command::Completion(C::Next));
        }
        session.execute(Command::Completion(C::Accept));
        assert!(
            text(&session).contains(expected),
            "{name}: {}",
            text(&session)
        );
        assert!(!text(&session).contains("  function"));
    }
}

// Edits, cursor movement, buffer switches and pane selection retire old targets.
// A later explicit accept cannot turn their cached scalar range into an edit.
#[test]
fn stale_targets() {
    for change in ["edit", "point", "view", "buffer", "paste"] {
        let (fixture, mut session) = source(SOURCE);
        complete(&mut session);
        match change {
            "edit" => {
                session.replace(0..0, "// changed\n").unwrap();
            }
            "point" => {
                session.execute(Command::Motion(Motion::BackwardChar));
            }
            "view" => {
                session.execute(Command::Split(rev::window::Split::Right));
                session.execute(Command::Window(WindowCommand::Other));
            }
            "buffer" => {
                session
                    .load(&fixture.write("other.rs", "fn other() {}"))
                    .unwrap();
            }
            _ => {
                session.handle(Event::Paste("x".into()));
            }
        }
        let before = text(&session);
        let history = session.buffer().unwrap().history().len();
        session.execute(Command::Completion(C::Accept));
        assert_eq!(text(&session), before, "{change}");
        assert_eq!(
            session.buffer().unwrap().history().len(),
            history,
            "{change}"
        );
        assert!(session.completion().is_none(), "{change}");
    }
}

// No usable declarations produce no popup and retain normal Tab dispatch.
// Manual requests still work in comments; syntax extraction only offers names
// of declarations, never identifiers that appeared only in an expression.
#[test]
fn admitted_sources() {
    for (document, expected) in [
        ("fn sample() {}\n// sa|", Some("sample")),
        ("fn sample() {}\nfn main() { use_only(); use| }", None),
        ("fn sample() {}\nfn main() { no_match| }", None),
    ] {
        let (_fixture, mut session) = source(document);
        complete(&mut session);
        assert_eq!(
            session.completion().map(|c| c.candidates()[0].text()),
            expected
        );
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// The popup is an overlay: it preserves source cursor/scroll, uses the selection
// palette across the candidate row and fits after split/resize, including above
// a bottom-row cursor. Tiny panes keep cancellation and source editing usable.
#[test]
fn cursor_popup() {
    let (_fixture, mut session) = source(SOURCE);
    let original = session.frame().unwrap();
    let cursor = original.cursor();
    complete(&mut session);
    let frame = session.frame().unwrap();
    assert_eq!(frame.cursor(), cursor);
    let selected = frame
        .rows()
        .iter()
        .find(|row| row.selection().is_some())
        .unwrap();
    assert!(selected.text().contains("cfg_value"));
    let cells = surface::draw(&frame);
    let range = selected.selection().unwrap();
    let palette = rev::config::Palette::default();
    for column in range {
        assert_eq!(
            cells[&(usize::from(selected.row()), usize::from(column))].background,
            Some(palette.selection)
        );
    }
    for (width, height) in [(40, 6), (12, 4), (2, 2), (80, 24)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(
                unicode_width::UnicodeWidthStr::width(row.text()),
                usize::from(width)
            );
        }
    }
    assert_eq!(text(&session), SOURCE.replace('|', ""));
    assert!(session.buffer().unwrap().history().is_empty());
}

// Dismissing a popup must not consume the first key of an ordinary command
// prefix. Accepting with a held Return must not leak its repeats into source.
#[test]
fn input_continuity() {
    let (_fixture, mut session) = source(SOURCE);
    complete(&mut session);
    key(&mut session, KeyCode::Char('x'), KeyModifiers::CONTROL);
    key(&mut session, KeyCode::Char('3'), KeyModifiers::NONE);
    assert_eq!(session.windows().iter().len(), 2);
    complete(&mut session);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    let accepted = text(&session);
    session.handle(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Repeat,
    )));
    assert_eq!(text(&session), accepted);
}

// A bottom-row list uses space above the cursor and remains inside the selected
// pane. The neighbouring split keeps all of its original source cells.
#[test]
fn pane_placement() {
    let (_fixture, mut session) = source("fn config() {}\nfn cfg_value() {}\n\n\n\n// cfg|");
    session.execute(Command::Split(rev::window::Split::Right));
    session.handle(Event::Resize(80, 8));
    let before = session.frame().unwrap();
    let cursor = before.cursor().unwrap();
    complete(&mut session);
    let after = session.frame().unwrap();
    let top = after
        .rows()
        .iter()
        .find(|row| row.text().contains('┌'))
        .unwrap();
    assert!(top.row() < cursor.row);
    assert_eq!(after.cursor(), Some(cursor));
    let boundary = session
        .windows()
        .area(
            session.windows().selected_id(),
            Size {
                width: 80,
                height: 7,
            },
        )
        .unwrap();
    for (old, new) in before.rows().iter().zip(after.rows()) {
        assert_eq!(
            old.text()
                .chars()
                .skip(usize::from(boundary.width()) + 1)
                .collect::<String>(),
            new.text()
                .chars()
                .skip(usize::from(boundary.width()) + 1)
                .collect::<String>()
        );
    }
}

fn settle(session: &mut Session) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let now = std::time::Instant::now();
        session.expire_input(now);
        if session.remaining_input(now).is_none() {
            break;
        }
        assert!(
            now < deadline,
            "native completion did not settle: {}",
            session.echo()
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
fn complete(session: &mut Session) {
    session.execute(Command::Completion(C::AtPoint));
    settle(session);
}

#[path = "completion/automatic.rs"]
mod automatic;

#[path = "completion/systems.rs"]
mod systems;
