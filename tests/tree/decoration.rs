use std::os::unix::fs::symlink;

use rev::buffer::Buffers;
use rev::config::prepare;
use rev::tree::Kind;
use rev::view::View;

use super::*;

fn configured(fixture: &config_fixture::Fixture, config: &str) -> Session {
    let source = fixture.write("root.rs/data.bin", "retained");
    let startup = prepare(&fixture.write(
        "config.toml",
        &format!("tree.visible=true\ntree.width=40\ntree.max_fraction=0.5\n{config}"),
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 16,
        },
    )
    .unwrap()
}

// The complete approved name catalog is independent of file contents and
// language registration; observed root/directory/link identity takes precedence.
#[test]
fn observed_kinds() {
    let fixture = config_fixture::Fixture::new();
    let mut cases = Vec::new();
    for (kind, suffixes) in [
        (
            Kind::Source,
            &[
                "rs", "go", "c", "h", "js", "jsx", "ts", "tsx", "html", "htm", "css",
            ][..],
        ),
        (Kind::Document, &["md", "markdown", "org", "txt", "rst"][..]),
        (
            Kind::Configuration,
            &["toml", "json", "yaml", "yml", "ini", "cfg", "conf", "xml"][..],
        ),
    ] {
        for suffix in suffixes {
            cases.push((format!("FILE.{}", suffix.to_ascii_uppercase()), kind));
        }
    }
    for name in [
        ".editorconfig",
        ".gitignore",
        ".gitattributes",
        "Cargo.lock",
        "Makefile",
        "Dockerfile",
    ] {
        cases.push((name.to_ascii_uppercase(), Kind::Configuration));
    }
    cases.extend([
        ("rs".into(), Kind::File),
        ("data.bin".into(), Kind::File),
        ("notes.md.tmp".into(), Kind::File),
    ]);
    for (name, _) in &cases {
        fixture.write(&format!("root.rs/{name}"), "not source code");
    }
    std::fs::create_dir(fixture.0.join("root.rs/directory.rs")).unwrap();
    symlink("directory.rs", fixture.0.join("root.rs/link.rs")).unwrap();
    symlink("absent", fixture.0.join("root.rs/dangling.toml")).unwrap();
    cases.extend([
        ("directory.rs".into(), Kind::Directory),
        ("link.rs".into(), Kind::Symlink),
        ("dangling.toml".into(), Kind::Symlink),
    ]);
    let mut session = configured(&fixture, "");
    assert_eq!(session.tree().entries()[0].kind(), Kind::Root);
    for (name, expected) in &cases {
        let entry = session
            .tree()
            .entries()
            .iter()
            .find(|entry| entry.path().ends_with(name))
            .unwrap();
        assert_eq!(entry.kind(), *expected, "{name}");
        if *expected == Kind::Symlink {
            assert!(!entry.is_directory() && !entry.expanded());
        }
    }
    let paths = session
        .tree()
        .entries()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect::<Vec<_>>();
    // Once observed, even removed entries can be painted and navigated without IO.
    std::fs::remove_dir_all(fixture.0.join("root.rs")).unwrap();
    tree(&mut session, TreeCommand::Focus);
    for _ in 0..cases.len() {
        tree(&mut session, TreeCommand::Next);
        session.frame().unwrap();
    }
    assert_eq!(
        session
            .tree()
            .entries()
            .iter()
            .map(|entry| entry.path())
            .collect::<Vec<_>>(),
        paths.iter().map(|path| path.as_path()).collect::<Vec<_>>()
    );
    assert_eq!(text(&session), "retained");
}

// Ordinary symbols distinguish all kinds. Explicit minimal mode retains exact
// prior labels under either font flag, including disclosure and directory slashes.
#[test]
fn label_modes() {
    for (name, config, root, directory, source, document, configuration, file, link) in [
        (
            "default",
            "",
            "▾ ◆ root.rs/",
            "  ▸ / folder.md/",
            "    λ main.rs",
            "    ≡ notes.org",
            "    = Cargo.toml",
            "    · data.bin",
            "    @ link.rs",
        ),
        (
            "ordinary",
            "tree.decorated=true",
            "▾ ◆ root.rs/",
            "  ▸ / folder.md/",
            "    λ main.rs",
            "    ≡ notes.org",
            "    = Cargo.toml",
            "    · data.bin",
            "    @ link.rs",
        ),
        (
            "minimal",
            "tree.decorated=false",
            "v root.rs/",
            "  > folder.md/",
            "    main.rs",
            "    notes.org",
            "    Cargo.toml",
            "    data.bin",
            "  @ link.rs",
        ),
        (
            "minimal font",
            "tree.decorated=false\ngutter.nerd_fonts=true",
            "v root.rs/",
            "  > folder.md/",
            "    main.rs",
            "    notes.org",
            "    Cargo.toml",
            "    data.bin",
            "  @ link.rs",
        ),
    ] {
        let fixture = config_fixture::Fixture::new();
        for name in ["main.rs", "notes.org", "Cargo.toml"] {
            fixture.write(&format!("root.rs/{name}"), "");
        }
        std::fs::create_dir(fixture.0.join("root.rs/folder.md")).unwrap();
        symlink("folder.md", fixture.0.join("root.rs/link.rs")).unwrap();
        let mut session = configured(&fixture, config);
        let frame = session.frame().unwrap();
        for (row, entry) in session.tree().entries().iter().enumerate() {
            let expected = match entry.kind() {
                Kind::Root => root,
                Kind::Directory => directory,
                Kind::Source => source,
                Kind::Document => document,
                Kind::Configuration => configuration,
                Kind::File => file,
                Kind::Symlink => link,
            };
            assert_eq!(
                frame.rows()[row]
                    .text()
                    .split('│')
                    .next()
                    .unwrap()
                    .trim_end(),
                expected,
                "{name}: {:?}",
                entry.path()
            );
        }
        tree(&mut session, TreeCommand::Focus);
        tree(&mut session, TreeCommand::Next);
        tree(&mut session, TreeCommand::Expand);
        let expected = directory.replace('▸', "▾").replace('>', "v");
        assert!(
            session.frame().unwrap().rows()[1]
                .text()
                .starts_with(&expected),
            "{name}"
        );
        tree(&mut session, TreeCommand::Previous);
        tree(&mut session, TreeCommand::Collapse);
        let expected = root.replace('▾', "▸").replacen('v', ">", 1);
        assert!(
            session.frame().unwrap().rows()[0]
                .text()
                .starts_with(&expected),
            "{name}"
        );
    }
}
