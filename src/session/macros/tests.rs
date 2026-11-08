use super::*;
use crate::buffer::Buffers;
use crate::command::Motion;
use crate::display::Size;
use crate::input::Input;
use crate::session::Pending;
use crate::view::View;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn session() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("abcdef").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 16,
        },
    )
    .unwrap()
}

fn record(session: &mut Session, named: bool, command: &str) {
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    if named {
        session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        session.handle(Event::Paste(command.into()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    } else {
        session.handle(key(KeyCode::Char('f'), KeyModifiers::CONTROL));
    }
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn replay(session: &mut Session) {
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    while session
        .remaining_input(Instant::now())
        .is_some_and(|wait| wait.is_zero())
    {
        assert!(Instant::now() < deadline);
        session.expire_input(Instant::now());
    }
}

// Replace only the admitted input map after capture. Saved names remain stable;
// ordinary keys resolve their current map, including unsupported-job admission.
#[test]
fn current_bindings() {
    for (named, target, expected, stop) in [
        (true, "backward-char", "aXbXcdef", None),
        (false, "backward-char", "aXXbcdef", None),
        (
            false,
            "shell-command",
            "aXbcdef",
            Some(StopReason::Unsupported),
        ),
    ] {
        let mut session = session();
        record(&mut session, named, "forward-char");
        let fixture = crate::fixture::Fixture::new();
        let config = fixture.write(
            "config.toml",
            &format!("[keys.editor]\n'C-f' = '{target}'\n'M-x' = 'delete-char'\n"),
        );
        let prepared = crate::config::prepare(&config);
        assert!(prepared.diagnostic.is_none());
        session.input = Input::with_settings(&prepared.snapshot);
        session.sync_input_context();
        replay(&mut session);
        assert_eq!(text(&session), expected);
        assert_eq!(session.macro_stop().map(|stop| stop.reason), stop);
        assert_eq!(session.shell_remaining(), None);
        assert!(!matches!(session.pending, Some(Pending::Shell(_))));
    }
}

// Built-ins cannot disappear in a live session. Mutate only the stored name of
// a real recording to exercise the unknown-name admission boundary directly.
#[test]
fn missing_name() {
    let mut session = session();
    record(&mut session, true, "forward-char");
    let definition = Arc::get_mut(session.macros.last.as_mut().unwrap()).unwrap();
    let crate::macros::Step::Command(command) = &mut definition.steps[0] else {
        panic!("named recording")
    };
    command.name = "missing-native-command".into();
    let before = text(&session);
    replay(&mut session);
    assert_eq!(text(&session), before);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::MissingCommand
    );
    assert_eq!(session.input.focus(), Focus::Editing);
    assert!(session.pending.is_none());
}

// A changed key may open another valid prompt, but its accepted argument role
// cannot be reassigned to that prompt or leak into source editing.
#[test]
fn prompt_role() {
    let mut session = session();
    let fixture = crate::fixture::Fixture::new();
    let config = fixture.write("config.toml", "[keys.editor]\n'C-f' = 'goto-line'\n");
    session.input = Input::with_settings(&crate::config::prepare(&config).snapshot);
    session.sync_input_context();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('f'), KeyModifiers::CONTROL));
    session.handle(Event::Paste("1".into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    let config = fixture.write("config.toml", "[keys.editor]\n'C-f' = 'switch-to-buffer'\n");
    session.input = Input::with_settings(&crate::config::prepare(&config).snapshot);
    session.sync_input_context();
    session.execute(Command::Motion(Motion::BufferStart));
    let before = text(&session);
    replay(&mut session);
    assert_eq!(text(&session), before);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::PromptMismatch
    );
    assert_eq!(session.input.focus(), Focus::Editing);
}

// Choose an ambiguous selector row, then change ranking until that row would
// select another command. Replay still runs the resolved recorded identity.
#[test]
fn changed_ranking() {
    let mut session = session();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste("char".into()));
    let candidates = session.input.palette().unwrap().candidates();
    let index = candidates
        .iter()
        .position(|command| command.name() == "forward-char")
        .unwrap();
    for _ in 0..index {
        session.handle(key(KeyCode::Down, KeyModifiers::NONE));
    }
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    for _ in 0..4 {
        session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        session.handle(Event::Paste("backward-char".into()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    }
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste("char".into()));
    assert_ne!(
        session.input.palette().unwrap().candidates()[index].name(),
        "forward-char"
    );
    session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    session.execute(Command::Motion(Motion::BufferStart));
    replay(&mut session);
    assert_eq!(text(&session), "aXXbcdef");
    assert_eq!(session.macro_stop(), None);
}

// Native search continuation has an ordinary no-match message outside macros.
// During capture its typed failure must prevent publishing a false success.
#[test]
fn search_continuation() {
    let mut session = session();
    let command = Command::Navigate(crate::command::Navigation::new(
        crate::command::navigation::Target::Search(crate::command::navigation::Search::Repeat),
    ));
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste(command.name().into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(text(&session), "abcdef");
    assert!(session.macros.last.is_none());
}
