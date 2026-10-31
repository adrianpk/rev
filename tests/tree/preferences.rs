use rev::buffer::Buffers;
use rev::config::prepare;
use rev::tree::Tree;
use rev::view::View;

use super::*;

// Hidden filtering and ordering happen during explicit observation. Symlinks
// stay leaves and filenames use lexical ordering rather than natural sorting.
#[test]
fn listing_policy() {
    use std::os::unix::fs::symlink;
    for (hidden, directories, expected) in [
        (
            true,
            true,
            vec![".folder", "z-dir", ".hidden", "file10", "file2", "link"],
        ),
        (false, true, vec!["z-dir", "file10", "file2", "link"]),
        (false, false, vec!["file10", "file2", "link", "z-dir"]),
    ] {
        let fixture = config_fixture::Fixture::new();
        let root = fixture.0.join("root");
        std::fs::create_dir_all(root.join(".folder")).unwrap();
        std::fs::create_dir(root.join("z-dir")).unwrap();
        for name in [".hidden", "file2", "file10"] {
            std::fs::write(root.join(name), b"").unwrap();
        }
        symlink("z-dir", root.join("link")).unwrap();
        let startup = prepare(&fixture.write(
            "config.toml",
            &format!("tree.show_hidden = {hidden}\ntree.directories_first = {directories}"),
        ));
        assert!(startup.diagnostic.is_none());
        let mut tree = Tree::with_settings(root, &startup.snapshot);
        tree.expand().unwrap();
        let names: Vec<_> = tree
            .entries()
            .iter()
            .skip(1)
            .map(|entry| entry.path().file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, expected);
        assert!(
            tree.entries()
                .iter()
                .find(|entry| entry.path().ends_with("link"))
                .unwrap()
                .is_symlink()
        );
        tree.refresh().unwrap();
        assert_eq!(
            tree.entries()
                .iter()
                .skip(1)
                .map(|entry| entry.path().file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

// Initial visibility retains editing focus, preferred widths obey the maximum
// share, and the historical small-frame threshold still hides the tree safely.
#[test]
fn initial_visibility() {
    let fixture = config_fixture::Fixture::new();
    for (width, fraction, expected) in [(1, 0.5, 1), (512, 0.5, 40), (24, 0.1, 8)] {
        let startup = prepare(&fixture.write(
            "config.toml",
            &format!("tree.visible = true\ntree.width = {width}\ntree.max_fraction = {fraction}"),
        ));
        assert!(startup.diagnostic.is_none());
        let buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let view = View::new(buffers.get(buffers.initial()).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 12,
            },
        )
        .unwrap();
        assert!(session.tree().visible() && !session.tree().focused());
        assert_eq!(
            session.tree().width(Size {
                width: 80,
                height: 12
            }),
            expected
        );
        assert_eq!(
            session.frame().unwrap().cursor().unwrap().column,
            expected + 1 + 6 + 1
        );
        tree(&mut session, TreeCommand::Focus);
        session.handle(Event::Resize(31, 4));
        assert!(session.tree().visible() && !session.tree().focused());
        assert_eq!(
            session.tree().width(Size {
                width: 31,
                height: 4
            }),
            0
        );
    }
}

// Automatic file-parent roots follow visits. Configured cwd retains its root;
// explicit directory precedence is exercised by the directory-opening cases.
#[test]
fn startup_roots() {
    for (policy, initial) in [
        ("auto", true),
        ("cwd", true),
        ("file-parent", true),
        ("auto", false),
        ("file-parent", false),
    ] {
        let fixture = config_fixture::Fixture::new();
        let file = fixture.write("first/main.rs", "fn main() {}\n");
        let later = fixture.write("later/other.txt", "retained");
        let startup = prepare(&fixture.write(
            "config.toml",
            &format!("tree.root = '{policy}'\ntree.visible = true"),
        ));
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = if initial {
            buffers.visit(&file).unwrap()
        } else {
            buffers.initial()
        };
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 12,
            },
        )
        .unwrap();
        let expected = if initial && policy != "cwd" {
            file.parent().unwrap().to_path_buf()
        } else {
            std::env::current_dir().unwrap()
        };
        assert_eq!(session.tree().root(), Some(expected.as_path()));
        assert!(!session.tree().focused());
        session.load(&later).unwrap();
        tree(&mut session, TreeCommand::Focus);
        tree(&mut session, TreeCommand::Toggle);
        tree(&mut session, TreeCommand::Toggle);
        let expected = if policy == "cwd" {
            expected
        } else {
            later.parent().unwrap().to_owned()
        };
        assert_eq!(session.tree().root(), Some(expected.as_path()));
        assert_eq!(text(&session), "retained");
    }
}

// Disappearing observed paths cannot turn redraw or visible movement into IO.
// A focused pending command, root and loaded entries survive those projections.
#[test]
fn memory_only_draw() {
    let fixture = config_fixture::Fixture::new();
    let file = fixture.write("root/main.rs", "fn main() {}\n");
    fixture.write("root/.hidden", "hidden");
    let startup = prepare(&fixture.write(
        "config.toml",
        "tree.visible = true\ntree.show_hidden = false",
    ));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&file).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    tree(&mut session, TreeCommand::Focus);
    tree(&mut session, TreeCommand::Expand);
    let root = session.tree().root().unwrap().to_path_buf();
    let observed: Vec<_> = session
        .tree()
        .entries()
        .iter()
        .map(|entry| entry.path().to_path_buf())
        .collect();
    assert_eq!(observed.len(), 2);
    std::fs::rename(&root, fixture.0.join("moved")).unwrap();
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("find".into()));
    for _ in 0..3 {
        let frame = session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .find(|row| row.band() == rev::display::Band::Echo)
                .unwrap()
                .text()
                .contains("M-x find")
        );
        assert!(session.tree().focused());
        assert_eq!(
            session
                .tree()
                .entries()
                .iter()
                .map(|entry| entry.path().to_path_buf())
                .collect::<Vec<_>>(),
            observed
        );
    }
    session.execute(Command::KeyboardQuit);
    tree(&mut session, TreeCommand::Next);
    session.frame().unwrap();
    assert_eq!(session.tree().root(), Some(root.as_path()));
    assert_eq!(text(&session), "fn main() {}\n");
    assert!(session.buffer().unwrap().history().is_empty());
}
