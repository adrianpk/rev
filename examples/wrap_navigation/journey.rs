use std::error::Error;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use rev::buffer::Buffers;
use rev::command::{Command, History, Motion, OrgCommand, Split};
use rev::config::{Snapshot, default_template, prepare};
use rev::display::Band;
use rev::session::Session;
use rev::view::{Size, View};

fn key(session: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    session.handle(Event::Key(KeyEvent::new(code, modifiers)));
}

fn control(session: &mut Session, value: char) {
    key(session, KeyCode::Char(value), KeyModifiers::CONTROL);
}

fn enter(session: &mut Session) {
    key(session, KeyCode::Enter, KeyModifiers::NONE);
}

fn extended(session: &mut Session, name: &str) {
    key(session, KeyCode::Char('x'), KeyModifiers::ALT);
    session.handle(Event::Paste(name.into()));
    enter(session);
}

fn source(session: &Session) -> Vec<u8> {
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    bytes
}

fn find(session: &mut Session) {
    control(session, 'x');
    control(session, 'f');
}

fn settle(session: &mut Session) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame()?;
        let now = Instant::now();
        if session.remaining_input(now).is_none() {
            return Ok(());
        }
        assert!(now < deadline, "native work did not settle");
        session.expire_input(now + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

// One real event/composition journey must work with absent-file defaults and
// both the packaged and compiled complete templates, not merely equal TOML.
fn journey(root: &Path, settings: Snapshot) -> Result<(), Box<dyn Error>> {
    let buffers = Buffers::with_settings(settings.clone())?;
    let view = View::with_settings(buffers.get(buffers.initial())?, &settings);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )?;
    let work = root.join("workspace");
    let rust = std::fs::read(work.join("alpha.rs"))?;
    let org = std::fs::read(work.join("bravo.org"))?;
    let cwd = std::env::current_dir()?;
    let scratch = session.view().point.stamp.buffer;
    session.load(&work)?;
    assert_eq!(session.view().point.stamp.buffer, scratch);
    assert!(session.tree().visible() && session.tree().focused());
    assert_eq!(session.tree().entries().len(), 4);
    assert!(
        !session
            .frame()?
            .rows()
            .iter()
            .any(|r| r.text().contains("nested.txt"))
    );

    find(&mut session);
    session.handle(Event::Paste("a".into()));
    key(&mut session, KeyCode::Tab, KeyModifiers::NONE);
    let frame = session.frame()?;
    let query = frame
        .rows()
        .iter()
        .find(|r| r.band() == Band::Echo)
        .unwrap()
        .text();
    assert_eq!(query.trim_end(), "Find file: alpha.rs");
    assert_eq!(
        session.view().point.stamp.buffer,
        scratch,
        "Tab accepted a file"
    );
    enter(&mut session);
    assert_eq!(
        session.buffer()?.file().unwrap().path(),
        work.join("alpha.rs")
    );
    assert_eq!(source(&session), rust);
    assert!(session.buffer()?.soft_wrap());
    let rust_view = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let org_view = session.windows().order()[1];
    session.select_view(org_view)?;
    let frame = session.frame()?;
    let row = frame
        .rows()
        .iter()
        .find(|r| r.text().split('│').next().unwrap().contains("bravo.org"))
        .unwrap()
        .row();
    session.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row,
        modifiers: KeyModifiers::NONE,
    }));
    assert_eq!(session.windows().selected_id(), org_view);
    assert_eq!(source(&session), org);
    assert_eq!(
        session.tree().selected().unwrap().path(),
        work.join("bravo.org")
    );
    assert_eq!(session.tree().root(), Some(work.as_path()));
    session.execute(Command::Org(OrgCommand::HideSubtree));
    settle(&mut session)?;
    let frame = session.frame()?;
    assert!(!frame.rows().iter().any(|r| r.text().contains("HIDDEN")));
    assert_eq!(source(&session), org);
    assert!(session.buffer()?.history().is_empty());

    session.select_view(rust_view)?;
    assert_eq!(source(&session), rust);
    control(&mut session, 'n');
    let point = session.view().point.position;
    assert!(point.0 > 0 && point.0 < rust.iter().position(|b| *b == b'\n').unwrap());
    control(&mut session, 'a');
    let start = session.view().point.position;
    control(&mut session, 'k');
    let killed = session.killed_text().unwrap().to_string();
    assert!(!killed.is_empty() && !killed.contains('\n'));
    assert_eq!(session.buffer()?.history().len(), 1);
    let mut expected = rust.clone();
    expected.drain(start.0..start.0 + killed.len());
    assert_eq!(source(&session), expected);
    control(&mut session, 'y');
    assert_eq!(source(&session), rust);
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), expected);
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), rust);
    assert!(!session.buffer()?.is_modified());

    session.execute(Command::Motion(Motion::BufferStart));
    control(&mut session, ' ');
    control(&mut session, 'n');
    control(&mut session, 'n');
    let end = session.view().point.position.0;
    assert!(
        session
            .frame()?
            .rows()
            .iter()
            .filter(|r| r.selection().is_some())
            .count()
            >= 2
    );
    key(&mut session, KeyCode::Char('w'), KeyModifiers::ALT);
    assert_eq!(
        session.killed_text().unwrap().to_string().as_bytes(),
        &rust[..end]
    );
    control(&mut session, 'g');
    session.execute(Command::Motion(Motion::BufferStart));
    control(&mut session, 's');
    session.handle(Event::Paste("NEEDLE".into()));
    let target = rust.windows(6).position(|b| b == b"NEEDLE").unwrap() + 6;
    assert_eq!(session.view().point.position.0, target);
    assert!(session.frame()?.cursor().is_some());
    control(&mut session, 'g');
    assert_eq!(
        session.view().point.position.0,
        0,
        "search cancellation lost origin"
    );

    for width in [48, 100, 80] {
        session.handle(Event::Resize(width, 24));
        assert!(session.frame()?.cursor().is_some());
        assert_eq!(source(&session), rust);
    }
    extended(&mut session, "visual-line-mode");
    assert!(!session.buffer()?.soft_wrap());
    control(&mut session, 'e');
    assert_eq!(session.view().point.position.0, rust.len() - 1);
    assert!(session.frame()?.cursor().is_some());
    extended(&mut session, "visual-line-mode");
    assert!(session.buffer()?.soft_wrap());
    assert_eq!(source(&session), rust);
    find(&mut session);
    session.handle(Event::Paste("not-created".into()));
    control(&mut session, 'g');
    assert_eq!(source(&session), rust);
    assert!(!work.join("not-created").exists());
    session.select_view(org_view)?;
    assert_eq!(source(&session), org);
    assert!(session.buffer()?.history().is_empty());
    assert_eq!(std::env::current_dir()?, cwd);
    assert_eq!(std::fs::read(work.join("alpha.rs"))?, rust);
    assert_eq!(std::fs::read(work.join("bravo.org"))?, org);
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    let packaged = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/reference/config.toml");
    let compiled = root.join("compiled-template.toml");
    std::fs::write(&compiled, default_template())?;
    journey(root, Snapshot::default())?;
    for path in [packaged, compiled] {
        let startup = prepare(&path);
        assert!(startup.diagnostic.is_none());
        journey(root, startup.snapshot)?;
    }
    println!(
        "PASS native joint journey: factory, packaged and compiled templates; directory/Tab/click, Rust/Org splits, wrap kill/selection/search, resize, cancellation and explicit no-wrap"
    );
    Ok(())
}
