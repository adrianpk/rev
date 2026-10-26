#![forbid(unsafe_code)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, TreeCommand};
use rev::display::Size;
use rev::session::{Action, Session};
use unicode_width::UnicodeWidthStr;

fn editor() -> Session {
    Session::empty(Size {
        width: 48,
        height: 8,
    })
    .unwrap()
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn tree(session: &mut Session, command: TreeCommand) {
    assert_eq!(session.execute(Command::Tree(command)), Action::Redraw);
}
fn named(session: &mut Session, name: &str) {
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// A narrow root projection and clear selection coexist with the unchanged
// editor. Tab transfers focus without hiding the tree or allocating a buffer.
#[test]
fn presentation() {
    let mut session = editor();
    let stamp = session.buffer().unwrap().stamp();
    assert!(!session.tree().visible());
    tree(&mut session, TreeCommand::Toggle);
    assert!(session.tree().focused());
    assert_eq!(
        session.tree().root(),
        Some(std::env::current_dir().unwrap().as_path())
    );
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].selection(), Some(0..16));
    assert_eq!(frame.cursor().unwrap().column, 0);
    assert_eq!(frame.rows()[0].text().chars().nth(16), Some('|'));
    assert!(frame.rows().iter().all(|row| row.text().width() == 48));
    session.handle(key(KeyCode::Tab));
    assert!(!session.tree().focused());
    assert!(session.tree().visible());
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 17);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), "");
    assert!(session.buffer().unwrap().history().is_empty());
}

// M-x and nested keys use the same handlers. Focus can return to the tree while
// it stays visible, and named hiding works from its own workspace context.
#[test]
fn command_routes() {
    for named_entry in [false, true] {
        let mut session = editor();
        if named_entry {
            named(&mut session, "toggle-file-tree");
        } else {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::CONTROL,
            )));
            session.handle(key(KeyCode::Char('t')));
            session.handle(key(KeyCode::Char('t')));
        }
        assert!(session.tree().focused());
        session.handle(key(KeyCode::Tab));
        named(&mut session, "focus-file-tree");
        assert!(session.tree().focused());
        named(&mut session, "toggle-file-tree");
        assert!(!session.tree().visible());
        assert!(!session.tree().focused());
        assert_eq!(text(&session), "");
    }
    assert_eq!(
        Command::named("toggle-file-tree"),
        Some(Command::Tree(TreeCommand::Toggle))
    );
    assert_eq!(
        Command::named("focus-file-tree"),
        Some(Command::Tree(TreeCommand::Focus))
    );
}

// Tree focus refuses document input and stays outside editing-leaf cycling.
// Dirty text, independent cursors and the original undo group survive toggling.
#[test]
fn editing_views() {
    use rev::command::{History, Split, WindowCommand};
    let mut session = editor();
    session.insert('界');
    session.execute(Command::Split(Split::Right));
    let first = session.windows().selected_id();
    let peer = session.windows().order()[1];
    tree(&mut session, TreeCommand::Toggle);
    session.insert('!');
    session.handle(Event::Paste("bad".into()));
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.windows().selected_id(), first);
    assert_eq!(text(&session), "界");
    assert!(session.select_view(peer).is_err());
    session.handle(key(KeyCode::Tab));
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.windows().selected_id(), peer);
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.windows().selected_id(), first);
    assert_eq!(session.windows().order().len(), 2);
    tree(&mut session, TreeCommand::Focus);
    session.execute(Command::KeyboardQuit);
    assert!(!session.tree().focused());
    tree(&mut session, TreeCommand::Toggle);
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
    assert!(!session.buffer().unwrap().is_modified());
}
