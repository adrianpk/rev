use std::os::unix::fs::symlink;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, TreeCommand};
use rev::config::{Rgb, prepare};
use rev::session::Session;
use rev::tree::Kind;

use super::*;

// Actual terminal cells use each configured entry foreground over the existing
// background. Focused selection wins; minimal mode ignores all new foregrounds.
#[test]
fn entry_colors() {
    let fixture = config_fixture::Fixture::new();
    for name in ["main.rs", "notes.org", "Cargo.toml", "data.bin"] {
        fixture.write(&format!("root/{name}"), "retained");
    }
    std::fs::create_dir(fixture.0.join("root/folder")).unwrap();
    symlink("folder", fixture.0.join("root/link.rs")).unwrap();
    let roles = [
        "root",
        "directory",
        "source",
        "document",
        "configuration",
        "file",
        "symlink",
    ];
    let colors = roles
        .iter()
        .enumerate()
        .map(|(index, role)| format!("palette.tree_entries.{role}='#{:02x}0203'\n", index + 1))
        .collect::<String>();
    for (decorated, nerd_fonts) in [(false, false), (false, true), (true, false), (true, true)] {
        let startup = prepare(&fixture.write(
            "config.toml",
            &format!("tree.visible=true\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}\n{colors}"),
        ));
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let palette = startup.snapshot.settings().palette.clone();
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&fixture.0.join("root/main.rs")).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 100,
                height: 14,
            },
        )
        .unwrap();
        let cells = surface::draw(&session.frame().unwrap());
        for (row, entry) in session.tree().entries().iter().enumerate() {
            let channel = match entry.kind() {
                Kind::Root => 1,
                Kind::Directory => 2,
                Kind::Source => 3,
                Kind::Document => 4,
                Kind::Configuration => 5,
                Kind::File => 6,
                Kind::Symlink => 7,
            };
            let expected = if decorated {
                Rgb(channel, 2, 3)
            } else {
                palette.foreground
            };
            let icon = if row == 0 { 2 } else { 4 };
            let label = if row == 0 { 4 } else { 6 };
            for column in [icon, label] {
                let cell = cells[&(row, column)];
                assert_eq!(
                    (cell.foreground, cell.background),
                    (Some(expected), Some(palette.tree)),
                    "{decorated} {:?}",
                    entry.kind()
                );
            }
        }
        session.execute(Command::Tree(TreeCommand::Focus));
        for row in [0, 1] {
            if row == 1 {
                session.execute(Command::Tree(TreeCommand::Next));
            }
            let cells = surface::draw(&session.frame().unwrap());
            for column in 0..24 {
                let cell = cells[&(row, column)];
                assert_eq!(
                    (cell.foreground, cell.background),
                    (
                        Some(palette.tree_selected_text),
                        Some(palette.tree_selected)
                    )
                );
            }
        }
        session.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
        let cells = surface::draw(&session.frame().unwrap());
        assert_eq!(
            cells[&(1, 6)].foreground,
            Some(if decorated {
                Rgb(2, 2, 3)
            } else {
                palette.foreground
            })
        );
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// A fresh child avoids the color cache shared by other renderer tests. Tree
// structure and focused reverse-video selection survive without RGB output.
#[test]
fn no_color() {
    if std::env::var_os("REV_TREE_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tree::no_color", "--nocapture"])
            .env("REV_TREE_COLOR_CHILD", "1")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    for (decorated, nerd_fonts) in [(false, false), (false, true), (true, false), (true, true)] {
        let fixture = config_fixture::Fixture::new();
        let source = fixture.write("root/main.rs", "retained");
        let startup = prepare(&fixture.write(
            "config.toml",
            &format!(
                "tree.visible=true\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}"
            ),
        ));
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&source).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 8,
            },
        )
        .unwrap();
        session.execute(Command::Tree(TreeCommand::Focus));
        let frame = session.frame().unwrap();
        let root = if !decorated {
            "v root/"
        } else if nerd_fonts {
            "▾ \u{f1b2} root/"
        } else {
            "▾ ◆ root/"
        };
        let source = if !decorated {
            "    main.rs"
        } else if nerd_fonts {
            "    \u{f121} main.rs"
        } else {
            "    λ main.rs"
        };
        assert!(frame.rows()[0].text().starts_with(root));
        assert!(frame.rows()[1].text().starts_with(source));
        let mut bytes = Vec::new();
        frame.write(&mut bytes).unwrap();
        let output = String::from_utf8(bytes).unwrap();
        assert!(!output.contains("38;2;") && !output.contains("48;2;"));
        assert!(output.contains("\x1b[7m"));
        assert!(output.contains(root) && output.contains(source));
        assert!(!session.buffer().unwrap().is_modified());
    }
}
