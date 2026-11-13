use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Mode;
use rev::command::{Command, History, Motion, Split};

use super::*;

// The real M-x and modal selectors share a mode action. Text suspension and
// explicit reentry preserve canonical text, points, saved state and history.
#[test]
fn native_modes() {
    for (profile, colon) in [("non-modal", false), ("modal", false), ("modal", true)] {
        let config = format!("editor.interaction_profile='{profile}'");
        let (_fixture, mut session) = editor(&config, "notes.txt", "*word*\n");
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point;
        let state = session.buffer().unwrap().state();
        for (name, mode) in [
            ("markdown-mode", "Markdown"),
            ("text-mode", "Text"),
            ("markdown-mode", "Markdown"),
        ] {
            selector(&mut session, name, colon);
            settle(&mut session);
            assert_eq!(session.buffer().unwrap().mode(), mode);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().state(), state);
            assert_eq!(session.view().point, point);
            assert_eq!(text(session.buffer().unwrap()), "*word*\n");
            assert!(!session.buffer().unwrap().is_modified());
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

fn selector(session: &mut Session, name: &str, colon: bool) {
    session.handle(Event::Key(if colon {
        KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE)
    } else {
        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT)
    }));
    session.handle(Event::Paste(name.into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Completed styles disappear immediately on publication. Undo, redo and an
// explicit disk reload must re-observe the current delimiters, never old spans.
#[test]
fn edit_undo_reload() {
    let (fixture, mut session) = editor("", "notes.md", "*word*\n");
    settle(&mut session);
    session.execute(Command::Motion(Motion::BufferStart));
    session.insert('`');
    session.execute(Command::Motion(Motion::BufferEnd));
    session.insert('`');
    settle(&mut session);
    assert_eq!(
        style_at(&session.frame().unwrap(), 0, "word"),
        Some(Style::Markdown(MarkdownRole::Code))
    );
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert_eq!(
        style_at(&session.frame().unwrap(), 0, "word"),
        Some(Style::Markdown(MarkdownRole::Emphasis))
    );
    session.execute(Command::History(History::Redo));
    settle(&mut session);
    assert_eq!(
        style_at(&session.frame().unwrap(), 0, "word"),
        Some(Style::Markdown(MarkdownRole::Code))
    );
    // Restore the saved source before exercising the unprompted clean reload.
    session.execute(Command::History(History::Undo));
    session.execute(Command::History(History::Undo));
    fixture.write("notes.md", "[word](https://example.com)\n");
    session.execute(Command::RevertBuffer);
    settle(&mut session);
    assert_eq!(
        text(session.buffer().unwrap()),
        "[word](https://example.com)\n"
    );
    assert_eq!(
        style_at(&session.frame().unwrap(), 0, "word"),
        Some(Style::Markdown(MarkdownRole::Link))
    );
}

// A result can finish before polling but still lacks publication authority
// after C-g, Text mode, a new source revision or replacement of the visible file.
#[test]
fn pending_retirement() {
    for action in ["cancel", "text", "edit", "hide"] {
        let (fixture, mut session) = editor("", "notes.md", "*word*\n");
        session.frame().unwrap();
        match action {
            "cancel" => {
                session.execute(Command::KeyboardQuit);
            }
            "text" => {
                session.execute(Command::Mode(Mode::Text));
            }
            "edit" => {
                session.insert('\\');
            }
            "hide" => {
                session.load(&fixture.write("plain.txt", "word\n")).unwrap();
            }
            _ => unreachable!(),
        }
        settle(&mut session);
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .all(|row| row.syntax().is_empty()),
            "{action}"
        );
        assert!(
            session.remaining_input(Instant::now()).is_none(),
            "{action}"
        );
    }
}

// Mode suspension releases shared evidence; both panes use the new generation
// on reentry even when its source stamp is identical to an old pending result.
#[test]
fn shared_mode_generation() {
    let (_fixture, mut session) = editor("", "notes.md", "*word*\n");
    session.frame().unwrap();
    session.execute(Command::Mode(Mode::Text));
    session.execute(Command::Mode(Mode::Markdown));
    session.execute(Command::Split(Split::Right));
    settle(&mut session);
    let buffer = session.buffer().unwrap();
    for (_, view) in session.windows().iter() {
        let frame = Frame::project(buffer, view, SIZE, "").unwrap();
        assert_eq!(
            style_at(&frame, 0, "word"),
            Some(Style::Markdown(MarkdownRole::Emphasis))
        );
    }
    assert!(buffer.history().is_empty());
}

// Native bounds reject the whole observation and release the worker deadline.
// A smaller configured cutoff takes precedence; equality remains admitted.
#[test]
fn observation_bounds() {
    for (name, config, source, colored) in [
        ("cutoff", "syntax.max_bytes=5", "*word*".into(), false),
        ("equality", "syntax.max_bytes=6", "*word*".into(), true),
        (
            "disabled colors",
            "syntax.enabled=false",
            "*word*".into(),
            false,
        ),
        ("source cap", "", "x".repeat(8 * 1024 * 1024 + 1), false),
        ("event cap", "", "*word* ".repeat(60_000), false),
        (
            "depth cap",
            "",
            format!("{}*word*", "> ".repeat(260)),
            false,
        ),
    ] {
        let (_fixture, mut session) = editor(config, "notes.md", &source);
        settle(&mut session);
        assert_eq!(session.buffer().unwrap().mode(), "Markdown", "{name}");
        let frame = session.frame().unwrap();
        assert_eq!(
            frame.rows().iter().any(|row| !row.syntax().is_empty()),
            colored,
            "{name}"
        );
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert!(session.remaining_input(Instant::now()).is_none(), "{name}");
    }
}

// A disabled explicit entry refuses before replacing an existing native mode;
// the rejection has no source/history effect and does not disable Org itself.
#[test]
fn disabled_entry() {
    let (_fixture, mut session) = editor("markdown.enabled=false", "notes.org", "* Heading\n");
    settle(&mut session);
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::Mode(Mode::Markdown));
    assert_eq!(session.buffer().unwrap().mode(), "Org");
    assert!(session.echo().contains("markdown.enabled"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}
