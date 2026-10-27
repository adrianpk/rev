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
            expected + 1
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
