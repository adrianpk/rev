use crossterm::event::Event;
use rev::buffer::Buffers;
use rev::command::{Command, History};
use rev::config::prepare;
use rev::coordinates::Scalar;
use rev::session::{Action, Session};
use rev::view::{Size, View};

use crate::config_fixture::Fixture;

fn editor(configuration: &str, source: &str, position: usize, readonly: bool) -> Session {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", configuration);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(readonly);
    let mut view = View::new(buffer);
    view.point.position = Scalar(position);
    Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap()
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Space indentation advances to the next indent stop using display columns;
// existing tabs and whole Unicode clusters contribute their real visual width.
// Tab indentation inserts exactly one literal tab independently of both widths.
#[test]
fn indent_stops() {
    let cases = [
        ("empty", "", "", 0, "    "),
        ("nonaligned", "", "a", 1, "a   "),
        ("aligned", "", "a   ", 4, "a       "),
        ("wide", "", "界", 1, "界  "),
        ("combining", "", "e\u{301}", 2, "e\u{301}   "),
        ("ZWJ", "", "👩\u{200d}💻", 3, "👩\u{200d}💻  "),
        (
            "existing tab",
            "editor.indent_width = 3\neditor.tab_width = 8",
            "a\t",
            2,
            "a\t ",
        ),
        (
            "separate widths",
            "editor.indent_width = 2\neditor.tab_width = 8",
            "\t",
            1,
            "\t  ",
        ),
        (
            "literal tab",
            "editor.indent_style = 'tabs'\neditor.indent_width = 2\neditor.tab_width = 8",
            "界",
            1,
            "界\t",
        ),
    ];
    for (name, configuration, source, position, expected) in cases {
        let mut session = editor(configuration, source, position, false);
        let stamp = session.buffer().unwrap().stamp();
        assert_eq!(session.execute(Command::InsertTab), Action::Redraw);
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(
            session.buffer().unwrap().stamp().revision,
            stamp.revision + 1
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected, "{name}");
    }
}

// Bracketed paste bypasses indentation policy: tabs, separators and key-looking
// bytes remain one literal replacement and one reversible history operation.
#[test]
fn literal_paste() {
    let payload = "\t界\r\ne\u{301}\n\x18\x03";
    for style in ["spaces", "tabs"] {
        let configuration = format!(
            "editor.indent_style = '{style}'\neditor.indent_width = 3\neditor.tab_width = 8"
        );
        let mut session = editor(&configuration, "<>", 1, false);
        assert_eq!(session.handle(Event::Paste(payload.into())), Action::Redraw);
        assert_eq!(text(&session), format!("<{payload}>"));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "<>");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), format!("<{payload}>"));
    }
}

// Bound Tab and real M-x dispatch share the configured handler and refusal
// boundary. Read-only text retains its point, revision, state and history.
#[test]
fn dispatch_admission() {
    let cases = [
        ("editor.indent_width = 3\neditor.tab_width = 8", "a\t界  "),
        (
            "editor.indent_style = 'tabs'\neditor.tab_width = 2",
            "a\t界\t",
        ),
    ];
    for (configuration, expected) in cases {
        for (readonly, named) in [(false, false), (false, true), (true, false), (true, true)] {
            dispatch_case(configuration, readonly, named, expected);
        }
    }
}

fn dispatch_case(configuration: &str, readonly: bool, named: bool, expected: &str) {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let source = "a\t界";
    let mut session = editor(configuration, source, 3, readonly);
    let before = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
        session.view().point,
    );
    if named {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::ALT,
        )));
        session.handle(Event::Paste("insert-tab".into()));
        session.handle(super::key(KeyCode::Enter));
    } else {
        session.handle(super::key(KeyCode::Tab));
    }
    if readonly {
        assert_eq!(text(&session), source);
        assert_eq!(
            (
                session.buffer().unwrap().stamp(),
                session.buffer().unwrap().state(),
                session.view().point
            ),
            before
        );
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(session.echo().contains("read-only"));
    } else {
        assert_eq!(text(&session), expected);
        assert_eq!(
            session.view().point.position,
            Scalar(expected.chars().count())
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert!(session.echo().is_empty());
    }
}
