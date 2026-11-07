use super::*;
use std::time::{Duration, Instant};

pub(super) fn ready(session: &mut Session, mut condition: impl FnMut(&mut Session) -> bool) {
    let until = Instant::now() + Duration::from_secs(5);
    while !condition(session) {
        assert!(
            Instant::now() < until,
            "Project operation did not settle: {}",
            session.echo()
        );
        assert_ne!(session.expire_input(Instant::now()), Action::Exit);
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(super) fn prompt(session: &mut Session, label: &str) -> bool {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(label))
}
pub(super) fn configured(f: &fixture::Fixture, profile: &str) -> Session {
    let mut session = editor(f, profile);
    session
        .set_context_storage(f.0.join("state/contexts.toml"))
        .unwrap();
    session
}
pub(super) fn catalog(f: &fixture::Fixture) -> Catalog {
    Store::open(&f.0.join("state/contexts.toml")).unwrap().1
}
fn name(session: &mut Session, name: &str) {
    ready(session, |s| prompt(s, "Project name:"));
    key(session, KeyCode::End);
    for _ in 0..64 {
        key(session, KeyCode::Backspace);
    }
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Enter);
}
pub(super) fn promote(session: &mut Session, profile: &str, title: &str) {
    named(session, profile, "project-promote");
    name(session, title);
    ready(session, |s| matches!(s.context(), Context::Project(_)));
}
pub(super) fn choose(session: &mut Session, profile: &str, title: &str) {
    named(session, profile, "project-switch-project");
    ready(session, |s| prompt(s, "Project:"));
    session.handle(Event::Paste(title.into()));
    key(session, KeyCode::Tab);
    key(session, KeyCode::Enter);
}
fn create(session: &mut Session, profile: &str, root: &Path, title: &str) {
    named(session, profile, "project-create");
    session.handle(Event::Paste(root.to_str().unwrap().into()));
    key(session, KeyCode::Enter);
    name(session, title);
    ready(session, |s| matches!(s.context(), Context::Project(_)));
}

// Promotion publishes only metadata/source anchors and retains the exact live
// source owners. A later explicit checkpoint restores ordered in-root and loose
// files from disk, with Unicode positions and the selected file preserved.
#[test]
fn promotion_and_restore() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let first = f.write("root/first", "a界e\u{301}\nsecond\n");
        let second = f.write("root/second", "second\n");
        let loose = f.write("loose", "outside\n");
        let mut session = configured(&f, profile);
        session.load(first.parent().unwrap()).unwrap();
        session.load(&first).unwrap();
        session.insert('!');
        session.load(&second).unwrap();
        session.load(&loose).unwrap();
        session.load(&first).unwrap();
        let before = (
            session.buffer().unwrap().stamp(),
            session.view().point,
            session.buffer().unwrap().history().len(),
        );
        let generation = session.context_generation();
        promote(&mut session, profile, "Source");
        assert_eq!(
            (
                session.buffer().unwrap().stamp(),
                session.view().point,
                session.buffer().unwrap().history().len()
            ),
            before
        );
        assert_ne!(session.context_generation(), generation);
        assert_eq!(text(&session), "!a界e\u{301}\nsecond\n");
        assert_eq!(fs::read_to_string(&first).unwrap(), "a界e\u{301}\nsecond\n");
        let initial = catalog(&f);
        let project = initial.projects[0].clone();
        let saved = initial
            .session(&project.session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap();
        assert_eq!(saved.files.len(), 3);
        assert!(matches!(
            saved.files[0].file,
            FileReference::RootRelative { .. }
        ));
        assert!(matches!(
            saved.files[2].file,
            FileReference::LooseAbsolute(_)
        ));
        assert_eq!(saved.active_file, Some(saved.files[0].file.clone()));
        let bytes = fs::read(f.0.join("state/contexts.toml")).unwrap();
        // Same typed ID neither asks for discard nor replays the old snapshot.
        choose(&mut session, profile, "Source");
        ready(&mut session, |s| s.echo().contains("already active"));
        assert_eq!(session.buffer().unwrap().stamp(), before.0);
        assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
        named(&mut session, profile, "context-close");
        assert!(session.echo().contains("Leave context"));
        cancel(&mut session);
        assert_eq!(session.buffer().unwrap().stamp(), before.0);
        assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
        named(&mut session, profile, "context-close");
        key(&mut session, KeyCode::Char('n'));
        ready(&mut session, |s| matches!(s.context(), Context::Standalone));
        let checkpoint = catalog(&f);
        assert!(checkpoint.projects[0].session_ref.revision > project.session_ref.revision);
        choose(&mut session, profile, "Source");
        ready(&mut session, |s| matches!(s.context(), Context::Project(_)));
        assert_eq!(text(&session), "a界e\u{301}\nsecond\n");
        assert_eq!(session.view().point.position.0, 1);
        assert_ne!(session.buffer().unwrap().id(), before.0.buffer);
        assert!(
            session
                .tree()
                .root()
                .is_some_and(|r| r == first.parent().unwrap())
        );
        session.load(&loose).unwrap();
        assert_eq!(text(&session), "outside\n");
    }
}

// Path/name cancellation and duplicate aliases do not register partial Projects
// or silently replace the saved session; conflicting name input stays editable.
#[test]
fn explicit_registration() {
    let f = fixture::Fixture::new();
    let source = f.write("root/file", "source");
    let other = f.write("other/file", "other");
    let mut session = configured(&f, "non-modal");
    named(&mut session, "non-modal", "project-list");
    ready(&mut session, |s| {
        s.echo().contains("No registered Projects")
    });
    assert!(!f.0.join("state").exists());
    named(&mut session, "non-modal", "project-create");
    cancel(&mut session);
    assert!(!f.0.join("state").exists());
    create(
        &mut session,
        "non-modal",
        source.parent().unwrap(),
        "Source",
    );
    assert_eq!(catalog(&f).projects.len(), 1);
    session.execute(Command::ContextClose);
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    let bytes = fs::read(f.0.join("state/contexts.toml")).unwrap();
    session.load(source.parent().unwrap()).unwrap();
    session.load(&source).unwrap();
    assert!(matches!(session.context(), Context::Directory(_)));
    assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
    symlink(source.parent().unwrap(), f.0.join("alias")).unwrap();
    session.execute(Command::ProjectPromote);
    ready(&mut session, |s| prompt(s, "Project:"));
    cancel(&mut session);
    assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
    session.execute(Command::ProjectCreate);
    session.handle(Event::Paste(
        other.parent().unwrap().to_str().unwrap().into(),
    ));
    key(&mut session, KeyCode::Enter);
    name(&mut session, "Source");
    assert!(session.echo().contains("already registered"));
    assert!(prompt(&mut session, "Project name:"));
    cancel(&mut session);
    assert_eq!(catalog(&f).projects.len(), 1);
}

// Native restoration skips missing/binary sources without creating replacement
// files, clamps changed Unicode text, and keeps missing-root/corrupt metadata.
#[test]
fn unavailable_sources() {
    let f = fixture::Fixture::new();
    let first = f.write("root/first", "original\nlonger line\n");
    let missing = f.write("root/missing", "missing");
    let binary = f.write("root/binary", "ordinary");
    let mut session = configured(&f, "non-modal");
    session.load(first.parent().unwrap()).unwrap();
    session.load(&missing).unwrap();
    session.load(&binary).unwrap();
    session.load(&first).unwrap();
    session.execute(Command::named("end-of-buffer").unwrap());
    promote(&mut session, "non-modal", "Source");
    session.execute(Command::ContextClose);
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    fs::remove_file(&missing).unwrap();
    fs::write(&binary, b"bad\0source").unwrap();
    fs::write(&first, "界\u{301}").unwrap();
    choose(&mut session, "non-modal", "Source");
    ready(&mut session, |s| matches!(s.context(), Context::Project(_)));
    assert_eq!(text(&session), "界\u{301}");
    assert!(session.view().point.position.0 <= 2);
    assert!(session.echo().contains("2 unavailable"));
    assert!(!missing.exists());
    session.execute(Command::ContextClose);
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    let c = catalog(&f);
    assert_eq!(
        c.session(&c.projects[0].session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .files
            .len(),
        3
    );
    let state = f.0.join("state/contexts.toml");
    let bytes = fs::read(&state).unwrap();
    fs::rename(first.parent().unwrap(), f.0.join("moved")).unwrap();
    choose(&mut session, "non-modal", "Source");
    ready(&mut session, |s| s.echo().contains("root unavailable"));
    assert_eq!(session.context(), &Context::Standalone);
    assert_eq!(fs::read(&state).unwrap(), bytes);
    fs::write(&state, "invalid catalog").unwrap();
    session.execute(Command::ProjectList);
    ready(&mut session, |s| s.echo().contains("Contexts:"));
    assert_eq!(fs::read_to_string(state).unwrap(), "invalid catalog");
}

// A busy publication lock or a competing instance cannot report a completed
// departure. Approved ordinary saves stay on disk while live source ownership
// survives the rejected metadata write; clean quit checkpoints before Exit.
#[test]
fn checked_publication() {
    let f = fixture::Fixture::new();
    let source = f.write("root/file", "original");
    let mut first = configured(&f, "non-modal");
    first.load(source.parent().unwrap()).unwrap();
    first.load(&source).unwrap();
    promote(&mut first, "non-modal", "Source");
    let mut second = configured(&f, "non-modal");
    choose(&mut second, "non-modal", "Source");
    ready(&mut second, |s| matches!(s.context(), Context::Project(_)));
    let state = f.0.join("state/contexts.toml");
    let before = fs::read(&state).unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.0.join("state/contexts.toml.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    first.insert('!');
    first.execute(Command::ContextClose);
    key(&mut first, KeyCode::Char('y'));
    ready(&mut first, |s| s.echo().contains("busy"));
    assert!(matches!(first.context(), Context::Project(_)));
    assert_eq!(fs::read(&state).unwrap(), before);
    assert_eq!(fs::read_to_string(&source).unwrap(), "!original");
    assert_eq!(text(&first), "!original");
    drop(lock);
    first.execute(Command::ContextClose);
    ready(&mut first, |s| matches!(s.context(), Context::Standalone));
    let checkpoint = fs::read(&state).unwrap();
    second.execute(Command::ContextClose);
    ready(&mut second, |s| s.echo().contains("stale catalog"));
    assert!(matches!(second.context(), Context::Project(_)));
    assert_eq!(fs::read(&state).unwrap(), checkpoint);
    let mut quitting = configured(&f, "non-modal");
    choose(&mut quitting, "non-modal", "Source");
    ready(&mut quitting, |s| {
        matches!(s.context(), Context::Project(_))
    });
    quitting.execute(Command::named("end-of-buffer").unwrap());
    ready(&mut quitting, |s| {
        s.frame().unwrap();
        s.view().point.position.0 == s.buffer().unwrap().len_chars()
    });
    assert_ne!(quitting.execute(Command::Quit), Action::Exit);
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        if quitting.expire_input(Instant::now()) == Action::Exit {
            break;
        }
        assert!(Instant::now() < until, "{}", quitting.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
    let c = catalog(&f);
    let saved = c
        .session(&c.projects[0].session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(saved.files[0].cursor.column, 10);
}

// A picker snapshot carries the selected record revision, not its row or label.
// Changes by another instance and canceled background reads cannot activate an
// obsolete destination or replace the live source behind a canceled prompt.
#[test]
fn picker_freshness() {
    let f = fixture::Fixture::new();
    let source = f.write("root/file", "original");
    let mut writer = configured(&f, "non-modal");
    create(&mut writer, "non-modal", source.parent().unwrap(), "Source");
    let mut picker = configured(&f, "non-modal");
    picker.execute(Command::ProjectList);
    ready(&mut picker, |s| prompt(s, "Project:"));
    writer.execute(Command::ContextClose);
    ready(&mut writer, |s| matches!(s.context(), Context::Standalone));
    picker.handle(Event::Paste("Source".into()));
    key(&mut picker, KeyCode::Tab);
    key(&mut picker, KeyCode::Enter);
    ready(&mut picker, |s| {
        s.echo().contains("Selected Project changed")
    });
    assert_eq!(picker.context(), &Context::Standalone);
    let loose = f.write("loose", "loose");
    picker.load(&loose).unwrap();
    picker.insert('!');
    let source_stamp = picker.buffer().unwrap().stamp();
    choose(&mut picker, "non-modal", "Source");
    ready(&mut picker, |s| s.echo().contains("Leave context"));
    choose(&mut writer, "non-modal", "Source");
    ready(&mut writer, |s| matches!(s.context(), Context::Project(_)));
    writer.execute(Command::ContextClose);
    ready(&mut writer, |s| matches!(s.context(), Context::Standalone));
    key(&mut picker, KeyCode::Char('n'));
    ready(&mut picker, |s| {
        s.echo().contains("changed before activation")
    });
    assert_eq!(picker.context(), &Context::Standalone);
    assert_eq!(picker.buffer().unwrap().stamp(), source_stamp);
    assert_eq!(text(&picker), "!loose");
    picker.execute(Command::ProjectList);
    cancel(&mut picker);
    ready(&mut picker, |s| s.echo().contains("preparation canceled"));
    assert_eq!(picker.context(), &Context::Standalone);
    assert!(!prompt(&mut picker, "Project:"));
}

// Restoration batches cross the sixteen-file boundary, preserve stable order,
// and clamp a saved scalar that now falls inside a combining grapheme. Native
// rendering and a subsequent edit must accept the resulting cursor.
#[test]
fn staged_restoration() {
    let f = fixture::Fixture::new();
    let root = f.0.join("root");
    let mut session = configured(&f, "non-modal");
    let mut paths = Vec::new();
    for index in 0..17 {
        paths.push(f.write(&format!("root/file-{index:02}"), "ab"));
    }
    fs::write(
        &paths[0],
        (0..40)
            .map(|i| format!("line {i:02}\n"))
            .collect::<String>(),
    )
    .unwrap();
    session.load(&root).unwrap();
    session.load(&paths[0]).unwrap();
    session.frame().unwrap();
    session.execute(Command::named("scroll-up-command").unwrap());
    ready(&mut session, |s| {
        s.frame().unwrap();
        s.view().top > 0
    });
    let retained_view = (session.view().point.position, session.view().top);
    for path in paths.iter().skip(1) {
        session.load(path).unwrap();
    }
    session.insert('!');
    promote(&mut session, "non-modal", "Batch");
    let bytes = fs::read(f.0.join("state/contexts.toml")).unwrap();
    let generation = session.context_generation();
    session.execute(Command::ContextClose);
    key(&mut session, KeyCode::Char('n'));
    cancel(&mut session);
    assert_eq!(session.context_generation(), generation);
    assert!(matches!(session.context(), Context::Project(_)));
    assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::ContextClose);
    key(&mut session, KeyCode::Char('n'));
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    fs::write(paths.last().unwrap(), "a\u{301}b").unwrap();
    choose(&mut session, "non-modal", "Batch");
    ready(&mut session, |s| matches!(s.context(), Context::Project(_)));
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        paths.last().unwrap()
    );
    assert_eq!(session.view().point.position.0, 0);
    session.frame().unwrap();
    session.insert('x');
    assert_eq!(text(&session), "xa\u{301}b");
    session.load(&paths[0]).unwrap();
    assert_eq!(
        (session.view().point.position, session.view().top),
        retained_view
    );
    let c = catalog(&f);
    let saved = c
        .session(&c.projects[0].session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(saved.files.len(), 17);
    for (entry, path) in saved.files.iter().zip(paths) {
        assert_eq!(
            entry
                .file
                .resolve(std::slice::from_ref(&c.projects[0].root))
                .unwrap(),
            path
        );
    }
}

// Live promotion does not detach an active Org clock. A later departure uses
// its native close/keep decision before resolving modified source buffers and
// checkpointing the Project; cancel retains the exact live clock source.
#[test]
fn project_clock_departure() {
    use rev::command::OrgCommand;
    use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
    fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
        ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
            Ok(std::time::UNIX_EPOCH + Duration::from_secs(1709303400))
        })
    }
    let f = fixture::Fixture::new();
    let source = f.write("root/clock.org", "* Task\nbody");
    let mut session = configured(&f, "non-modal").with_org_clock(clock);
    session.load(source.parent().unwrap()).unwrap();
    session.load(&source).unwrap();
    session.execute(Command::Org(OrgCommand::ClockIn));
    ready(&mut session, |s| text(s).contains("CLOCK:"));
    let source_text = text(&session);
    promote(&mut session, "non-modal", "Clock");
    session.execute(Command::ContextClose);
    assert!(session.echo().contains("detach"));
    cancel(&mut session);
    assert_eq!(text(&session), source_text);
    assert!(matches!(session.context(), Context::Project(_)));
    session.execute(Command::ContextClose);
    session.handle(Event::Paste("keep".into()));
    key(&mut session, KeyCode::Enter);
    assert!(session.echo().contains("y=save"));
    key(&mut session, KeyCode::Char('y'));
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    assert_eq!(fs::read_to_string(source).unwrap(), source_text);
    let c = catalog(&f);
    assert_eq!(
        c.session(&c.projects[0].session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .files
            .len(),
        1
    );
}

// CLI/library directory paths remain native bytes through promotion, storage
// and typed selection. A lossy picker label cannot become the filesystem path.
#[test]
fn native_project_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let f = fixture::Fixture::new();
    let root = f.0.join(OsString::from_vec(b"root-\xff".to_vec()));
    fs::create_dir(&root).unwrap();
    let source = root.join("source");
    fs::write(&source, "native").unwrap();
    let mut session = configured(&f, "non-modal");
    session.load(&root).unwrap();
    session.load(&source).unwrap();
    promote(&mut session, "non-modal", "界 Source");
    assert_eq!(catalog(&f).projects[0].root.path.as_path(), root);
    session.execute(Command::ContextClose);
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    choose(&mut session, "non-modal", "界 Source");
    ready(&mut session, |s| matches!(s.context(), Context::Project(_)));
    assert_eq!(text(&session), "native");
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), source);
}

// A failure before initial publication preserves unregistered dirty directory
// work; cancellation and root substitution during naming have the same boundary.
#[test]
fn failed_promotion() {
    let f = fixture::Fixture::new();
    let source = f.write("root/file", "source");
    let lock_path = f.write("state/contexts.toml.lock", "");
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(lock_path)
        .unwrap();
    lock.try_lock().unwrap();
    let mut session = configured(&f, "non-modal");
    session.load(source.parent().unwrap()).unwrap();
    session.load(&source).unwrap();
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::ProjectPromote);
    name(&mut session, "Source");
    ready(&mut session, |s| s.echo().contains("busy"));
    assert!(matches!(session.context(), Context::Directory(_)));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), "!source");
    assert!(!f.0.join("state/contexts.toml").exists());
    drop(lock);
    session.execute(Command::ProjectPromote);
    ready(&mut session, |s| prompt(s, "Project name:"));
    cancel(&mut session);
    assert!(!f.0.join("state/contexts.toml").exists());
    session.execute(Command::ProjectPromote);
    ready(&mut session, |s| prompt(s, "Project name:"));
    fs::rename(source.parent().unwrap(), f.0.join("moved")).unwrap();
    fs::create_dir(source.parent().unwrap()).unwrap();
    key(&mut session, KeyCode::Enter);
    assert!(session.echo().contains("directory changed"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(!f.0.join("state/contexts.toml").exists());
}
