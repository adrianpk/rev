#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod config_fixture;
#[path = "tree/hits.rs"]
mod hits;
#[path = "tree/initialization.rs"]
mod initialization;
#[path = "tree/preferences.rs"]
mod preferences;

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
    assert_eq!(frame.rows()[0].text().chars().nth(16), Some('│'));
    assert!(frame.rows().iter().all(|row| row.text().width() == 48));
    session.handle(key(KeyCode::Tab));
    assert!(!session.tree().focused());
    assert!(session.tree().visible());
    // Tree (16), shared separator (1), factory number field (6), source column 0.
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 23);
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

// Unusable geometry retains tree intent until editing resumes. Restoration
// stays within native extents and cannot steal focus after an explicit edit.
#[test]
fn geometry_lifecycle() {
    let mut session = editor();
    tree(&mut session, TreeCommand::Toggle);
    let root = session.tree().root().unwrap().to_path_buf();
    let dimensions = [
        (31, 8, 0),
        (32, 8, 10),
        (71, 8, 23),
        (72, 8, 24),
        (80, 2, 0),
        (80, 3, 24),
        (0, 0, 0),
        (1, 1, 0),
        (65535, 1, 0),
        (1, 65535, 0),
        (48, 8, 16),
    ];
    for (width, height, expected) in dimensions {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        assert_eq!(session.tree().width(Size { width, height }), expected);
        assert!(session.tree().visible());
        assert_eq!(session.tree().focused(), expected > 0);
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.column < width && cursor.row < height);
        }
        assert_eq!(session.tree().root(), Some(root.as_path()));
    }
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 0);
    session.handle(Event::Resize(31, 8));
    session.insert('!');
    assert_eq!(text(&session), "!");
    session.handle(Event::Resize(48, 8));
    assert!(!session.tree().focused());
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 24);
    tree(&mut session, TreeCommand::Toggle);
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 7);
}

// Transient M-x input returns to its retained workspace owner on cancel/error.
// An editor argument prompt keeps full-width echo/cursor and rejects tree toggles.
#[test]
fn command_focus() {
    let mut session = editor();
    tree(&mut session, TreeCommand::Toggle);
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("toggle-file-tree".into()));
    let frame = session.frame().unwrap();
    let query = frame
        .rows()
        .iter()
        .find(|row| row.band() == rev::display::Band::Echo)
        .unwrap();
    assert_eq!(frame.cursor().unwrap().row, query.row());
    let candidate = &frame.rows()[usize::from(query.row()) + 1];
    assert_eq!(candidate.band(), rev::display::Band::Completion);
    assert!(candidate.text().starts_with("toggle-file-tree"));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert!(session.tree().focused());
    assert_eq!(session.echo(), "Quit");
    named(&mut session, "missing-command");
    assert_eq!(session.echo(), "Unknown command");
    assert!(session.tree().focused());
    session.handle(key(KeyCode::Tab));
    session.execute(Command::FindFile);
    session.handle(Event::Paste("example.txt".into()));
    tree(&mut session, TreeCommand::Toggle);
    assert_eq!(session.echo(), "a prompt owns input focus");
    assert!(session.tree().visible());
    assert!(!session.tree().focused());
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows().last().unwrap().text().width(), 48);
    let prompt = frame
        .rows()
        .iter()
        .find(|row| row.band() == rev::display::Band::Echo)
        .unwrap();
    assert!(prompt.text().starts_with("Find file: "));
    assert_eq!(frame.cursor().unwrap().row, prompt.row());
    session.execute(Command::KeyboardQuit);
    session.insert('x');
    assert_eq!(text(&session), "x");
}

// Workspace focus blocks document edits and paste, while find-file remains
// available. Returning
// preserves dirty state, mark, cursor and history rather than replacing a buffer.
#[test]
fn focused_admission() {
    use rev::command::{
        BufferCommand, History, LinearCommand, Motion, RectangleCommand, RegionCommand, Scroll,
        WindowCommand,
    };
    let mut session = editor();
    session.replace(0..0, "ab界e\u{301}\nlast").unwrap();
    session.execute(Command::Region(RegionCommand::Set));
    let stamp = session.buffer().unwrap().stamp();
    let point = session.view().point;
    let mark = session.buffer().unwrap().mark();
    tree(&mut session, TreeCommand::Toggle);
    for command in [
        Command::DeleteChar,
        Command::Motion(Motion::BufferStart),
        Command::Scroll(Scroll::PageForward),
        Command::History(History::Undo),
        Command::Buffer(BufferCommand::Kill),
        Command::Linear(LinearCommand::Kill),
        Command::Rectangle(RectangleCommand::Mark),
        Command::Window(WindowCommand::Delete),
    ] {
        session.execute(command);
        assert_eq!(session.echo(), "File tree owns input focus", "{command:?}");
        assert!(session.tree().focused());
    }
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::CONTROL,
    )));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('f'),
        KeyModifiers::CONTROL,
    )));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.band() == rev::display::Band::Completion)
    );
    session.execute(Command::KeyboardQuit);
    assert!(session.tree().focused());
    session.handle(Event::Paste("\x18\x03bad".into()));
    assert_eq!(text(&session), "ab界e\u{301}\nlast");
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.view().point, point);
    assert_eq!(session.buffer().unwrap().mark(), mark);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(session.execute(Command::Quit), Action::Redraw);
    assert!(session.echo().ends_with("y=save, n=discard, C-g=cancel"));
    session.execute(Command::KeyboardQuit);
    assert!(session.tree().focused());
    session.execute(Command::KeyboardQuit);
    assert!(!session.tree().focused());
    assert_eq!(session.buffer().unwrap().mark(), mark);
    assert!(session.buffer().unwrap().mark_active());
}

// Selection/search include the tree offset and the six-cell factory gutter.
// Prompt coordinates stay in the global echo, and cancel restores the live point.
#[test]
fn overlay_coordinates() {
    use rev::command::{Motion, RegionCommand};
    let mut session = editor();
    session
        .replace(0..0, "ab target tail\nsecond target")
        .unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::LineEnd));
    tree(&mut session, TreeCommand::Toggle);
    assert_eq!(session.frame().unwrap().rows()[0].selection(), Some(0..16));
    session.handle(key(KeyCode::Tab));
    assert_eq!(session.frame().unwrap().rows()[0].selection(), Some(25..37));
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("target".into()));
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].search(), Some(26..32));
    assert_eq!(frame.cursor().unwrap().row, 7);
    session.execute(Command::KeyboardQuit);
    assert_eq!(session.view().point.position.0, 0);
    assert_eq!(text(&session), "ab target tail\nsecond target");
}

// A later file visit uses the common owner without relocating an explicitly
// chosen root. No Git metadata is needed by this real temporary directory.
#[test]
fn retained_root() {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "rev-tree-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("example.txt");
    fs::write(&path, b"plain text").unwrap();
    let mut session = editor();
    tree(&mut session, TreeCommand::Toggle);
    let root = session.tree().root().unwrap().to_path_buf();
    session.load(&root).unwrap();
    session.load(&path).unwrap();
    assert!(!session.tree().focused());
    assert!(session.tree().visible());
    assert_eq!(text(&session), "plain text");
    tree(&mut session, TreeCommand::Toggle);
    tree(&mut session, TreeCommand::Toggle);
    assert_eq!(session.tree().root(), Some(root.as_path()));
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), path);
    assert!(!directory.join(".git").exists());
    fs::remove_dir_all(directory).unwrap();
}

// The side pane reserves width once before nested editing layout. Removing or
// cycling leaves keeps a valid return view and never treats the tree as a leaf.
#[test]
fn nested_layout() {
    use rev::command::{Split, WindowCommand};
    let mut session = editor();
    session.handle(Event::Resize(80, 12));
    session.execute(Command::Split(Split::Below));
    session.execute(Command::Split(Split::Right));
    let first = session.windows().selected_id();
    tree(&mut session, TreeCommand::Toggle);
    assert_eq!(session.windows().order().len(), 3);
    let frame = session.frame().unwrap();
    assert!(frame.rows().iter().all(|row| row.text().width() == 80));
    assert_eq!(frame.cursor().unwrap().column, 0);
    session.handle(key(KeyCode::Tab));
    assert_eq!(session.windows().selected_id(), first);
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 31);
    session.execute(Command::Window(WindowCommand::Delete));
    let retained = session.windows().selected_id();
    tree(&mut session, TreeCommand::Focus);
    session.handle(key(KeyCode::Tab));
    assert_eq!(session.windows().selected_id(), retained);
    session.execute(Command::Window(WindowCommand::Keep));
    assert_eq!(session.windows().order().len(), 1);
    assert!(session.tree().visible());
    assert_eq!(session.frame().unwrap().cursor().unwrap().column, 31);
}

// Repeated prefixes/suffixes cannot toggle twice. Undefined nested keys and
// cancellation retain workspace ownership without leaking a literal character.
#[test]
fn terminal_phases() {
    use crossterm::event::KeyEventKind;
    let mut session = editor();
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::CONTROL,
    )));
    session.handle(key(KeyCode::Char('t')));
    for phase in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut event = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        event.kind = phase;
        session.handle(Event::Key(event));
    }
    assert!(!session.tree().visible());
    session.handle(key(KeyCode::Char('t')));
    assert!(session.tree().focused());
    let mut event = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    event.kind = KeyEventKind::Repeat;
    session.handle(Event::Key(event));
    assert!(session.tree().visible());
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::CONTROL,
    )));
    session.handle(key(KeyCode::Char('t')));
    session.handle(key(KeyCode::Char('q')));
    assert_eq!(session.echo(), "Undefined key sequence");
    assert!(session.tree().focused());
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert!(!session.tree().focused());
    assert!(session.tree().visible());
    assert_eq!(text(&session), "");
    assert!(session.buffer().unwrap().history().is_empty());
}

#[path = "tree/navigation.rs"]
mod navigation;
