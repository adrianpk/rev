use super::projects::{catalog, configured, prompt, ready};
use super::*;
use rev::context::RootId;

fn workspace(f: &fixture::Fixture) -> Workspace {
    catalog(f).workspaces.into_iter().next().unwrap()
}
fn command(session: &mut Session, profile: &str, name: &str) {
    // Colon belongs to modal source editing; the tree retains its own M-x map.
    if profile == "modal" && session.tree().focused() {
        cancel(session);
    }
    named(session, profile, name);
}
fn create(session: &mut Session, profile: &str, name: &str) {
    command(session, profile, "workspace-create");
    ready(session, |s| prompt(s, "Workspace name:"));
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Enter);
    ready(session, |s| matches!(s.context(), Context::Workspace(_)));
}
fn choose(session: &mut Session, profile: &str, name: &str) {
    command(session, profile, "workspace-open");
    ready(session, |s| prompt(s, "Workspace:"));
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Tab);
    key(session, KeyCode::Enter);
}
fn add(session: &mut Session, profile: &str, path: &Path) {
    let generation = session.context_generation();
    command(session, profile, "workspace-add-directory");
    session.handle(Event::Paste(path.to_str().unwrap().into()));
    key(session, KeyCode::Enter);
    ready(session, |s| s.context_generation() != generation);
}
fn remove(session: &mut Session, profile: &str, query: &str) {
    command(session, profile, "workspace-remove-root");
    assert!(prompt(session, "Remove Workspace root:"));
    session.handle(Event::Paste(query.into()));
    key(session, KeyCode::Tab);
    key(session, KeyCode::Enter);
}
fn selected_root(session: &mut Session, id: &RootId) {
    session.execute(Command::Tree(TreeCommand::Focus));
    let limit = session.tree().entries().len() + 1;
    for _ in 0..limit {
        if session
            .tree()
            .selected()
            .is_some_and(|e| e.depth() == 0 && e.root_id() == Some(id))
        {
            return;
        }
        session.execute(Command::Tree(TreeCommand::Next));
    }
    for _ in 0..limit {
        if session
            .tree()
            .selected()
            .is_some_and(|e| e.depth() == 0 && e.root_id() == Some(id))
        {
            return;
        }
        session.execute(Command::Tree(TreeCommand::Previous));
    }
    panic!("root identity not reachable");
}

// Empty Workspaces stay valid through focus, loose-file editing and same-ID
// selection. Project-derived roots append only their path/provenance; the source
// Project and its selected file snapshot remain unchanged.
#[test]
fn independent_sessions() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let source = f.write("project/source", "project");
        let loose = f.write("loose", "loose");
        let mut project_session = configured(&f, profile);
        project_session.load(source.parent().unwrap()).unwrap();
        project_session.load(&source).unwrap();
        projects::promote(&mut project_session, profile, "Shared name");
        let original = catalog(&f);
        let project = original.projects[0].clone();
        let snapshot = original.sessions[0].clone();
        let mut session = configured(&f, profile);
        create(&mut session, profile, "Shared name");
        let id = session.context().clone();
        assert_eq!(session.tree().root(), None);
        assert!(session.echo().contains("workspace-add-directory"));
        session.execute(Command::Tree(TreeCommand::Focus));
        assert_eq!(session.tree().root(), None);
        assert!(prompt(&mut session, "No roots;"));
        session.load(&loose).unwrap();
        session.insert('!');
        let stamp = session.buffer().unwrap().stamp();
        choose(&mut session, profile, "Shared name");
        ready(&mut session, |s| s.echo().contains("already active"));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        command(&mut session, profile, "workspace-add-project");
        ready(&mut session, |s| prompt(s, "Add Project directory:"));
        session.handle(Event::Paste("Shared name".into()));
        key(&mut session, KeyCode::Tab);
        key(&mut session, KeyCode::Enter);
        ready(&mut session, |s| s.echo().contains("roots published"));
        assert_eq!(session.context(), &id);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&session), "!loose");
        let after = catalog(&f);
        assert_eq!(after.projects[0], project);
        assert_eq!(after.session(&project.session_ref).unwrap(), &snapshot);
        let workspace = &after.workspaces[0];
        assert_eq!(workspace.roots.len(), 1);
        assert_eq!(
            workspace.roots[0].origin_project_id,
            Some(project.id.clone())
        );
        assert_ne!(workspace.roots[0].id, project.root.id);
        let saved = after
            .session(&workspace.session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap();
        assert_eq!(saved.files.len(), 1);
        assert!(matches!(
            saved.files[0].file,
            FileReference::LooseAbsolute(_)
        ));
        session.execute(Command::ContextClose);
        key(&mut session, KeyCode::Char('n'));
        ready(&mut session, |s| matches!(s.context(), Context::Standalone));
        choose(&mut session, profile, "Shared name");
        ready(&mut session, |s| {
            matches!(s.context(), Context::Workspace(_))
        });
        assert_eq!(text(&session), "loose");
        assert_eq!(session.buffer().unwrap().file().unwrap().path(), loose);
    }
}

// Peer roots keep stored order and route-qualified rows/hits. Overlapping paths
// have independent expansion while file activation reuses one canonical buffer.
// Actual native mouse clicks and narrow/resized rendering share those identities.
#[test]
fn peer_tree_routes() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let nested = f.write("backend/sub/file.rs", "source");
        let frontend = f.write("frontend/page.html", "page");
        let mut session = configured(&f, profile);
        create(&mut session, profile, "Peer roots");
        add(
            &mut session,
            profile,
            nested.parent().unwrap().parent().unwrap(),
        );
        add(&mut session, profile, frontend.parent().unwrap());
        add(&mut session, profile, nested.parent().unwrap());
        let roots = workspace(&f).roots;
        assert_eq!(session.tree().entries().len(), 3);
        assert_eq!(
            session
                .tree()
                .entries()
                .iter()
                .map(|e| e.root_id().unwrap().clone())
                .collect::<Vec<_>>(),
            roots.iter().map(|r| r.id.clone()).collect::<Vec<_>>()
        );
        selected_root(&mut session, &roots[0].id);
        session.execute(Command::Tree(TreeCommand::Expand));
        session.execute(Command::Tree(TreeCommand::Next));
        session.execute(Command::Tree(TreeCommand::Expand));
        session.execute(Command::Tree(TreeCommand::Next));
        session.execute(Command::Tree(TreeCommand::Open));
        assert_eq!(session.buffer().unwrap().file().unwrap().path(), nested);
        let id = session.buffer().unwrap().id();
        session.insert('!');
        let stamp = session.buffer().unwrap().stamp();
        selected_root(&mut session, &roots[2].id);
        session.execute(Command::Tree(TreeCommand::Expand));
        session.execute(Command::Tree(TreeCommand::Next));
        session.execute(Command::Tree(TreeCommand::Open));
        assert_eq!(session.buffer().unwrap().id(), id);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(
            session
                .tree()
                .entries()
                .iter()
                .filter(|e| e.path() == nested)
                .count(),
            2
        );
        let bytes = fs::read(f.0.join("state/contexts.toml")).unwrap();
        symlink(roots[0].path.as_path(), f.0.join("alias")).unwrap();
        command(&mut session, profile, "workspace-add-directory");
        session.handle(Event::Paste(f.0.join("alias").to_str().unwrap().into()));
        key(&mut session, KeyCode::Enter);
        ready(&mut session, |s| {
            s.echo().contains("already a Workspace root")
        });
        assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
        // Collapse the two expanded routes, leaving exactly three peer rows.
        selected_root(&mut session, &roots[0].id);
        session.execute(Command::Tree(TreeCommand::Collapse));
        selected_root(&mut session, &roots[2].id);
        session.execute(Command::Tree(TreeCommand::Collapse));
        session.frame().unwrap();
        let hit = session.tree().hit(1).unwrap();
        let stale = session.tree().hit(2).unwrap();
        session.handle(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        }));
        assert_eq!(
            session.tree().selected().unwrap().root_id(),
            Some(&roots[1].id)
        );
        assert!(
            session
                .tree()
                .entries()
                .iter()
                .any(|e| e.path() == frontend)
        );
        assert_eq!(session.tree().hit(1).unwrap(), hit);
        assert_ne!(session.tree().hit(2).unwrap(), stale);
        session.handle(Event::Resize(20, 7));
        session.frame().unwrap();
        session.handle(Event::Resize(100, 18));
        session.frame().unwrap();
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
    }
}

// Partial and all-unavailable membership retain both metadata and saved file
// entries. Empty/loading views never substitute cwd or manufacture source files;
// adding an available peer preserves the unavailable roots and session entries.
#[test]
fn unavailable_membership() {
    let f = fixture::Fixture::new();
    let first = f.write("first/a", "first");
    let second = f.write("second/b", "second");
    let third = f.write("third/c", "third");
    let mut session = configured(&f, "non-modal");
    create(&mut session, "non-modal", "Offline");
    add(&mut session, "non-modal", first.parent().unwrap());
    add(&mut session, "non-modal", second.parent().unwrap());
    session.load(&first).unwrap();
    session.load(&second).unwrap();
    session.execute(Command::ContextClose);
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    let saved = workspace(&f);
    fs::rename(second.parent().unwrap(), f.0.join("away-second")).unwrap();
    choose(&mut session, "non-modal", "Offline");
    ready(&mut session, |s| {
        matches!(s.context(), Context::Workspace(_))
    });
    assert_eq!(text(&session), "first");
    assert!(session.echo().contains("1 unavailable root"));
    assert!(
        session
            .tree()
            .entries()
            .iter()
            .any(|e| e.path() == second.parent().unwrap() && e.unavailable())
    );
    assert!(!second.exists());
    session.execute(Command::ContextClose);
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    assert_eq!(workspace(&f).roots, saved.roots);
    fs::rename(first.parent().unwrap(), f.0.join("away-first")).unwrap();
    choose(&mut session, "non-modal", "Offline");
    ready(&mut session, |s| {
        matches!(s.context(), Context::Workspace(_))
    });
    assert_eq!(session.context(), &Context::Workspace(saved.id.clone()));
    assert!(session.buffer().unwrap().file().is_none());
    assert!(session.echo().contains("2 unavailable root"));
    session.execute(Command::Tree(TreeCommand::Focus));
    assert_eq!(session.tree().root(), Some(first.parent().unwrap()));
    add(&mut session, "non-modal", third.parent().unwrap());
    let catalog = catalog(&f);
    let workspace = &catalog.workspaces[0];
    assert_eq!(workspace.roots.len(), 3);
    let snapshot = catalog
        .session(&workspace.session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(snapshot.files.len(), 2);
    assert!(
        snapshot
            .files
            .iter()
            .all(|e| matches!(e.file, FileReference::RootRelative { .. }))
    );
    assert!(!first.exists() && !second.exists());
    remove(&mut session, "non-modal", "2:");
    ready(&mut session, |s| s.echo().contains("roots published"));
    let after = projects::catalog(&f);
    let w = &after.workspaces[0];
    assert_eq!(
        w.roots.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        vec![saved.roots[0].id.clone(), workspace.roots[2].id.clone()]
    );
    let remaining = after
        .session(&w.session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(remaining.files.len(), 1);
    assert!(
        matches!(&remaining.files[0].file,FileReference::RootRelative{root,..} if root.as_str()==saved.roots[0].id.as_str())
    );
    assert_eq!(
        fs::read_to_string(f.0.join("away-second/b")).unwrap(),
        "second"
    );
}

// Removal remaps overlap through the remaining root without source decisions.
// Only files losing coverage then use native save/discard/cancel; dirty loose
// buffers survive removal of the last root in the same valid Workspace.
#[test]
fn scoped_removal() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let unique = f.write("root/unique", "unique");
        let shared = f.write("root/sub/shared", "shared");
        let loose = f.write("loose", "loose");
        let mut session = configured(&f, profile);
        create(&mut session, profile, "Removal");
        add(&mut session, profile, unique.parent().unwrap());
        add(&mut session, profile, shared.parent().unwrap());
        let first = workspace(&f);
        session.load(&unique).unwrap();
        session.insert('u');
        session.load(&shared).unwrap();
        session.insert('s');
        let shared_stamp = session.buffer().unwrap().stamp();
        session.load(&loose).unwrap();
        session.insert('l');
        let loose_stamp = session.buffer().unwrap().stamp();
        let generation = session.context_generation();
        remove(&mut session, profile, "2:");
        ready(&mut session, |s| s.context_generation() != generation);
        assert_eq!(session.buffer().unwrap().stamp(), loose_stamp);
        session.load(&shared).unwrap();
        assert_eq!(session.buffer().unwrap().stamp(), shared_stamp);
        let after = catalog(&f);
        let w = &after.workspaces[0];
        let snapshot = after
            .session(&w.session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap();
        assert!(snapshot.files.iter().any(|e|matches!(&e.file,FileReference::RootRelative{root,path} if *root==first.roots[0].id && path.as_path()==Path::new("sub/shared"))));
        let bytes = fs::read(f.0.join("state/contexts.toml")).unwrap();
        remove(&mut session, profile, "1:");
        assert!(session.echo().contains("Remove root:"));
        cancel(&mut session);
        assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
        assert_eq!(session.buffer().unwrap().stamp(), shared_stamp);
        session.load(&loose).unwrap();
        remove(&mut session, profile, "1:");
        key(&mut session, KeyCode::Char('y'));
        assert!(session.echo().contains("Remove root:"));
        key(&mut session, KeyCode::Char('n'));
        ready(&mut session, workspace_state_empty);
        assert_eq!(session.context(), &Context::Workspace(first.id));
        assert_eq!(session.buffer().unwrap().stamp(), loose_stamp);
        assert_eq!(text(&session), "lloose");
        assert_eq!(session.tree().root(), None);
        session.execute(Command::Tree(TreeCommand::Focus));
        assert_eq!(session.tree().root(), None);
        assert_eq!(fs::read_to_string(unique).unwrap(), "uunique");
        assert_eq!(fs::read_to_string(shared).unwrap(), "shared");
        assert!(workspace(&f).roots.is_empty());
        let after = catalog(&f);
        let w = &after.workspaces[0];
        assert_eq!(
            after
                .session(&w.session_ref)
                .unwrap()
                .snapshot
                .as_ref()
                .unwrap()
                .files
                .len(),
            1
        );
    }
}
fn workspace_state_empty(session: &mut Session) -> bool {
    session.echo().contains("roots published") && session.tree().entries().is_empty()
}

// Pre-commit publication failure retains root membership, dirty source owners
// and the exact catalog. Retrying after the owned lock is released closes only
// the approved sources; competing Workspace checkpoints remain conflicts.
#[test]
fn membership_publication() {
    let f = fixture::Fixture::new();
    let source = f.write("root/file", "source");
    let mut session = configured(&f, "non-modal");
    create(&mut session, "non-modal", "Conflict");
    add(&mut session, "non-modal", source.parent().unwrap());
    session.load(&source).unwrap();
    session.insert('!');
    let before = catalog(&f);
    let bytes = fs::read(f.0.join("state/contexts.toml")).unwrap();
    let stamp = session.buffer().unwrap().stamp();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.0.join("state/contexts.toml.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    remove(&mut session, "non-modal", "1:");
    key(&mut session, KeyCode::Char('n'));
    ready(&mut session, |s| s.echo().contains("busy"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), bytes);
    assert_eq!(workspace(&f), before.workspaces[0]);
    drop(lock);
    let mut competing = configured(&f, "non-modal");
    choose(&mut competing, "non-modal", "Conflict");
    ready(&mut competing, |s| {
        matches!(s.context(), Context::Workspace(_))
    });
    competing.execute(Command::ContextClose);
    ready(&mut competing, |s| {
        matches!(s.context(), Context::Standalone)
    });
    let newer = fs::read(f.0.join("state/contexts.toml")).unwrap();
    remove(&mut session, "non-modal", "1:");
    key(&mut session, KeyCode::Char('n'));
    ready(&mut session, |s| s.echo().contains("stale catalog"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(fs::read(f.0.join("state/contexts.toml")).unwrap(), newer);
    assert_eq!(workspace(&f).roots, before.workspaces[0].roots);
}

// An active clock is consulted only when its source loses coverage. Cancellation
// retains that source; keep/save uses the same checked close continuation before
// the root disappears, and never serializes the clock into the Workspace.
#[test]
fn root_clock_departure() {
    use rev::command::OrgCommand;
    use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
    use std::time::{Duration, UNIX_EPOCH};
    fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
        ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
            Ok(UNIX_EPOCH + Duration::from_secs(1709303400))
        })
    }
    let f = fixture::Fixture::new();
    let source = f.write("root/clock.org", "* Task\nbody");
    let other = f.write("other/file", "other");
    let mut session = configured(&f, "non-modal").with_org_clock(clock);
    create(&mut session, "non-modal", "Clock roots");
    add(&mut session, "non-modal", source.parent().unwrap());
    add(&mut session, "non-modal", other.parent().unwrap());
    session.load(&source).unwrap();
    session.execute(Command::Org(OrgCommand::ClockIn));
    ready(&mut session, |s| text(s).contains("CLOCK:"));
    let clock_source = text(&session);
    let generation = session.context_generation();
    remove(&mut session, "non-modal", "2:");
    ready(&mut session, |s| s.context_generation() != generation);
    assert_eq!(text(&session), clock_source);
    remove(&mut session, "non-modal", "1:");
    assert!(session.echo().contains("detach"));
    cancel(&mut session);
    assert_eq!(text(&session), clock_source);
    assert_eq!(workspace(&f).roots.len(), 1);
    remove(&mut session, "non-modal", "1:");
    session.handle(Event::Paste("keep".into()));
    key(&mut session, KeyCode::Enter);
    assert!(session.echo().contains("Remove root:"));
    key(&mut session, KeyCode::Char('y'));
    ready(&mut session, workspace_state_empty);
    assert_eq!(fs::read_to_string(source).unwrap(), clock_source);
    assert!(workspace(&f).roots.is_empty());
}

// The final-source fallback is admitted before publication. An unrelated live
// terminal survives root removal and is reaped only by the later explicit
// context-departure confirmation, through the existing terminal owner.
#[test]
fn terminal_and_final_source() {
    use rev::command::TerminalCommand;
    use std::time::{Duration, Instant};
    let f = fixture::Fixture::new();
    let source = f.write("root/source", "source");
    let script = f.write("terminal.sh", "printf READY\nwhile :; do sleep 1; done\n");
    let settings = prepare(&f.write(
        "terminal.toml",
        &format!(
            "terminal.program='/bin/sh'\nterminal.args=[{}]",
            toml_edit::Value::from(script.to_str().unwrap())
        ),
    ));
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
    session
        .set_context_storage(f.0.join("state/contexts.toml"))
        .unwrap();
    create(&mut session, "non-modal", "Terminal roots");
    add(&mut session, "non-modal", source.parent().unwrap());
    session.load(&source).unwrap();
    session.execute(Command::Buffer(BufferCommand::Kill));
    session.handle(Event::Paste("*scratch*".into()));
    key(&mut session, KeyCode::Enter);
    session.execute(Command::Terminal(TerminalCommand::NewBuffer));
    let deadline = Instant::now() + Duration::from_secs(3);
    let pid = loop {
        if let Some(pid) = session
            .terminals()
            .ids()
            .find_map(|id| session.terminals().get(id).unwrap().process_id())
        {
            break pid;
        }
        assert!(Instant::now() < deadline, "{}", session.echo());
        session.expire_input(Instant::now());
        std::thread::yield_now();
    };
    remove(&mut session, "non-modal", "1:");
    ready(&mut session, workspace_state_empty);
    assert!(Path::new("/proc").join(pid.to_string()).exists());
    assert_eq!(session.terminals().ids().count(), 1);
    session.execute(Command::ContextClose);
    assert!(session.echo().contains("terminate"));
    key(&mut session, KeyCode::Char('y'));
    ready(&mut session, |s| matches!(s.context(), Context::Standalone));
    assert!(!Path::new("/proc").join(pid.to_string()).exists());
    assert_eq!(text(&session), "");
}

// Workspace peer rows use the existing decorated/minimal and Nerd Fonts policies
// while remaining lazy. Duplicate basenames keep independent route identities;
// drawing their cached labels cannot expand either root or load source buffers.
#[test]
fn peer_presentations() {
    for (decorated, nerd) in [(true, false), (true, true), (false, false)] {
        let f = fixture::Fixture::new();
        let left = f.write("left/shared/source.rs", "source");
        let right = f.write("right/shared/note.md", "note");
        let settings = prepare(&f.write(
            "tree.toml",
            &format!("tree.decorated={decorated}\ngutter.nerd_fonts={nerd}"),
        ));
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
        session
            .set_context_storage(f.0.join("state/contexts.toml"))
            .unwrap();
        create(&mut session, "non-modal", "Presentation");
        add(&mut session, "non-modal", left.parent().unwrap());
        add(&mut session, "non-modal", right.parent().unwrap());
        let frame = session.frame().unwrap();
        let rows: Vec<_> = frame
            .rows()
            .iter()
            .filter(|r| r.text().contains("shared"))
            .collect();
        assert_eq!(rows.len(), 2);
        for row in rows {
            if decorated {
                assert!(row.text().contains(if nerd { "\u{f1b2}" } else { "◆" }));
            } else {
                assert!(row.text().starts_with("> shared"));
                assert!(!row.text().contains("◆"));
            }
        }
        assert_eq!(session.tree().entries().len(), 2);
        assert!(session.tree().entries().iter().all(|e| !e.expanded()));
        assert_ne!(session.tree().hit(0), session.tree().hit(1));
        assert!(session.buffer().unwrap().file().is_none());
    }
}
