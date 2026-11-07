use super::*;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{BufferCommand, Command, History, TreeCommand};
use rev::config::prepare;
use rev::session::{Action, Session};
use rev::view::{Size, View};
use std::fs;
use std::os::unix::fs::symlink;

fn editor(f: &fixture::Fixture, profile: &str) -> Session {
    let settings = prepare(&f.write(
        "config.toml",
        &format!("editor.interaction_profile = '{profile}'"),
    ));
    assert!(settings.diagnostic.is_none());
    let buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap()
}
fn key(session: &mut Session, code: KeyCode) -> Action {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
}
fn cancel(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn named(session: &mut Session, profile: &str, name: &str) {
    if profile == "modal" {
        key(session, KeyCode::Char(':'));
    } else {
        session.execute(Command::ExecuteExtended);
    }
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Enter);
}
fn directory(session: &mut Session, path: &Path) {
    session.execute(Command::OpenDirectory);
    session.handle(Event::Paste(path.to_str().unwrap().into()));
    key(session, KeyCode::Enter);
}

// Tree presentation and ordinary file visits retain context intent. Explicit
// directory selection replaces only after admission, then canonical aliases and
// outside files share that temporary context without changing its root.
#[test]
fn ordinary_contexts() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let source = f.write("chosen/source", "original");
        let outside = f.write("outside", "outside");
        symlink(&source, f.0.join("alias")).unwrap();
        let mut session = editor(&f, profile);
        session.load(&source).unwrap();
        let standalone = session.context_generation();
        let original = session.buffer().unwrap().id();
        assert_eq!(session.context(), &Context::Standalone);
        assert!(!session.tree().visible());
        session.execute(Command::Tree(TreeCommand::Focus));
        assert_eq!(session.context(), &Context::Standalone);
        assert_eq!(session.context_generation(), standalone);
        cancel(&mut session);
        directory(&mut session, source.parent().unwrap());
        let directory_generation = session.context_generation();
        assert_ne!(directory_generation, standalone);
        assert!(
            matches!(session.context(), Context::Directory(path) if path.as_path() == source.parent().unwrap())
        );
        assert_ne!(session.buffer().unwrap().id(), original);
        assert_eq!(text(&session), "");
        session.load(&source).unwrap();
        session.insert('!');
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point;
        let history = session.buffer().unwrap().history().len();
        session.load(&f.0.join("alias")).unwrap();
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.view().point, point);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert_eq!(text(&session), "!original");
        session.load(&outside).unwrap();
        assert_eq!(text(&session), "outside");
        session.load(&f.0.join("alias")).unwrap();
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&session), "!original");
        assert_eq!(session.context_generation(), directory_generation);
        assert_eq!(session.tree().root(), source.parent());
        assert!(!f.0.join("contexts.toml").exists());
        assert!(!source.parent().unwrap().join(".rev").exists());
    }
}

// Both named entrypoints use the same source decisions. Cancel preserves text,
// undo, stamps, windows and context; save/discard close only after the decision.
#[test]
fn guarded_departure() {
    for profile in ["non-modal", "modal"] {
        for decision in ["cancel", "save", "discard"] {
            let f = fixture::Fixture::new();
            let source = f.write("first/source", "original");
            f.write("second/next", "next");
            let mut session = editor(&f, profile);
            session.load(&source).unwrap();
            session.insert('!');
            let stamp = session.buffer().unwrap().stamp();
            let view = session.view().clone();
            let generation = session.context_generation();
            named(&mut session, profile, "open-directory");
            session.handle(Event::Paste(f.0.join("second").to_str().unwrap().into()));
            key(&mut session, KeyCode::Enter);
            assert_eq!(session.context_generation(), generation);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert!(session.echo().contains("Leave context"));
            match decision {
                "cancel" => {
                    cancel(&mut session);
                    assert_eq!(session.context_generation(), generation);
                    assert_eq!(session.view(), &view);
                    assert_eq!(session.buffer().unwrap().stamp(), stamp);
                    assert_eq!(text(&session), "!original");
                    session.execute(Command::History(History::Undo));
                    assert_eq!(text(&session), "original");
                    assert_eq!(session.execute(Command::Quit), Action::Exit);
                }
                answer => {
                    assert_ne!(
                        key(
                            &mut session,
                            KeyCode::Char(if answer == "save" { 'y' } else { 'n' })
                        ),
                        Action::Exit
                    );
                    assert_ne!(session.context_generation(), generation);
                    assert!(matches!(session.context(), Context::Directory(_)));
                    assert_eq!(text(&session), "");
                }
            }
            assert_eq!(
                fs::read(&source).unwrap(),
                if decision == "save" {
                    b"!original".as_slice()
                } else {
                    b"original".as_slice()
                }
            );
        }
    }
}

// Closing resolves unnamed text through native write/save or discard. New
// buffers remain local to the active context; old names leave its inventory.
#[test]
fn unnamed_and_close() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let mut session = editor(&f, profile);
        let generation = session.context_generation();
        named(&mut session, profile, "generate-new-buffer");
        session.insert('Δ');
        assert_eq!(session.context_generation(), generation);
        let old = session.buffer().unwrap().id();
        named(&mut session, profile, "context-close");
        key(&mut session, KeyCode::Char('y'));
        let target = f.0.join("saved");
        session.handle(Event::Paste(target.to_str().unwrap().into()));
        assert_ne!(key(&mut session, KeyCode::Enter), Action::Exit);
        assert_ne!(session.context_generation(), generation);
        assert_ne!(session.buffer().unwrap().id(), old);
        assert_eq!(fs::read_to_string(target).unwrap(), "Δ");
        assert_eq!(session.context(), &Context::Standalone);
        assert!(!session.tree().visible());
        assert_eq!(session.windows().iter().count(), 1);
        assert_eq!(text(&session), "");
        session.execute(Command::Buffer(BufferCommand::List));
        let frame = session.frame().unwrap();
        assert!(!frame.rows().iter().any(|row| row.text().contains("saved")));
        cancel(&mut session);
        assert_eq!(session.execute(Command::Quit), Action::Exit);
    }
}

// A directory picker rejects files and preserves the retained owner on cancel.
// A destination swapped after source decisions never activates the replacement.
#[test]
fn directory_revalidation() {
    let f = fixture::Fixture::new();
    let source = f.write("source", "source");
    f.write("target/file", "target");
    let mut session = editor(&f, "non-modal");
    session.load(&source).unwrap();
    let stamp = session.buffer().unwrap().stamp();
    directory(&mut session, &source);
    assert!(session.echo().contains("existing directory"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    cancel(&mut session);
    session.insert('!');
    directory(&mut session, &f.0.join("target"));
    fs::rename(f.0.join("target"), f.0.join("retained-target")).unwrap();
    fs::create_dir(f.0.join("target")).unwrap();
    key(&mut session, KeyCode::Char('n'));
    assert!(session.echo().contains("Prepared directory changed"));
    assert_eq!(text(&session), "!source");
    assert_eq!(session.context(), &Context::Standalone);
    assert_eq!(fs::read(&source).unwrap(), b"source");
    cancel(&mut session);
    session.execute(Command::ContextClose);
    key(&mut session, KeyCode::Char('n'));
    assert_eq!(text(&session), "");
}

// Ordinary temporary-directory use neither discovers nor checkpoints a named
// Project at that path; source saving affects only the chosen source bytes.
#[test]
fn registered_path_is_temporary() {
    let f = fixture::Fixture::new();
    let source = f.write("root/source", "source");
    let path = f.0.join("state/contexts.toml");
    let (store, empty) = Store::open(&path).unwrap();
    let mut catalog = empty.clone();
    project(&mut catalog, source.parent().unwrap(), "Existing");
    store.publish(&empty.prepare(catalog).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    let mut session = editor(&f, "non-modal");
    session.load(source.parent().unwrap()).unwrap();
    assert!(matches!(session.context(), Context::Directory(_)));
    session.load(&source).unwrap();
    session.insert('!');
    session.execute(Command::SaveBuffer);
    session.execute(Command::ContextClose);
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read(&source).unwrap(), b"!source");
    assert_eq!(session.context(), &Context::Standalone);
}

// Histories, bookmarks and the kill ring belong to the application, while their
// old buffer/view identities must retire. Bookmark navigation reopens source
// through its native checked resolver after the context has been closed.
#[test]
fn retained_application_state() {
    use rev::command::{BookmarkCommand, KillCommand, LinearCommand};
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let source = f.write("source", "retained");
        let mut session = editor(&f, profile);
        session.load(&source).unwrap();
        let old = session.buffer().unwrap().id();
        session.execute(Command::Bookmark(BookmarkCommand::Set));
        session.handle(Event::Paste("here".into()));
        key(&mut session, KeyCode::Enter);
        session.execute(Command::Kill(KillCommand::Line));
        assert_eq!(text(&session), "");
        named(&mut session, profile, "context-close");
        key(&mut session, KeyCode::Char('n'));
        session.execute(Command::Linear(LinearCommand::Yank));
        assert_eq!(text(&session), "retained");
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::ALT,
        )));
        let frame = session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .any(|row| row.band() == rev::display::Band::Echo
                    && row.text().contains("context-close"))
        );
        cancel(&mut session);
        session.execute(Command::Bookmark(BookmarkCommand::Jump));
        session.handle(Event::Paste("here".into()));
        key(&mut session, KeyCode::Enter);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while session.buffer().unwrap().file().is_none() && std::time::Instant::now() < deadline {
            session.expire_input(std::time::Instant::now());
            std::thread::yield_now();
        }
        assert_eq!(session.buffer().unwrap().file().unwrap().path(), source);
        assert_ne!(session.buffer().unwrap().id(), old);
        assert_eq!(text(&session), "retained");
    }
}

// A real owned shell process is stopped and reaped before its source context
// retires. Its late output and old observation wake cannot edit the new source.
#[test]
fn retired_async_source() {
    let f = fixture::Fixture::new();
    let config = prepare(&f.write(
        "async.toml",
        "shell.program='/bin/sh'\nfiles.external_change_debounce_ms=0",
    ));
    assert!(config.diagnostic.is_none());
    let source = f.write("source", "source");
    let mut buffers = Buffers::with_settings(config.snapshot).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap();
    let marker = f.0.join("owned-pid");
    session.execute(Command::ShellCommandInsert);
    session.handle(Event::Paste(format!(
        "printf '%s' $$ > '{}'; sleep 10; printf stale",
        marker.display()
    )));
    key(&mut session, KeyCode::Enter);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let pid = loop {
        if let Ok(pid) = fs::read_to_string(&marker)
            .unwrap_or_default()
            .parse::<u32>()
        {
            break pid;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "owned shell did not publish its identity"
        );
        session.expire_input(std::time::Instant::now());
        std::thread::yield_now();
    };
    assert!(Path::new("/proc").join(pid.to_string()).exists());
    let old = session.context_generation();
    assert_ne!(session.execute(Command::ContextClose), Action::Exit);
    assert_ne!(session.context_generation(), old);
    assert!(!Path::new("/proc").join(pid.to_string()).exists());
    fs::write(&source, "changed outside the retired context").unwrap();
    for _ in 0..16 {
        session.expire_input(std::time::Instant::now());
    }
    assert_eq!(text(&session), "");
    assert_ne!(session.buffer().unwrap().id(), id);
    assert!(session.buffer().unwrap().file().is_none());
    assert!(!session.buffer().unwrap().is_modified());
    assert_ne!(
        session.remaining_input(std::time::Instant::now()),
        Some(std::time::Duration::ZERO)
    );
}

// Renaming an explicitly browsed directory keeps the temporary descriptor,
// native tree and open file associations on the same new path without changing
// context generation, source revisions, dirty text or saved file bytes.
#[test]
fn renamed_temporary_root() {
    let f = fixture::Fixture::new();
    let source = f.write("root/source", "source");
    let mut session = editor(&f, "non-modal");
    session.load(source.parent().unwrap()).unwrap();
    session.load(&source).unwrap();
    session.insert('!');
    let generation = session.context_generation();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::Tree(TreeCommand::Focus));
    for _ in 0..session.tree().entries().len() {
        session.execute(Command::Tree(TreeCommand::Previous));
    }
    session.execute(Command::Tree(TreeCommand::Rename));
    session.handle(Event::Paste("renamed".into()));
    key(&mut session, KeyCode::Enter);
    let root = f.0.join("renamed");
    assert!(matches!(session.context(), Context::Directory(path) if path.as_path() == root));
    assert_eq!(session.tree().root(), Some(root.as_path()));
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        root.join("source")
    );
    assert_eq!(session.context_generation(), generation);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), "!source");
    assert_eq!(fs::read(root.join("source")).unwrap(), b"source");
}

// A live terminal requires its own confirmation after source decisions. Decline
// retains the context and child; approval completes bounded owned cleanup and
// activates the replacement without returning an application-exit action.
#[test]
fn terminal_departure() {
    for persistent in [false, true] {
        terminal_context_departure(persistent);
    }
}
fn terminal_context_departure(persistent: bool) {
    use rev::command::TerminalCommand;
    let f = fixture::Fixture::new();
    let script = f.write("terminal.sh", "printf READY\nwhile :; do sleep 1; done\n");
    let config = prepare(&f.write(
        "terminal.toml",
        &format!(
            "terminal.program='/bin/sh'\nterminal.args=[{}]\n",
            toml_edit::Value::from(script.to_str().unwrap())
        ),
    ));
    assert!(config.diagnostic.is_none());
    let buffers = Buffers::with_settings(config.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )
    .unwrap();
    if persistent {
        let source = f.write("root/source", "source");
        session
            .set_context_storage(f.0.join("state/contexts.toml"))
            .unwrap();
        session.load(source.parent().unwrap()).unwrap();
        session.load(&source).unwrap();
        projects::promote(&mut session, "non-modal", "Terminal work");
    }
    session.execute(Command::Terminal(TerminalCommand::NewBuffer));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    let pid = loop {
        if let Some(pid) = session
            .terminals()
            .ids()
            .find_map(|id| session.terminals().get(id).unwrap().process_id())
        {
            break pid;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "terminal did not start: {}",
            session.echo()
        );
        session.expire_input(std::time::Instant::now());
        std::thread::yield_now();
    };
    let generation = session.context_generation();
    session.execute(Command::ContextClose);
    assert!(session.echo().contains("Leave context: terminate"));
    key(&mut session, KeyCode::Char('n'));
    assert_eq!(session.context_generation(), generation);
    assert_eq!(session.terminals().ids().count(), 1);
    assert!(Path::new("/proc").join(pid.to_string()).exists());
    session.execute(Command::ContextClose);
    assert_ne!(key(&mut session, KeyCode::Char('y')), Action::Exit);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while session.context_generation() == generation {
        assert!(
            std::time::Instant::now() < deadline,
            "context cleanup did not complete: {}",
            session.echo()
        );
        assert_ne!(
            session.expire_input(std::time::Instant::now()),
            Action::Exit
        );
        std::thread::yield_now();
    }
    assert_eq!(session.terminals().ids().count(), 0);
    assert!(!Path::new("/proc").join(pid.to_string()).exists());
    assert_eq!(session.context(), &Context::Standalone);
    assert_eq!(text(&session), "");
    if persistent {
        let catalog = projects::catalog(&f);
        let saved = catalog
            .session(&catalog.projects[0].session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap();
        assert_eq!(saved.files.len(), 1);
    }
}

// Context replacement cannot re-enable analysis that the user disabled at
// runtime, even when startup configuration originally enabled it. No source
// file is attached, so this exercises policy without starting a server.
#[test]
fn retained_analysis_choice() {
    use rev::command::LspCommand;
    let f = fixture::Fixture::new();
    let settings = prepare(&f.write("lsp.toml", "lsp.enabled=true"));
    assert!(settings.diagnostic.is_none());
    let buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap();
    session.execute(Command::Lsp(LspCommand::Mode));
    assert!(matches!(session.lsp_status(), rev::lsp::Status::Disabled));
    session.execute(Command::ContextClose);
    assert!(matches!(session.lsp_status(), rev::lsp::Status::Disabled));
    session.execute(Command::Lsp(LspCommand::Mode));
    assert!(!matches!(session.lsp_status(), rev::lsp::Status::Disabled));
}

#[path = "projects.rs"]
mod projects;

#[path = "workspaces.rs"]
mod workspaces;
