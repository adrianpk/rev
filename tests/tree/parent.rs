use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;
use std::time::{Duration, Instant};

use rev::buffer::Buffers;
use rev::command::{RegionCommand, Split};
use rev::context::Context;
use rev::tree::Tree;
use rev::view::View;

use super::*;

fn configured(f: &config_fixture::Fixture, config: &str, file: &Path) -> Session {
    let startup = rev::config::prepare(&f.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(file).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
}

fn entries(session: &Session) -> String {
    format!("{:?}", session.tree().entries())
}

fn rows(session: &mut Session) -> Vec<String> {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|row| row.band() == rev::display::Band::Document)
        .map(|row| row.text().to_owned())
        .collect()
}

fn ready(session: &mut Session, mut condition: impl FnMut(&mut Session) -> bool) {
    let until = Instant::now() + Duration::from_secs(5);
    while !condition(session) {
        assert!(Instant::now() < until, "{}", session.echo());
        session.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(1));
    }
}

// Each upward transition selects the directory just left, reveals an offscreen
// result and keeps automatic file following and independent dirty views intact.
#[test]
fn successive_parents() {
    for policy in ["auto", "file-parent"] {
        let f = config_fixture::Fixture::new();
        let file = f.write("parent/zz-child/source.txt", "retained");
        let later = f.write("later/other.txt", "later");
        for i in 0..24 {
            fs::create_dir(f.0.join(format!("parent/aa-{i:02}"))).unwrap();
        }
        let mut session = configured(&f, &format!("tree.root='{policy}'"), &file);
        session.insert('!');
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::Split(Split::Right));
        tree(&mut session, TreeCommand::Focus);
        session.frame().unwrap();
        let views = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>();
        let stamp = session.buffer().unwrap().stamp();
        let history = session.buffer().unwrap().history().len();
        let mark = session.buffer().unwrap().mark();
        let generation = session.context_generation();
        let cwd = std::env::current_dir().unwrap();
        for previous in [file.parent().unwrap(), f.0.join("parent").as_path()] {
            tree(&mut session, TreeCommand::UpDirectory);
            assert_eq!(session.tree().root(), previous.parent());
            let selected = session.tree().selected().unwrap();
            assert_eq!(selected.path(), previous);
            assert_eq!(selected.depth(), 1);
            assert!(!selected.expanded());
            let frame = session.frame().unwrap();
            let cursor = frame.cursor().unwrap();
            let row = &frame.rows()[usize::from(cursor.row)];
            assert!(
                row.text()
                    .contains(previous.file_name().unwrap().to_str().unwrap())
            );
            assert!(row.selection().is_some());
            assert!(session.tree().focused());
            assert_eq!(session.context(), &Context::Standalone);
            assert_eq!(session.context_generation(), generation);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().history().len(), history);
            assert_eq!(session.buffer().unwrap().mark(), mark);
            assert!(session.buffer().unwrap().is_modified());
            assert_eq!(text(&session), "!retained");
            assert_eq!(
                session
                    .windows()
                    .iter()
                    .map(|(id, view)| (id, view.clone()))
                    .collect::<Vec<_>>(),
                views
            );
        }
        tree(&mut session, TreeCommand::Open);
        assert!(session.tree().selected().unwrap().expanded());
        assert_eq!(std::env::current_dir().unwrap(), cwd);
        assert_eq!(fs::read_to_string(&file).unwrap(), "retained");
        assert!(!f.0.join("state").exists());
        session.load(&later).unwrap();
        assert_eq!(session.tree().root(), later.parent());
        session.load(&file).unwrap();
        assert_eq!(session.tree().root(), file.parent());
        assert_eq!(text(&session), "!retained");
    }
}

// A fixed boundary refuses before observing its parent. All four context
// variants preserve their membership, loaded rows, selected route and storage.
#[test]
fn fixed_roots() {
    for kind in ["directory", "project", "workspace", "cwd"] {
        let f = config_fixture::Fixture::new();
        let file = f.write("root/source.txt", "retained");
        let mut session = configured(
            &f,
            &format!("tree.root='{}'", if kind == "cwd" { "cwd" } else { "auto" }),
            &file,
        );
        let storage = f.0.join("state/contexts.toml");
        session.set_context_storage(storage.clone()).unwrap();
        if matches!(kind, "directory" | "project") {
            session.load(file.parent().unwrap()).unwrap();
        }
        if kind == "project" {
            named(&mut session, "project-promote");
            ready(&mut session, |s| {
                s.frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .any(|r| r.text().contains("Project name:"))
            });
            session.handle(key(KeyCode::End));
            for _ in 0..64 {
                session.handle(key(KeyCode::Backspace));
            }
            session.handle(Event::Paste("Tree project".into()));
            session.handle(key(KeyCode::Enter));
            ready(&mut session, |s| matches!(s.context(), Context::Project(_)));
        }
        if kind == "workspace" {
            named(&mut session, "workspace-create");
            ready(&mut session, |s| {
                s.frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .any(|r| r.text().contains("Workspace name:"))
            });
            session.handle(Event::Paste("Tree workspace".into()));
            session.handle(key(KeyCode::Enter));
            ready(&mut session, |s| {
                matches!(s.context(), Context::Workspace(_))
            });
            named(&mut session, "workspace-add-directory");
            session.handle(Event::Paste(
                file.parent().unwrap().to_str().unwrap().into(),
            ));
            session.handle(key(KeyCode::Enter));
            ready(&mut session, |s| !s.tree().entries().is_empty());
        }
        tree(&mut session, TreeCommand::Focus);
        session.frame().unwrap();
        let root = session.tree().root().unwrap().to_owned();
        assert!(
            session
                .tree()
                .entries()
                .iter()
                .all(|entry| !entry.is_parent())
        );
        let before = entries(&session);
        let selected = session.tree().selected().unwrap().path().to_owned();
        let context = session.context().clone();
        let generation = session.context_generation();
        let stored = fs::read(&storage).ok();
        let boundary = root.parent().unwrap();
        let permissions = fs::metadata(boundary).unwrap().permissions();
        if kind != "cwd" {
            fs::set_permissions(boundary, fs::Permissions::from_mode(0o000)).unwrap();
        }
        tree(&mut session, TreeCommand::UpDirectory);
        if kind != "cwd" {
            fs::set_permissions(boundary, permissions).unwrap();
        }
        assert!(
            session.echo().contains("root is fixed"),
            "{kind}: {}",
            session.echo()
        );
        assert_eq!(session.tree().root(), Some(root.as_path()));
        assert_eq!(entries(&session), before);
        assert_eq!(session.tree().selected().unwrap().path(), selected);
        assert_eq!(session.context(), &context);
        assert_eq!(session.context_generation(), generation);
        assert_eq!(fs::read(&storage).ok(), stored);
        assert!(session.tree().focused());
    }
}

// Failed observations preserve loaded expansion, selection, scroll and focus.
// Missing, unreadable, filtered and replaced paths cannot publish a substitute.
#[test]
fn rejected_parent() {
    for reason in [
        "hidden",
        "missing-child",
        "missing-parent",
        "denied",
        "parent-link",
        "child-link",
    ] {
        let f = config_fixture::Fixture::new();
        let child = if reason == "hidden" {
            ".child"
        } else {
            "child"
        };
        let file = f.write(&format!("parent/{child}/source.txt"), "retained");
        for i in 0..20 {
            f.write(&format!("parent/{child}/file-{i:02}"), "");
        }
        let mut session = configured(&f, "tree.show_hidden=false", &file);
        session.insert('!');
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::Split(Split::Right));
        tree(&mut session, TreeCommand::Focus);
        for _ in 0..16 {
            tree(&mut session, TreeCommand::Next);
        }
        let before_rows = rows(&mut session);
        let before = entries(&session);
        let root = session.tree().root().unwrap().to_owned();
        let selected = session.tree().selected().unwrap().path().to_owned();
        let view = session.view().clone();
        let views = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>();
        let stamp = session.buffer().unwrap().stamp();
        let mark = session.buffer().unwrap().mark();
        let history = session.buffer().unwrap().history().len();
        let parent = f.0.join("parent");
        let moved = f.0.join("moved");
        match reason {
            "missing-child" | "child-link" => fs::rename(&root, &moved).unwrap(),
            "missing-parent" | "parent-link" => fs::rename(&parent, &moved).unwrap(),
            "denied" => fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).unwrap(),
            _ => {}
        }
        if reason == "parent-link" {
            symlink(&moved, &parent).unwrap();
        }
        if reason == "child-link" {
            symlink(&moved, &root).unwrap();
        }
        tree(&mut session, TreeCommand::UpDirectory);
        if reason == "denied" {
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(
            session.echo().contains(
                if matches!(reason, "hidden" | "missing-child" | "child-link") {
                    "cannot represent"
                } else if reason == "denied" {
                    "Permission denied"
                } else if reason == "missing-parent" {
                    "No such file or directory"
                } else {
                    "not a directory"
                }
            ),
            "{reason}: {}",
            session.echo()
        );
        assert_eq!(session.tree().root(), Some(root.as_path()));
        assert_eq!(entries(&session), before);
        assert_eq!(session.tree().selected().unwrap().path(), selected);
        assert_eq!(rows(&mut session), before_rows);
        assert_eq!(session.view(), &view);
        assert!(session.tree().focused());
        assert_eq!(text(&session), "!retained");
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            views
        );
    }
}

// The ordinary bound admits an incomplete parent only if it contains the return
// directory. Pick both routes from the real retained read order, not name order.
#[test]
fn bounded_parent() {
    let f = config_fixture::Fixture::new();
    let parent = f.0.join("parent");
    fs::create_dir(&parent).unwrap();
    for i in 0..=rev::file::completion::ENTRY_LIMIT {
        fs::create_dir(parent.join(format!("dir-{i:04}"))).unwrap();
    }
    let observed = fs::read_dir(&parent)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    for (path, admitted) in [(&observed[0], true), (observed.last().unwrap(), false)] {
        let mut tree = Tree::new(path.clone());
        tree.expand().unwrap();
        let result = tree.up_directory();
        assert_eq!(result.is_ok(), admitted);
        assert_eq!(
            tree.root(),
            Some(if admitted {
                parent.as_path()
            } else {
                path.as_path()
            })
        );
        assert_eq!(tree.selected().unwrap().path(), path);
        if admitted {
            assert_eq!(tree.entries().len(), rev::file::completion::ENTRY_LIMIT + 2);
            assert!(
                tree.diagnostic()
                    .unwrap()
                    .contains("Incomplete tree listing")
            );
        } else {
            assert_eq!(tree.entries().len(), 2);
            assert!(result.unwrap_err().to_string().contains("cannot represent"));
        }
    }
}

// Filesystem root has no parent; it does not enumerate or replace retained rows.
#[test]
fn filesystem_root() {
    let mut tree = Tree::new("/".into());
    assert!(
        tree.up_directory()
            .unwrap_err()
            .to_string()
            .contains("no parent")
    );
    assert_eq!(tree.root(), Some(Path::new("/")));
    assert_eq!(tree.entries().len(), 1);
    assert!(!tree.entries()[0].expanded());
}

// The parent row routes keyboard and pointer activation through the same checked
// transition; selection-only clicks leave source and root intact until Enter.
#[test]
fn parent_entry() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

    for (profile, route, decorated, nerd_fonts) in [
        ("non-modal", "enter", true, true),
        ("modal", "enter", true, false),
        ("non-modal", "click", true, true),
        ("non-modal", "select-only", false, true),
    ] {
        let f = config_fixture::Fixture::new();
        let file = f.write("parent/child/source.txt", "retained");
        let config = format!(
            "editor.interaction_profile='{profile}'\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}\nmouse.tree_activate_on_click={}",
            route != "select-only"
        );
        let mut session = configured(&f, &config, &file);
        session.insert('!');
        let stamp = session.buffer().unwrap().stamp();
        let history = session.buffer().unwrap().history().len();
        tree(&mut session, TreeCommand::Focus);
        if route == "enter" {
            tree(&mut session, TreeCommand::Collapse);
        }
        let frame = session.frame().unwrap();
        let row = &frame.rows()[1];
        assert!(
            row.text()
                .split('│')
                .next()
                .unwrap()
                .trim_end()
                .ends_with("..")
        );
        if route == "enter" {
            tree(&mut session, TreeCommand::Next);
        } else {
            session.handle(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 1,
                row: row.row(),
                modifiers: KeyModifiers::NONE,
            }));
        }
        if route != "click" {
            assert_eq!(session.tree().root(), file.parent());
            tree(&mut session, TreeCommand::Refresh);
            session.handle(key(KeyCode::Enter));
        }
        assert_eq!(session.tree().root(), Some(f.0.join("parent").as_path()));
        assert_eq!(
            session.tree().selected().unwrap().path(),
            file.parent().unwrap()
        );
        assert!(session.tree().focused());
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert_eq!(text(&session), "!retained");
    }
}

// Navigation never grants filesystem mutation authority. Refresh and collapse
// preserve the parent row, while a retained hit cannot target a different root.
#[test]
fn parent_entry_identity() {
    let f = config_fixture::Fixture::new();
    let file = f.write("parent/child/source.txt", "retained");
    let mut tree = Tree::new(file.parent().unwrap().to_owned());
    tree.expand().unwrap();
    let hit = tree.hit(1).unwrap();
    assert!(tree.select_hit(&hit));
    assert!(tree.selected().unwrap().is_parent());
    for kind in [
        rev::tree::CreateKind::File,
        rev::tree::CreateKind::Directory,
    ] {
        assert!(
            tree.prepare_create(kind)
                .err()
                .unwrap()
                .to_string()
                .contains("navigation only")
        );
    }
    assert!(
        tree.prepare_delete()
            .err()
            .unwrap()
            .to_string()
            .contains("navigation only")
    );
    assert!(
        tree.prepare_rename()
            .err()
            .unwrap()
            .to_string()
            .contains("navigation only")
    );
    tree.refresh().unwrap();
    assert!(tree.selected().unwrap().is_parent());
    tree.activate().unwrap();
    assert_eq!(tree.root(), Some(f.0.join("parent").as_path()));
    assert!(!tree.select_hit(&hit));
    assert_eq!(fs::read_to_string(file).unwrap(), "retained");
}
