use rev::command::{History, Motion};
use rev::coordinates::Coordinates;
use rev::org::{Cancellation, ColorScan, Coverage, Limits};

use super::*;

fn current(session: &mut Session) {
    settle(session);
    let buffer = session.buffer().unwrap();
    let colors = buffer.org_highlights().expect("current Org observation");
    assert_eq!(colors.stamp(), buffer.stamp());
    assert_eq!(colors.coverage(), Coverage::Complete);
    let fresh = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    assert_eq!(
        buffer.org_captures(0..buffer.len_bytes()),
        fresh.highlights().captures(buffer, 0..buffer.len_bytes())
    );
    let coordinates = Coordinates::new(buffer).unwrap();
    for (_, view) in session.windows().iter() {
        assert_eq!(view.point.stamp, buffer.stamp());
        assert!(coordinates.is_boundary(view.point.position).unwrap());
    }
}

// A stale worker must be reaped before a new revision is observed. Shared panes
// receive current byte/cell styles after paste, history and delimiter recovery;
// coloring itself cannot change BOM/CRLF, file bytes, dirty state or history.
#[test]
fn shared_revisions() {
    let source =
        "* Heading 界e\u{301}\r\n#+begin_example\r\n,* Fake [[file:a]]\r\n#+end_example\r\n";
    let (fixture, mut session) = editor("", "notes.org", &format!("\u{feff}{source}"));
    let original = session.buffer().unwrap().stamp();
    session.frame().unwrap();
    assert!(session.remaining_input(Instant::now()).is_some());
    session.insert('*');
    assert!(
        session
            .buffer()
            .unwrap()
            .org_captures(0..source.len())
            .is_empty()
    );
    current(&mut session);
    assert_ne!(session.buffer().unwrap().stamp(), original);
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    session.execute(Command::Motion(Motion::BufferEnd));
    session.handle(Event::Paste("* Tail\r\n~code~".into()));
    current(&mut session);
    let edited = text(&session);
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .filter(|row| !row.syntax().is_empty())
            .count()
            >= 2
    );
    assert_eq!(session.remaining_input(Instant::now()), None);
    session.execute(Command::History(History::Undo));
    current(&mut session);
    session.execute(Command::History(History::Redo));
    current(&mut session);
    assert_eq!(text(&session), edited);
    let stamp = session.buffer().unwrap().stamp();
    let state = session.buffer().unwrap().state();
    let history = session.buffer().unwrap().history().len();
    for _ in 0..3 {
        session.frame().unwrap();
        assert_eq!(session.remaining_input(Instant::now()), None);
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().state(), state);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    session.execute(Command::SaveBuffer);
    assert!(
        !session.buffer().unwrap().is_modified(),
        "{}",
        session.echo()
    );
    assert_eq!(
        std::fs::read(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{edited}").as_bytes()
    );
}

// C-g consumes pending observation ownership before any publication. No redraw
// retries that revision; accepted editing starts fresh work and restores colors.
#[test]
fn canceled_revision() {
    let (_fixture, mut session) = editor("", "notes.org", "* Heading\nbody\n");
    session.frame().unwrap();
    assert!(session.remaining_input(Instant::now()).is_some());
    session.execute(Command::KeyboardQuit);
    settle(&mut session);
    assert!(session.buffer().unwrap().org_highlights().is_none());
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.remaining_input(Instant::now()), None);
    session.insert('*');
    current(&mut session);
    assert!(!session.buffer().unwrap().org_captures(0..10).is_empty());
}

// Hidden/disabled/tiny panes cancel only their owned work. Plain buffers and
// inactive Org revisions retain the native indefinite input wait.
#[test]
fn retired_work() {
    for name in ["text mode", "hidden", "tiny"] {
        let (fixture, mut session) = editor("", "notes.org", "* Heading\nbody\n");
        session.frame().unwrap();
        match name {
            "text mode" => {
                session.execute(Command::Mode(Mode::Text));
            }
            "hidden" => {
                session.load(&fixture.write("plain.txt", "plain")).unwrap();
            }
            _ => {
                session.handle(Event::Resize(0, 0));
            }
        }
        settle(&mut session);
        assert_eq!(session.remaining_input(Instant::now()), None, "{name}");
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .all(|row| row.syntax().is_empty()),
            "{name}"
        );
        assert!(
            session.buffer().unwrap().org_highlights().is_none(),
            "{name}"
        );
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Native Org admission honors the ordinary syntax toggle and exact UTF-8 cutoff,
// independently of a configured Rust grammar. Growing past equality drops old
// styles immediately; undo makes the current revision eligible for fresh work.
#[test]
fn syntax_policy() {
    let source = "* 界\n";
    for (name, config, colored) in [
        ("default", "", true),
        ("disabled", "syntax.enabled = false", false),
        ("equal", "syntax.max_bytes = 6", true),
        ("cutoff", "syntax.max_bytes = 5", false),
        ("unlimited", "syntax.max_bytes = 0", true),
        ("grammar", "language.org.grammar = 'rust'", true),
        ("unavailable", "org.enabled = false", false),
    ] {
        let (_fixture, mut session) = editor(config, "notes.org", source);
        let stamp = session.buffer().unwrap().stamp();
        settle(&mut session);
        assert_eq!(
            session.buffer().unwrap().org_highlights().is_some(),
            colored,
            "{name}"
        );
        assert_eq!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| !row.syntax().is_empty()),
            colored,
            "{name}"
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert_eq!(session.remaining_input(Instant::now()), None, "{name}");
        if name == "equal" {
            session.insert('界');
            settle(&mut session);
            assert!(
                session
                    .frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .all(|row| row.syntax().is_empty())
            );
            session.execute(Command::History(History::Undo));
            current(&mut session);
            assert_eq!(text(&session), source);
        }
    }
}
