use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, Split, TreeCommand};
use rev::config::prepare;
use rev::session::Session;
use rev::view::{Size, View};

#[path = "../configuration/fixture.rs"]
mod fixture;
use fixture::Fixture;

fn editor(fixture: &Fixture, settings: &str) -> Session {
    let startup = prepare(&fixture.write("config.toml", settings));
    assert!(startup.diagnostic.is_none());
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap()
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn accept(session: &mut Session, draft: &str) {
    session.execute(Command::FindFile);
    session.handle(Event::Paste(draft.into()));
    session.handle(key(KeyCode::Enter));
}

fn text(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Directory opening preserves a dirty shared document and exact undo state.
// Explicit roots survive nested/outside files and aliases under every policy.
#[test]
fn root_precedence() {
    for policy in ["auto", "file-parent", "cwd"] {
        let fixture = Fixture::new();
        let nested = fixture.write("chosen/sub/main.txt", "source");
        let outside = fixture.write("outside.txt", "outside");
        let root = fixture.0.join("chosen");
        symlink("chosen/sub", fixture.0.join("alias")).unwrap();
        let mut session = editor(&fixture, &format!("tree.root = '{policy}'"));
        session.load(&nested).unwrap();
        session.insert('!');
        session.execute(Command::Split(Split::Right));
        let point = session.view().point;
        let points = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.point))
            .collect::<Vec<_>>();
        let history = session.buffer().unwrap().history().len();
        accept(&mut session, fixture.0.join("alias/..").to_str().unwrap());
        assert_eq!(session.tree().root(), Some(root.as_path()));
        assert!(session.tree().visible() && session.tree().focused());
        assert_eq!(session.view().point, point);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.point))
                .collect::<Vec<_>>(),
            points
        );
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert_eq!(text(&session), "!source");
        assert_eq!(session.tree().entries().len(), 2);
        assert!(!session.tree().entries()[1].expanded());
        for path in [&outside, &nested] {
            session.load(path).unwrap();
            assert_eq!(session.tree().root(), Some(root.as_path()));
            assert!(session.tree().visible());
        }
        assert_eq!(text(&session), "!source");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "source");
        assert_eq!(fs::read(&nested).unwrap(), b"source");
        session.load(outside.parent().unwrap()).unwrap();
        assert_eq!(session.tree().root(), Some(fixture.0.as_path()));
    }
}

// An empty accepted draft denotes the prompt base. Directory-only sessions
// retain their scratch identity and reuse the chosen context for later paths.
#[test]
fn empty_directory() {
    let fixture = Fixture::new();
    let root = fixture.0.join("empty 界 space");
    fs::create_dir(&root).unwrap();
    let mut session = editor(&fixture, "tree.visible = false");
    let original = session.view().point;
    session.load(&root).unwrap();
    assert_eq!(session.view().point, original);
    assert!(session.buffer().unwrap().file().is_none());
    assert_eq!(session.tree().entries().len(), 1);
    assert!(session.tree().entries()[0].expanded());
    session.execute(Command::Tree(TreeCommand::Toggle));
    assert!(!session.tree().visible());
    accept(&mut session, "");
    assert_eq!(session.tree().root(), Some(root.as_path()));
    assert!(session.tree().visible() && session.tree().focused());
    accept(&mut session, "new.txt");
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        root.join("new.txt")
    );
    assert!(!root.join("new.txt").exists());
    assert_eq!(session.tree().root(), Some(root.as_path()));
}

// Unreadable, replaced and looping directory targets cannot replace a usable
// root or consume the prompt. All source/view/history assertions stay exact.
#[test]
fn rejected_directory() {
    let fixture = Fixture::new();
    let first = fixture.write("first/main.txt", "retained");
    fs::create_dir(fixture.0.join("denied")).unwrap();
    symlink("loop", fixture.0.join("loop")).unwrap();
    let mut session = editor(&fixture, "");
    session.load(&first).unwrap();
    session.insert('!');
    session.load(first.parent().unwrap()).unwrap();
    session.frame().unwrap();
    let view = session.view().clone();
    let history = session.buffer().unwrap().history().len();
    let root = session.tree().root().unwrap().to_owned();
    for target in ["denied", "loop", "main.txt/child"] {
        let path = if target == "main.txt/child" {
            first.join("child")
        } else {
            fixture.0.join(target)
        };
        let denied = fixture.0.join("denied");
        let permissions = fs::metadata(&denied).unwrap().permissions();
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
        accept(&mut session, path.to_str().unwrap());
        fs::set_permissions(&denied, permissions).unwrap();
        assert!(!session.echo().is_empty());
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.band() == rev::display::Band::Completion)
        );
        assert_eq!(session.tree().root(), Some(root.as_path()));
        assert!(session.tree().focused());
        assert_eq!(session.view(), &view);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert_eq!(text(&session), "!retained");
        session.execute(Command::KeyboardQuit);
    }
}
