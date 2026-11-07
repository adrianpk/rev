use std::path::{Path, PathBuf};

use rev::buffer::Buffers;
use rev::command::{BufferCommand, Motion, Split, WindowCommand};
use rev::view::View;

use super::*;

fn configured(f: &config_fixture::Fixture, config: &str, path: &Path) -> Session {
    let startup = rev::config::prepare(&f.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(path).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    session.load(path.parent().unwrap()).unwrap();
    session.load(path).unwrap();
    session
}

fn choose(session: &mut Session, path: &Path) {
    tree(session, TreeCommand::Focus);
    let target = session
        .tree()
        .entries()
        .iter()
        .position(|e| e.path() == path)
        .unwrap();
    for _ in 0..session.tree().entries().len() {
        let current = session
            .tree()
            .entries()
            .iter()
            .position(|e| e.path() == session.tree().selected().unwrap().path())
            .unwrap();
        if current == target {
            return;
        }
        tree(
            session,
            if current < target {
                TreeCommand::Next
            } else {
                TreeCommand::Previous
            },
        );
    }
    panic!("loaded entry not reachable");
}

fn highlights(session: &mut Session) -> Vec<String> {
    let frame = session.frame().unwrap();
    frame
        .rows()
        .iter()
        .filter(|row| row.selection() == Some(0..24))
        .map(|row| row.text().split('│').next().unwrap().to_owned())
        .collect()
}

#[track_caller]
fn indicated(session: &mut Session, name: Option<&str>) {
    let rows = highlights(session);
    match name {
        Some(name) => {
            assert_eq!(rows.len(), 1, "{rows:?}");
            assert!(rows[0].contains(name), "{rows:?} should mark {name}");
        }
        None => assert!(rows.is_empty(), "{rows:?}"),
    }
}

fn switch(session: &mut Session, name: &str) {
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}

// Both input profiles preserve a separate browsing cursor and one persistent
// highlight. Prompt focus retains its tree/editing origin with either setting.
#[test]
fn focus_policy() {
    for (profile, decorated, follow) in [
        ("non-modal", true, true),
        ("modal", false, true),
        ("non-modal", false, false),
        ("modal", true, false),
    ] {
        let f = config_fixture::Fixture::new();
        let a = f.write("root/alpha.txt", "alpha\nsecond");
        let b = f.write("root/bravo.txt", "bravo");
        let mut session = configured(
            &f,
            &format!(
                "editor.interaction_profile='{profile}'\ntree.visible=true\ntree.decorated={decorated}\ntree.follow_active_buffer={follow}"
            ),
            &a,
        );
        session.handle(Event::Resize(100, 24));
        choose(&mut session, &b);
        let point = session.view().point;
        let history = session.buffer().unwrap().history().len();
        indicated(&mut session, Some("bravo.txt"));
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Paste("tree".into()));
        indicated(&mut session, Some("bravo.txt"));
        session.execute(Command::KeyboardQuit);
        assert!(session.tree().focused());
        session.handle(key(KeyCode::Tab));
        indicated(
            &mut session,
            Some(if follow { "alpha.txt" } else { "bravo.txt" }),
        );
        session.execute(Command::ExecuteExtended);
        indicated(
            &mut session,
            Some(if follow { "alpha.txt" } else { "bravo.txt" }),
        );
        session.execute(Command::KeyboardQuit);
        assert!(!session.tree().focused());
        assert_eq!(session.view().point, point);
        assert_eq!(text(&session), "alpha\nsecond");
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(std::fs::read(&a).unwrap(), b"alpha\nsecond");
        tree(&mut session, TreeCommand::Focus);
        assert_eq!(session.tree().selected().unwrap().path(), b);
        session.handle(key(KeyCode::Tab));
        session.load(&b).unwrap();
        switch(&mut session, "alpha.txt");
        indicated(
            &mut session,
            Some(if follow { "alpha.txt" } else { "bravo.txt" }),
        );
        switch(&mut session, "*scratch*");
        indicated(&mut session, if follow { None } else { Some("bravo.txt") });
        tree(&mut session, TreeCommand::Focus);
        assert_eq!(session.tree().selected().unwrap().path(), b);
    }
}

// Pane following changes only presentation. Dirty bytes, point, mark, history
// and the other view's source position survive focus, listing and resize events.
#[test]
fn panes_and_source() {
    let f = config_fixture::Fixture::new();
    let a = f.write("root/alpha.txt", "alpha\nsecond");
    let b = f.write("root/bravo.txt", "bravo\nsecond");
    let mut session = configured(&f, "tree.visible=true", &a);
    session.insert('!');
    session.execute(Command::Region(rev::command::RegionCommand::Set));
    session.execute(Command::Motion(Motion::ForwardChar));
    let first = session.windows().selected_id();
    let a_point = session.view().point;
    let a_mark = session.buffer().unwrap().mark();
    let history = session.buffer().unwrap().history().len();
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    session.load(&b).unwrap();
    let second = session.windows().selected_id();
    let b_point = session.view().point;
    indicated(&mut session, Some("bravo.txt"));
    session.select_view(first).unwrap();
    indicated(&mut session, Some("alpha.txt"));
    assert_eq!(session.view().point, a_point);
    assert_eq!(session.buffer().unwrap().mark(), a_mark);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(text(&session), "!alpha\nsecond");
    session.select_view(second).unwrap();
    assert_eq!(session.view().point, b_point);
    choose(&mut session, &a);
    tree(&mut session, TreeCommand::Refresh);
    session.handle(key(KeyCode::Tab));
    session.handle(Event::Resize(31, 12));
    assert!(highlights(&mut session).is_empty());
    session.handle(Event::Resize(100, 12));
    indicated(&mut session, Some("bravo.txt"));
    assert_eq!(session.tree().selected().unwrap().path(), a);
    assert_eq!(std::fs::read(&a).unwrap(), b"alpha\nsecond");
    assert_eq!(std::fs::read(&b).unwrap(), b"bravo\nsecond");
}

// Following reveals only loaded rows. Manual scrolling survives frames and
// point movement; another real pane/file transition can reveal its target.
#[test]
fn reveal_transitions() {
    use crossterm::event::{MouseEvent, MouseEventKind};
    let f = config_fixture::Fixture::new();
    let a = f.write("root/00-first.txt", "first\nsecond");
    for index in 1..25 {
        f.write(&format!("root/{index:02}-file.txt"), "retained");
    }
    let last = f.write("root/zz-last.txt", "last\nsecond");
    let mut session = configured(&f, "tree.visible=true", &a);
    session.load(&last).unwrap();
    indicated(&mut session, Some("zz-last.txt"));
    session.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 2,
        row: 1,
        modifiers: KeyModifiers::NONE,
    }));
    let top = session.tree().hit(0).unwrap();
    let point = session.view().point;
    for _ in 0..3 {
        session.frame().unwrap();
        assert_eq!(session.tree().hit(0), Some(top.clone()));
    }
    session.execute(Command::Motion(Motion::ForwardChar));
    session.frame().unwrap();
    assert_eq!(session.tree().hit(0), Some(top));
    assert_ne!(session.view().point, point);
    switch(&mut session, "00-first.txt");
    indicated(&mut session, Some("00-first.txt"));
    switch(&mut session, "zz-last.txt");
    indicated(&mut session, Some("zz-last.txt"));
    // A prompt can borrow rows before the offscreen browsing cursor is revealed.
    // Its transient cursor must not subtract the newer active-file viewport.
    let browsing = session.tree().selected().unwrap().path().to_path_buf();
    tree(&mut session, TreeCommand::Focus);
    session.execute(Command::ExecuteExtended);
    session.frame().unwrap();
    assert_eq!(session.tree().selected().unwrap().path(), browsing);
    session.execute(Command::KeyboardQuit);
    session.frame().unwrap();
}

// Collapsed, filtered, outside and missing sources never impersonate another
// row. Explicit expansion/refresh restores only reliable loaded observations.
#[test]
fn unavailable_targets() {
    let f = config_fixture::Fixture::new();
    let a = f.write("root/alpha.txt", "alpha");
    let nested = f.write("root/folder/nested.txt", "nested");
    let hidden = f.write("root/.hidden", "hidden");
    let outside = f.write("outside/other.txt", "outside");
    let mut session = configured(&f, "tree.visible=true\ntree.show_hidden=false", &a);
    let root = session.tree().root().unwrap().to_path_buf();
    session.load(&nested).unwrap();
    indicated(&mut session, None);
    choose(&mut session, nested.parent().unwrap());
    tree(&mut session, TreeCommand::Expand);
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, Some("nested.txt"));
    choose(&mut session, nested.parent().unwrap());
    tree(&mut session, TreeCommand::Collapse);
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, None);
    assert!(!session.tree().selected().unwrap().expanded());
    for path in [&hidden, &outside] {
        session.load(path).unwrap();
        indicated(&mut session, None);
        assert_eq!(session.tree().root(), Some(root.as_path()));
    }
    session.load(&a).unwrap();
    indicated(&mut session, Some("alpha.txt"));
    std::fs::remove_file(&a).unwrap();
    tree(&mut session, TreeCommand::Focus);
    tree(&mut session, TreeCommand::Refresh);
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, None);
    assert_eq!(text(&session), "alpha");
    f.write("root/alpha.txt", "alpha");
    tree(&mut session, TreeCommand::Focus);
    tree(&mut session, TreeCommand::Refresh);
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, Some("alpha.txt"));
}

// A leaf alias requires successful admitted ingress. Merely sharing a name
// cannot match, and a listing refresh retires the old alias association.
#[test]
fn admitted_alias() {
    let f = config_fixture::Fixture::new();
    let source = f.write("outside/source.txt", "source");
    let first = f.write("root/first.txt", "first");
    let alias: PathBuf = f.0.join("root/alias.txt");
    std::os::unix::fs::symlink(&source, &alias).unwrap();
    let mut session = configured(&f, "tree.visible=true", &first);
    session.load(&source).unwrap();
    indicated(&mut session, None);
    choose(&mut session, &alias);
    tree(&mut session, TreeCommand::Open);
    let id = session.buffer().unwrap().id();
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), source);
    indicated(&mut session, Some("alias.txt"));
    tree(&mut session, TreeCommand::Focus);
    tree(&mut session, TreeCommand::Refresh);
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, None);
    choose(&mut session, &alias);
    tree(&mut session, TreeCommand::Open);
    assert_eq!(session.buffer().unwrap().id(), id);
    indicated(&mut session, Some("alias.txt"));
}

// Successful source relocation refreshes the visiting identity; failed rename
// leaves the prior source and highlight available without new text history.
#[test]
fn source_rebinding() {
    let f = config_fixture::Fixture::new();
    let a = f.write("root/alpha.txt", "alpha");
    let b = f.0.join("root/bravo.txt");
    let mut session = configured(&f, "tree.visible=true", &a);
    let id = session.buffer().unwrap().id();
    session.execute(Command::WriteFile);
    session.handle(Event::Paste(b.to_str().unwrap().into()));
    session.handle(key(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), b);
    indicated(&mut session, Some("bravo.txt"));
    choose(&mut session, &b);
    tree(&mut session, TreeCommand::Rename);
    session.handle(Event::Paste("alpha.txt".into()));
    session.handle(key(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), b);
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, Some("bravo.txt"));
    choose(&mut session, &b);
    tree(&mut session, TreeCommand::Rename);
    session.handle(Event::Paste("charlie.txt".into()));
    session.handle(key(KeyCode::Enter));
    session.handle(key(KeyCode::Tab));
    indicated(&mut session, Some("charlie.txt"));
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(text(&session), "alpha");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(std::fs::read(a).unwrap(), b"alpha");
    assert_eq!(
        std::fs::read(f.0.join("root/charlie.txt")).unwrap(),
        b"alpha"
    );
}
