use crossterm::event::{Event, KeyCode};
use rev::buffer::Buffers;
use rev::command::Command;
use rev::config::prepare;
use rev::display::{Band, Frame};
use rev::input::Input;
use rev::session::Session;
use rev::view::{Size, View};

fn configured(config: &str, size: Size) -> Session {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(buffers, view, size).unwrap()
}

fn names(frame: &Frame) -> Vec<&str> {
    frame
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim())
        .collect()
}

fn query(session: &mut Session, text: &str) {
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste(text.into()));
}

fn size() -> Size {
    Size {
        width: 80,
        height: 14,
    }
}

// The configured row limit bounds one-name-per-row output. The query precedes
// that list, owns its cursor and never becomes document text or an edit group.
#[test]
fn vertical_rows() {
    let mut session = configured("minibuffer.candidate_limit = 3", size());
    let stamp = session.buffer().unwrap().stamp();
    query(&mut session, "kill");
    let frame = session.frame().unwrap();
    assert_eq!(
        names(&frame),
        ["kill-buffer", "kill-line", "kill-rectangle"]
    );
    let prompt = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Echo)
        .unwrap();
    let first = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion)
        .unwrap();
    assert_eq!(first.row(), prompt.row() + 1);
    assert!(prompt.text().starts_with("M-x kill"));
    assert_eq!(frame.cursor().unwrap().row, prompt.row());
    assert_eq!(first.selection(), Some(0..80));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Matching modes retain the audited lexical catalog order within ranks. Prefix
// mode excludes substrings; substring mode has one rank instead of prefix bias.
#[test]
fn configured_matching() {
    for (mode, count, first) in [
        ("prefix-then-substring", 9, "kill-buffer"),
        ("prefix", 6, "kill-buffer"),
        ("substring", 9, "backward-kill-word"),
    ] {
        let fixture = super::config_fixture::Fixture::new();
        let startup =
            prepare(&fixture.write("config.toml", &format!("minibuffer.match = '{mode}'")));
        let mut input = Input::with_settings(&startup.snapshot);
        input.commands().unwrap();
        input.handle(Event::Paste("kill".into()));
        let candidates = input.palette().unwrap().candidates();
        assert_eq!(candidates.len(), count, "{mode}");
        assert_eq!(candidates[0].name(), first, "{mode}");
        input.handle(super::key(KeyCode::Tab));
        assert_eq!(input.prompt().unwrap().text(), first, "{mode}");
    }
}

// A no-match result has a dedicated unselected row. Rendering and resize cannot
// invoke a command or alter the draft; cancellation restores the editing owner.
#[test]
fn empty_result() {
    let mut session = configured("minibuffer.match = 'prefix'", size());
    let point = session.view().point;
    query(&mut session, "界e\u{301}");
    let frame = session.frame().unwrap();
    assert_eq!(names(&frame), ["No matching commands"]);
    assert!(
        frame
            .rows()
            .iter()
            .filter(|row| row.band() == Band::Completion)
            .all(|row| row.selection().is_none())
    );
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x 界e\u{301}"))
    );
    assert_eq!(session.view().point, point);
    session.execute(Command::KeyboardQuit);
    assert!(names(&session.frame().unwrap()).is_empty());
    assert_eq!(session.view().point, point);
}
