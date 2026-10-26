use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rev::tree::Tree;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rev-navigation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn directory(&self, name: &str) {
        fs::create_dir_all(self.path(name)).unwrap();
    }
    fn file(&self, name: &str) {
        fs::write(self.path(name), b"retained").unwrap();
    }
    fn tree(&self) -> Tree {
        Tree::new(self.0.clone())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn selected(tree: &Tree) -> &Path {
    tree.selected().unwrap().path()
}
fn choose(tree: &mut Tree, path: &Path) {
    let index = tree
        .entries()
        .iter()
        .position(|entry| entry.path() == path)
        .unwrap();
    while selected(tree) != path {
        tree.move_selection(
            tree.entries()
                .iter()
                .position(|entry| entry.path() == selected(tree))
                .unwrap()
                < index,
        );
    }
}

// Directory-first ordering is independent of creation order. Symlinks remain
// leaves, including links to directories; no child content is needed to sort.
#[test]
fn ordering() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    fixture.file("z.txt");
    fixture.directory("b");
    fixture.file(".hidden");
    fixture.directory("a");
    fixture.file("A.txt");
    fixture.file("界.txt");
    symlink(fixture.path("a"), fixture.path("link")).unwrap();
    let mut tree = fixture.tree();
    assert_eq!(tree.entries().len(), 1);
    tree.expand().unwrap();
    let names: Vec<_> = tree.entries()[1..]
        .iter()
        .map(|entry| entry.path().file_name().unwrap().to_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["a", "b", ".hidden", "A.txt", "link", "z.txt", "界.txt"]
    );
    let link = tree
        .entries()
        .iter()
        .find(|entry| entry.path() == fixture.path("link"))
        .unwrap();
    assert!(link.is_symlink());
    assert!(!link.is_directory());
    assert!(tree.entries().iter().all(|entry| entry.depth() <= 1));
}

// Movement visits only visible preorder and clamps at endpoints. Expansion is
// explicit, repeated Right is idempotent, and Left collapses or selects parent.
#[test]
fn visible_walk() {
    let fixture = Fixture::new();
    fixture.directory("a/deep");
    fixture.file("a/deep/item");
    fixture.file("last");
    let mut tree = fixture.tree();
    tree.move_selection(true);
    assert_eq!(tree.entries().len(), 1);
    tree.expand().unwrap();
    tree.move_selection(true);
    tree.expand().unwrap();
    assert_eq!(tree.entries().len(), 4);
    tree.expand().unwrap();
    assert_eq!(tree.entries().len(), 4);
    tree.move_selection(true);
    assert_eq!(selected(&tree), fixture.path("a/deep"));
    tree.expand().unwrap();
    tree.move_selection(true);
    assert_eq!(selected(&tree), fixture.path("a/deep/item"));
    tree.collapse();
    assert_eq!(selected(&tree), fixture.path("a/deep"));
    tree.collapse();
    assert_eq!(tree.entries().len(), 4);
    tree.collapse();
    assert_eq!(selected(&tree), fixture.path("a"));
    tree.collapse();
    assert_eq!(tree.entries().len(), 3);
    tree.move_selection(true);
    tree.move_selection(true);
    assert_eq!(selected(&tree), fixture.path("last"));
    tree.collapse();
    tree.collapse();
    tree.move_selection(false);
    assert_eq!(selected(&tree), fixture.0);
    assert_eq!(tree.entries().len(), 1);
}

// Refresh reloads open branches, leaves unopened subtrees lazy and retains
// selection by path. A removed selection falls back to its nearest live parent.
#[test]
fn refreshed_selection() {
    let fixture = Fixture::new();
    fixture.directory("a/closed");
    fixture.file("a/keep");
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("a/keep"));
    fixture.file("a/first");
    fixture.file("a/closed/unobserved");
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), fixture.path("a/keep"));
    assert!(
        tree.entries()
            .iter()
            .any(|entry| entry.path() == fixture.path("a/first"))
    );
    assert!(
        !tree
            .entries()
            .iter()
            .any(|entry| entry.path() == fixture.path("a/closed/unobserved"))
    );
    fs::remove_file(fixture.path("a/keep")).unwrap();
    tree.refresh().unwrap();
    assert_eq!(selected(&tree), fixture.path("a"));
    assert!(tree.selected().unwrap().expanded());
}

// Real bound navigation and named commands share tree handlers. Enter repeats
// cannot toggle again, and returning to editing retains text and its history.
#[test]
fn command_navigation() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use rev::command::{Command, TreeCommand};
    let mut session = super::editor();
    session.insert('!');
    super::tree(&mut session, TreeCommand::Toggle);
    session.handle(super::key(KeyCode::Enter));
    assert!(session.tree().selected().unwrap().expanded());
    let count = session.tree().entries().len();
    let mut repeat = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    repeat.kind = KeyEventKind::Repeat;
    session.handle(Event::Key(repeat));
    assert_eq!(session.tree().entries().len(), count);
    session.handle(super::key(KeyCode::Left));
    assert!(!session.tree().selected().unwrap().expanded());
    super::named(&mut session, "tree-expand");
    assert!(session.tree().selected().unwrap().expanded());
    session.handle(super::key(KeyCode::Down));
    session.handle(super::key(KeyCode::Up));
    assert_eq!(
        session.tree().selected().unwrap().path(),
        session.tree().root().unwrap()
    );
    super::named(&mut session, "tree-collapse");
    super::named(&mut session, "tree-refresh");
    assert_eq!(session.tree().entries().len(), 1);
    session.handle(super::key(KeyCode::Tab));
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(super::text(&session), "");
    for command in [
        TreeCommand::Previous,
        TreeCommand::Next,
        TreeCommand::Expand,
        TreeCommand::Collapse,
        TreeCommand::Activate,
        TreeCommand::Refresh,
    ] {
        assert_eq!(Command::named(command.name()), Some(Command::Tree(command)));
        super::tree(&mut session, command);
        assert_eq!(session.echo(), "File tree is not focused");
    }
}

#[path = "failures.rs"]
mod failures;
