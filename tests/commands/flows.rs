use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Stamp, State};
use rev::command::{BufferCommand, Command, Motion, Scroll, Split};
use rev::session::{Action, Session};
use rev::view::{Size, View};
use rev::window::ViewId;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "rev-command-flows-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    text: String,
    stamp: Stamp,
    state: State,
    dirty: bool,
    file: Option<PathBuf>,
    history: usize,
    redo: usize,
    selected: ViewId,
    views: Vec<(ViewId, View)>,
}

fn snapshot(session: &Session) -> Snapshot {
    let buffer = session.buffer().unwrap();
    Snapshot {
        text: text(session),
        stamp: buffer.stamp(),
        state: buffer.state(),
        dirty: buffer.is_modified(),
        file: buffer.file().map(|file| file.path().to_owned()),
        history: buffer.history().len(),
        redo: buffer.redo_history().len(),
        selected: session.windows().selected_id(),
        views: session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect(),
    }
}

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

fn named(session: &mut Session, name: &str) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    payload(session, name);
    session.handle(super::key(KeyCode::Enter));
}

fn submit(session: &mut Session, path: &Path) {
    payload(session, path.to_str().unwrap());
    session.handle(super::key(KeyCode::Enter));
}

fn shared() -> Session {
    let mut session = editor();
    let first = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    session.select_view(session.windows().order()[1]).unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session.select_view(first).unwrap();
    session
}

fn close_stage(session: &mut Session, stage: &str, target: &Path) {
    named(session, "kill-buffer");
    if stage == "name" {
        return;
    }
    session.handle(super::key(KeyCode::Enter));
    if stage == "question" {
        return;
    }
    session.handle(super::character('y'));
    payload(session, target.to_str().unwrap());
    if stage == "destination" {
        return;
    }
    session.handle(super::key(KeyCode::Enter));
    assert!(session.echo().contains("Overwrite occupied destination"));
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

// Accepted and rejected command names retire the transient stage. A subsequent
// editing C-g regains its normal navigation boundary instead of a stale prompt.
#[test]
fn entry_retirement() {
    for name in ["next-line", "absent-command", ""] {
        let mut session = editor();
        named(&mut session, name);
        assert_eq!(session.view().column, Some(8), "{name}");
        session.handle(super::control('g'));
        assert_eq!(session.view().column, None, "{name}");
        assert_eq!(text(&session), "0123456789\nx\nend");
    }
    let mut session = editor();
    let before = session.view().clone();
    named(&mut session, "execute-extended-command");
    session.handle(super::control('g'));
    assert_eq!(*session.view(), before);
    session.handle(super::control('g'));
    assert_eq!(session.view().column, None);
}

// Canceling either the name query or its path stage leaves every live view,
// document identity/history and disk byte intact; fresh editing cannot resume it.
#[test]
fn canceled_paths() {
    for name in ["find-file", "write-file", "save-buffer"] {
        for stage in ["command", "argument"] {
            let fixture = Fixture::new();
            let target = fixture.file("occupied.txt", b"keep");
            let mut session = shared();
            let before = snapshot(&session);
            session.execute(Command::ExecuteExtended);
            payload(&mut session, name);
            if stage == "argument" {
                session.handle(super::key(KeyCode::Enter));
                payload(&mut session, target.to_str().unwrap());
            }
            session.handle(super::control('g'));
            assert_eq!(snapshot(&session), before, "{name} {stage}");
            assert_eq!(fs::read(&target).unwrap(), b"keep");
            session.handle(super::character('!'));
            assert_eq!(text(&session), "0123456789\nx!\nend");
            assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
        }
    }
}

// An overwrite question retains explicit consent after invalid/held/pasted
// responses. Both no and C-g retire save authority without changing source/disk.
#[test]
fn canceled_decisions() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    for answer in [super::control('g'), super::character('n')] {
        let fixture = Fixture::new();
        let target = fixture.file("occupied.txt", b"keep");
        let mut session = shared();
        let before = snapshot(&session);
        named(&mut session, "write-file");
        submit(&mut session, &target);
        session.handle(super::key(KeyCode::Enter));
        payload(&mut session, "y");
        for kind in [Release, Repeat] {
            assert_eq!(
                session.handle(super::phase(super::character('y'), kind)),
                Action::Wait
            );
        }
        assert!(session.echo().contains("Overwrite occupied destination"));
        assert_eq!(snapshot(&session), before);
        session.handle(answer);
        assert_eq!(snapshot(&session), before);
        assert_eq!(fs::read(&target).unwrap(), b"keep");
        session.handle(super::character('y'));
        assert_eq!(text(&session), "0123456789\nxy\nend");
        assert_eq!(fs::read(&target).unwrap(), b"keep");
    }
}

// C-g at each nested close stage drops the captured close target. A later
// independent successful save must retain both panes and the still-open buffer.
#[test]
fn canceled_close() {
    for stage in ["name", "question", "destination", "overwrite"] {
        let fixture = Fixture::new();
        let occupied = fixture.file("occupied.txt", b"keep");
        let fresh = fixture.0.join("fresh.txt");
        let mut session = shared();
        let before = snapshot(&session);
        close_stage(&mut session, stage, &occupied);
        session.handle(super::control('g'));
        assert_eq!(snapshot(&session), before, "{stage}");
        named(&mut session, "write-file");
        submit(&mut session, &fresh);
        assert_eq!(
            session.buffer().unwrap().id(),
            before.stamp.buffer,
            "{stage}"
        );
        assert_eq!(session.windows().iter().len(), 2, "{stage}");
        assert!(!session.buffer().unwrap().is_modified(), "{stage}");
        assert_eq!(
            session.buffer().unwrap().history().len(),
            before.history,
            "{stage}"
        );
        assert_eq!(fs::read(&fresh).unwrap(), before.text.as_bytes(), "{stage}");
        assert_eq!(fs::read(&occupied).unwrap(), b"keep", "{stage}");
    }
}

// Every stage requires a fresh acceptance. Held Return/y reports cannot skip
// from buffer name to close decision, destination, overwrite consent or editing.
#[test]
fn stage_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let fixture = Fixture::new();
    let occupied = fixture.file("occupied.txt", b"keep");
    let mut session = shared();
    let before = snapshot(&session);
    named(&mut session, "kill-buffer");
    session.handle(super::key(KeyCode::Enter));
    for kind in [Release, Repeat] {
        assert_eq!(
            session.handle(super::phase(super::key(KeyCode::Enter), kind)),
            Action::Wait
        );
    }
    session.handle(super::character('y'));
    assert_eq!(
        session.handle(super::phase(super::character('y'), Repeat)),
        Action::Wait
    );
    assert!(echo_row(&mut session).starts_with("Write file:"));
    submit(&mut session, &occupied);
    assert_eq!(
        session.handle(super::phase(super::key(KeyCode::Enter), Repeat)),
        Action::Wait
    );
    assert_eq!(
        session.handle(super::phase(super::character('y'), Repeat)),
        Action::Wait
    );
    session.handle(super::character('n'));
    assert_eq!(
        session.handle(super::phase(super::character('n'), Repeat)),
        Action::Wait
    );
    assert_eq!(snapshot(&session), before);
    assert_eq!(fs::read(occupied).unwrap(), b"keep");
}

// Ingress/save failures return keyboard ownership without publishing text or
// keeping a pending close that could affect a later independent successful save.
#[test]
fn failed_stages() {
    for stage in ["find", "write", "close"] {
        let fixture = Fixture::new();
        let invalid = fixture.file("invalid.txt", b"\xff");
        let missing = fixture.0.join("missing/new.txt");
        let fresh = fixture.0.join("fresh.txt");
        let mut session = shared();
        let before = snapshot(&session);
        match stage {
            "find" => {
                named(&mut session, "find-file");
                submit(&mut session, &invalid);
            }
            "write" => {
                named(&mut session, "write-file");
                submit(&mut session, &missing);
            }
            _ => {
                close_stage(&mut session, "destination", &missing);
                session.handle(super::key(KeyCode::Enter));
            }
        }
        assert_eq!(snapshot(&session), before, "{stage}");
        assert!(!session.echo().is_empty(), "{stage}");
        assert!(!missing.exists(), "{stage}");
        session.handle(super::character('!'));
        named(&mut session, "write-file");
        submit(&mut session, &fresh);
        assert_eq!(
            session.buffer().unwrap().id(),
            before.stamp.buffer,
            "{stage}"
        );
        assert_eq!(session.windows().iter().len(), 2, "{stage}");
        assert_eq!(fs::read(&fresh).unwrap(), b"0123456789\nx!\nend", "{stage}");
    }
}

// Emacs cursor/deletion keys repair a real Unicode path draft before Return;
// only the accepted path opens, while the other pane retains its source owner.
#[test]
fn edited_path() {
    let fixture = Fixture::new();
    let target = fixture.file("e\u{301}界-file.txt", "\u{feff}é\r\nlast".as_bytes());
    let mut session = shared();
    let before = snapshot(&session);
    named(&mut session, "find-file");
    payload(&mut session, &format!("{}BAD", target.display()));
    for _ in 0..3 {
        session.handle(super::control('b'));
    }
    for _ in 0..3 {
        session.handle(super::control('d'));
    }
    for event in [
        super::control('a'),
        super::control('f'),
        super::control('b'),
        super::control('e'),
        super::character('X'),
        super::control('h'),
    ] {
        session.handle(event);
    }
    assert_eq!(snapshot(&session), before);
    session.handle(super::key(KeyCode::Enter));
    assert_eq!(text(&session), "é\r\nlast");
    assert!(session.buffer().unwrap().has_bom());
    assert_eq!(session.windows().selected_id(), before.selected);
    let passive = before
        .views
        .iter()
        .find(|(id, _)| *id != before.selected)
        .unwrap();
    assert_eq!(session.windows().get(passive.0).unwrap(), &passive.1);
    session.handle(super::character('!'));
    assert_eq!(text(&session), "!é\r\nlast");
    assert_eq!(fs::read(target).unwrap(), "\u{feff}é\r\nlast".as_bytes());
}

// Geometry changes retain the current nested argument/decision. Cancel restores
// the same selected buffer and qualified points, with no deferred disk mutation.
#[test]
fn resized_stages() {
    for stage in ["path", "question", "destination", "overwrite"] {
        let fixture = Fixture::new();
        let target = fixture.file("occupied.txt", b"keep");
        let mut session = shared();
        let before = snapshot(&session);
        if stage == "path" {
            named(&mut session, "find-file");
            payload(&mut session, "draft");
        } else {
            close_stage(&mut session, stage, &target);
        }
        for (width, height) in [(1, 1), (0, 0), (100, 10)] {
            session.handle(Event::Resize(width, height));
            let frame = session.frame().unwrap();
            assert_eq!(frame.rows().len(), usize::from(height), "{stage}");
            if let Some(cursor) = frame.cursor() {
                assert!(cursor.column < width && cursor.row < height, "{stage}");
            }
        }
        session.handle(super::control('g'));
        assert_eq!(session.buffer().unwrap().stamp(), before.stamp, "{stage}");
        assert_eq!(session.buffer().unwrap().state(), before.state, "{stage}");
        assert_eq!(session.windows().selected_id(), before.selected, "{stage}");
        let points: Vec<_> = session
            .windows()
            .iter()
            .map(|(_, view)| view.point)
            .collect();
        assert_eq!(
            points,
            before
                .views
                .iter()
                .map(|(_, view)| view.point)
                .collect::<Vec<_>>(),
            "{stage}"
        );
        session.handle(super::character('!'));
        assert_eq!(text(&session), "0123456789\nx!\nend", "{stage}");
        assert_eq!(fs::read(target).unwrap(), b"keep", "{stage}");
    }
}
