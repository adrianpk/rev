use std::os::unix::fs::symlink;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, TreeCommand};
use rev::config::{Palette, Rgb, prepare};
use rev::session::Session;
use rev::tree::Kind;

use super::*;

// Parent navigation uses ordinary root/directory rows and semantic terminal
// cells, including Nerd Fonts, full-row selection and narrow split geometry.
#[test]
fn parent_presentation() {
    for decorated in [false, true] {
        for nerd_fonts in [false, true] {
            for width in [1, 6, 24, 40] {
                check_parent_presentation(decorated, nerd_fonts, width);
            }
        }
    }
}

fn check_parent_presentation(decorated: bool, nerd_fonts: bool, width: usize) {
    let f = config_fixture::Fixture::new();
    let source = f.write("parent/child/main.rs", "retained");
    for name in ["main.rs", "notes.org", "Cargo.toml", "data.bin"] {
        f.write(&format!("parent/{name}"), "");
    }
    symlink("child", f.0.join("parent/link")).unwrap();
    let startup = prepare(&f.write("config.toml", &format!(
        "tree.width={width}\ntree.max_fraction=0.5\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}"
    )));
    assert!(startup.diagnostic.is_none());
    let palette = startup.snapshot.settings().palette.clone();
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 16,
        },
    )
    .unwrap();
    session.execute(Command::Split(rev::command::Split::Right));
    session.execute(Command::Tree(TreeCommand::Focus));
    session.execute(Command::Tree(TreeCommand::UpDirectory));
    assert_eq!(session.tree().root(), source.parent().unwrap().parent());
    assert_eq!(
        session.tree().selected().unwrap().path(),
        source.parent().unwrap()
    );
    let frame = session.frame().unwrap();
    assert_eq!(frame.cursor().unwrap(), Position { row: 2, column: 0 });
    assert!(frame.rows().iter().all(|r| r.text().width() == 100));
    let cells = surface::draw(&frame);
    for (row, entry) in session.tree().entries().iter().enumerate() {
        assert_eq!(
            frame.rows()[row].text().split('│').next().unwrap().width(),
            width
        );
        let selected = row == 2;
        let expected = if selected {
            palette.tree_selected_text
        } else if !decorated {
            palette.foreground
        } else {
            entry_foreground(&palette, entry.kind())
        };
        for column in 0..width {
            assert_eq!(cells[&(row, column)].foreground, Some(expected));
            assert_eq!(
                cells[&(row, column)].background,
                Some(if selected {
                    palette.tree_selected
                } else {
                    palette.tree
                })
            );
        }
    }
    if width >= 24 {
        check_parent_labels(&frame, decorated, nerd_fonts);
    }
    assert!(!session.buffer().unwrap().is_modified());
}

fn entry_foreground(palette: &Palette, kind: Kind) -> Rgb {
    match kind {
        Kind::Root => palette.tree_entries.root,
        Kind::Directory => palette.tree_entries.directory,
        Kind::Source => palette.tree_entries.source,
        Kind::Document => palette.tree_entries.document,
        Kind::Configuration => palette.tree_entries.configuration,
        Kind::File => palette.tree_entries.file,
        Kind::Symlink => palette.tree_entries.symlink,
    }
}

fn check_parent_labels(frame: &Frame, decorated: bool, nerd_fonts: bool) {
    let root = if !decorated {
        "v parent/"
    } else if nerd_fonts {
        "▾ \u{f1b2} parent/"
    } else {
        "▾ ◆ parent/"
    };
    let child = if !decorated {
        "  > child/"
    } else if nerd_fonts {
        "  ▸ \u{f07b} child/"
    } else {
        "  ▸ / child/"
    };
    assert!(frame.rows()[0].text().starts_with(root));
    assert!(frame.rows()[2].text().starts_with(child));
}

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
        check_entry_palette(&mut session, &fixture, &palette, decorated);
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
        let active_row = session
            .tree()
            .entries()
            .iter()
            .position(|e| e.path() == fixture.0.join("root/main.rs"))
            .unwrap();
        let cells = surface::draw(&session.frame().unwrap());
        for column in 0..24 {
            assert_eq!(
                (
                    cells[&(active_row, column)].foreground,
                    cells[&(active_row, column)].background
                ),
                (
                    Some(palette.tree_selected_text),
                    Some(palette.tree_selected)
                )
            );
        }
        assert!(session.frame().unwrap().cursor().unwrap().column > 24);
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

fn check_entry_palette(
    session: &mut Session,
    fixture: &config_fixture::Fixture,
    palette: &Palette,
    decorated: bool,
) {
    let cells = surface::draw(&session.frame().unwrap());
    for (row, entry) in session.tree().entries().iter().enumerate() {
        let channel = entry_channel(entry.kind());
        let active = entry.path() == fixture.0.join("root/main.rs");
        let expected = if active {
            palette.tree_selected_text
        } else if decorated {
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
                (
                    Some(expected),
                    Some(if active {
                        palette.tree_selected
                    } else {
                        palette.tree
                    })
                ),
                "{decorated} {:?}",
                entry.kind()
            );
        }
    }
}

fn entry_channel(kind: Kind) -> u8 {
    match kind {
        Kind::Root => 1,
        Kind::Directory => 2,
        Kind::Source => 3,
        Kind::Document => 4,
        Kind::Configuration => 5,
        Kind::File => 6,
        Kind::Symlink => 7,
    }
}

// Persistent tree feedback must coexist with an active multiline source region
// on the same screen row, preserving both palette roles and source state.
#[test]
fn tree_and_region() {
    for decorated in [false, true] {
        let fixture = config_fixture::Fixture::new();
        let source = fixture.write("root/main.rs", "first\nsecond\nthird");
        let startup = prepare(&fixture.write(
            "config.toml",
            &format!("tree.visible=true\ntree.decorated={decorated}"),
        ));
        assert!(startup.diagnostic.is_none());
        let palette = startup.snapshot.settings().palette.clone();
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&source).unwrap();
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
        session.execute(Command::Region(rev::command::RegionCommand::Set));
        session.execute(Command::Motion(rev::command::Motion::BufferEnd));
        let stamp = session.buffer().unwrap().stamp();
        let mark = session.buffer().unwrap().mark();
        let point = session.view().point;
        let frame = session.frame().unwrap();
        let region = frame.rows()[2].selection().unwrap();
        assert!(region.start > 24);
        let cells = surface::draw(&frame);
        for column in 0..24 {
            assert_eq!(
                (
                    cells[&(2, column)].foreground,
                    cells[&(2, column)].background
                ),
                (
                    Some(palette.tree_selected_text),
                    Some(palette.tree_selected)
                )
            );
        }
        for column in usize::from(region.start)..usize::from(region.end) {
            assert_eq!(
                (
                    cells[&(2, column)].foreground,
                    cells[&(2, column)].background
                ),
                (Some(palette.selection_text), Some(palette.selection))
            );
        }
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert_eq!(session.view().point, point);
        assert!(session.buffer().unwrap().history().is_empty());
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
        check_monochrome_tree(decorated, nerd_fonts);
    }
}

fn check_monochrome_tree(decorated: bool, nerd_fonts: bool) {
    let fixture = config_fixture::Fixture::new();
    let source = fixture.write("root/main.rs", "first\nsecond\nthird");
    let startup = prepare(&fixture.write(
        "config.toml",
        &format!("tree.visible=true\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}"),
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
    assert!(frame.rows()[2].text().starts_with(source));
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(output.contains("\x1b[7m"));
    assert!(output.contains(root) && output.contains(source));
    session.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].selection(), None);
    assert_eq!(frame.rows()[2].selection(), Some(0..24));
    assert!(frame.cursor().unwrap().column > 24);
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(output.contains("\x1b[7m"));
    session.execute(Command::Region(rev::command::RegionCommand::Set));
    session.execute(Command::Motion(rev::command::Motion::BufferEnd));
    let frame = session.frame().unwrap();
    assert!(frame.rows()[1].selection().unwrap().start > 24);
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(output.matches("\x1b[7m").count() >= 4);
    assert!(!session.buffer().unwrap().is_modified());
}
