use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{BufferCommand, Command, Motion, Scroll};
use rev::session::{Action, Session};
use rev::view::Size;

fn editor() -> Session {
    let mut session = Session::empty(Size {
        width: 100,
        height: 10,
    })
    .unwrap();
    session.replace(0..0, "0123456789\nx\nend").unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..8 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Scroll(Scroll::NextLine));
    session.execute(Command::Scroll(Scroll::Recenter));
    assert_eq!(session.view().column, Some(8));
    session
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn echo_row(session: &mut Session) -> String {
    session
        .frame()
        .unwrap()
        .rows()
        .last()
        .unwrap()
        .text()
        .to_owned()
}

fn payload(session: &mut Session, text: &str) {
    assert_eq!(session.handle(Event::Paste(text.into())), Action::Redraw);
}

// C-g from every focused entry preserves the selected view's intended column
// and recenter state, rather than treating transient command text as navigation.
#[test]
fn canceled_navigation() {
    let cases = [
        Command::ExecuteExtended,
        Command::FindFile,
        Command::WriteFile,
        Command::Buffer(BufferCommand::Switch),
        Command::Buffer(BufferCommand::Kill),
        Command::Buffer(BufferCommand::List),
    ];
    for command in cases {
        let mut session = editor();
        let before = session.view().clone();
        session.execute(command);
        session.handle(super::control('g'));
        assert_eq!(*session.view(), before, "{}", command.name());
        assert_eq!(text(&session), "0123456789\nx\nend");
        session.handle(super::character('!'));
        assert_eq!(text(&session), "0123456789\nx!\nend");
    }
}

// Tab in a path argument keeps its exact draft and focus, presenting the path
// interaction policy instead of an internal buffer-completion state error.
#[test]
fn path_tab() {
    for command in [Command::FindFile, Command::WriteFile, Command::SaveBuffer] {
        let mut session = editor();
        let before = session.view().clone();
        session.execute(command);
        payload(&mut session, "draft-é.txt");
        session.handle(super::key(KeyCode::Tab));
        assert_eq!(
            session.echo(),
            "Enter a path; C-g cancels",
            "{}",
            command.name()
        );
        assert!(echo_row(&mut session).contains("draft-é.txt"));
        session.handle(super::control('g'));
        assert_eq!(*session.view(), before);
        assert_eq!(text(&session), "0123456789\nx\nend");
    }
}

// A successful prompt edit retires feedback for the previous invalid key while
// retaining the same argument owner and never inserting into the document.
#[test]
fn argument_feedback() {
    let mut session = editor();
    session.execute(Command::FindFile);
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('q'),
        KeyModifiers::ALT,
    )));
    assert_eq!(session.echo(), "Unbound prompt key");
    session.handle(super::character('é'));
    assert_eq!(session.echo(), "");
    assert!(echo_row(&mut session).starts_with("Find file: é"));
    assert_eq!(text(&session), "0123456789\nx\nend");
    session.handle(super::control('g'));
    assert_eq!(session.echo(), "Quit");
}
