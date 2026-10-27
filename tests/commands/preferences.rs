use rev::buffer::Buffers;
use rev::config::{Snapshot, prepare};
use rev::input::{Error, Focus, Input, Outcome};
use rev::session::{Action, Session};
use rev::view::{Size, View};

use super::{Command, Event, KeyCode, character, control, key};

fn settings(config: &str) -> Snapshot {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{config}");
    startup.snapshot
}

// Input bounds count scalars rather than UTF-8 bytes or graphemes. A refused
// paste or key preserves the draft/cursor; deleting an accent frees two scalars.
#[test]
fn unicode_limit() {
    let mut input = Input::with_settings(&settings("minibuffer.input_limit = 3"));
    input.argument("Text: ", "e\u{301}界").unwrap();
    input.handle(key(KeyCode::Home));
    for event in [character('é'), Event::Paste("ab".into())] {
        assert_eq!(
            input.handle(event),
            Outcome::Message("Prompt input limit exceeded")
        );
        assert_eq!(input.prompt().unwrap().text(), "e\u{301}界");
        assert_eq!(input.prompt().unwrap().cursor(), 0);
    }
    input.handle(key(KeyCode::Delete));
    assert_eq!(input.handle(Event::Paste("é😀".into())), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text(), "é😀界");
    assert_eq!(input.prompt().unwrap().cursor(), "é😀".len());
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.focus(), Focus::Editing);
}

// Initial acquisition and external completion both enforce the supplied limit;
// rejected acquisition preserves prefixes and rejected replacement preserves focus.
#[test]
fn prompt_replacement() {
    let mut input = Input::with_settings(&settings("minibuffer.input_limit = 1"));
    input.handle(control('x'));
    assert_eq!(input.argument("Text: ", "界é"), Err(Error::Limit));
    assert_eq!(input.prefix(), Some(super::Prefix::ControlX));
    input.argument("Text: ", "界").unwrap();
    assert_eq!(input.complete("é😀"), Err(Error::Limit));
    assert_eq!(input.prompt().unwrap().text(), "界");
    assert_eq!(input.prompt().unwrap().cursor(), "界".len());
    assert_eq!(input.focus(), Focus::Argument);
    input.complete("é").unwrap();
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Argument("é".into())
    );
    assert_eq!(input.focus(), Focus::Editing);
}

fn accept(input: &mut Input, query: &str) -> Command {
    input.commands().unwrap();
    assert_eq!(input.handle(Event::Paste(query.into())), Outcome::Changed);
    let Outcome::Command(command) = input.handle(key(KeyCode::Enter)) else {
        panic!("valid fixed command query: {query}");
    };
    command
}

// History keeps only distinct accepted invocations, including command-handler
// refusals. Eviction, repeat promotion, cancellation and draft restoration agree.
#[test]
fn command_retention() {
    for limit in [1, 2] {
        let mut input =
            Input::with_settings(&settings(&format!("minibuffer.history_limit = {limit}")));
        for name in ["undo", "yank", "undo-redo", "yank"] {
            accept(&mut input, name);
        }
        input.commands().unwrap();
        input.handle(Event::Paste("draft".into()));
        input.handle(key(KeyCode::Up));
        assert_eq!(input.prompt().unwrap().text(), "yank");
        input.handle(key(KeyCode::Up));
        assert_eq!(
            input.prompt().unwrap().text(),
            if limit == 1 { "yank" } else { "undo-redo" }
        );
        input.handle(key(KeyCode::Up));
        assert_eq!(
            input.prompt().unwrap().text(),
            if limit == 1 { "yank" } else { "undo-redo" }
        );
        for _ in 0..limit {
            input.handle(key(KeyCode::Down));
        }
        assert_eq!(input.prompt().unwrap().text(), "draft");
        input.handle(control('g'));
        input.commands().unwrap();
        input.handle(key(KeyCode::Up));
        assert_eq!(input.prompt().unwrap().text(), "yank");
    }
}

// A short query may invoke a longer catalog name, but completion/recall cannot
// insert it past the text limit. Failure preserves the draft and navigation state.
#[test]
fn bounded_command_names() {
    let mut input = Input::with_settings(&settings("minibuffer.input_limit = 3"));
    assert_eq!(
        accept(&mut input, "und"),
        Command::History(rev::command::History::Undo)
    );
    input.commands().unwrap();
    input.handle(Event::Paste("ya".into()));
    for code in [KeyCode::Tab, KeyCode::Up, KeyCode::Up] {
        assert_eq!(
            input.handle(key(code)),
            Outcome::Message("Prompt input limit exceeded")
        );
        assert_eq!(input.prompt().unwrap().text(), "ya");
        assert_eq!(input.prompt().unwrap().cursor(), 2);
    }
    input.handle(key(KeyCode::Down));
    assert_eq!(input.prompt().unwrap().text(), "ya");
    input.handle(control('g'));
    assert_eq!(input.focus(), Focus::Editing);
}

fn editor() -> Session {
    let mut buffers = Buffers::with_settings(settings("minibuffer.input_limit = 3")).unwrap();
    let id = buffers.create("é界abc").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

// Real session prompt stages share the same scalar bound. Over-limit errors
// remain cancellable and cannot publish text or alter saved-state/history.
#[test]
fn session_cancellation() {
    for command in [
        Command::ExecuteExtended,
        Command::FindFile,
        Command::SearchForward,
        Command::QueryReplace,
    ] {
        let mut session = editor();
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        session.execute(command);
        assert_eq!(
            session.handle(Event::Paste("界é😀a".into())),
            Action::Redraw
        );
        assert_eq!(session.echo(), "Prompt input limit exceeded");
        session.handle(control('g'));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().state(), state);
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
        session.handle(character('z'));
        assert_eq!(
            session.buffer().unwrap().stamp().revision,
            stamp.revision + 1
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}

// Literal search paste admits separators but shares the scalar bound, including
// the tab insertion path. Refusal retains the exact multiline query until cancel.
#[test]
fn literal_limit() {
    let mut input = Input::with_settings(&settings("minibuffer.input_limit = 3"));
    input.search(rev::search::Direction::Forward).unwrap();
    assert_eq!(
        input.handle(Event::Paste("\n\t界".into())),
        Outcome::Changed
    );
    for event in [key(KeyCode::Tab), Event::Paste("é".into())] {
        assert_eq!(
            input.handle(event),
            Outcome::Message("Prompt input limit exceeded")
        );
        assert_eq!(input.prompt().unwrap().text(), "\n\t界");
    }
    input.handle(control('g'));
    assert_eq!(input.focus(), Focus::Editing);
}

// The second query-replace stage uses the same admitted input bound. A rejected
// replacement cannot mutate the source and cancellation clears the pending edit.
#[test]
fn replacement_limit() {
    let mut session = editor();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::QueryReplace);
    session.handle(Event::Paste("é".into()));
    session.handle(key(KeyCode::Enter));
    session.handle(Event::Paste("界é😀a".into()));
    assert_eq!(session.echo(), "Prompt input limit exceeded");
    session.handle(control('g'));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    session.handle(character('z'));
    assert_eq!(
        session.buffer().unwrap().stamp().revision,
        stamp.revision + 1
    );
}
