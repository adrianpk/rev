use crate::support::{DOCUMENT, Result};
use crate::surface;
use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::buffer::Buffers;
use rev::command::{BookmarkCommand, Command, TreeCommand};
use rev::config::{Rgb, prepare};
use rev::panel::Composition;
use rev::session::Session;
use rev::tree::{Entry, Hit, Kind};
use rev::view::{Size, View};
use std::collections::BTreeMap;
use std::os::unix::fs::symlink;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

#[path = "configuration.rs"]
mod configuration;

const SIZE: Size = Size {
    width: 100,
    height: 16,
};
const ROLES: [&str; 7] = [
    "root",
    "directory",
    "source",
    "document",
    "configuration",
    "file",
    "symlink",
];

pub(super) fn run(root: &Path) -> Result<()> {
    configuration::run(root)?;
    for profile in ["non-modal", "modal"] {
        for (name, decorated, nerd_fonts, explicit) in [
            ("factory", true, false, false),
            ("ordinary", true, false, true),
            ("icons", true, true, true),
            ("minimal", false, false, true),
            ("minimal-font", false, true, true),
        ] {
            let started = std::time::Instant::now();
            let fixture = root.join(format!("{profile}-{name}"));
            let mut editor = open(&fixture, profile, decorated, nerd_fonts, explicit)?;
            presentation(&mut editor, decorated, nerd_fonts)?;
            interaction(&mut editor, &fixture)?;
            unchanged(&editor, &fixture)?;
            println!(
                "MEASURE native {profile}/{name}: elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
    }
    println!(
        "PASS native tree: both profiles, omitted defaults and four settings, seven kinds/colors, shared bookmark font, retained automatic width, frozen configuration, directory expansion, canonical full-row selection/open, cached scroll/resize and unchanged source/history"
    );
    Ok(())
}

fn open(
    root: &Path,
    profile: &str,
    decorated: bool,
    nerd_fonts: bool,
    explicit: bool,
) -> Result<Session> {
    prepare_files(root)?;
    let palette = ROLES
        .iter()
        .enumerate()
        .map(|(i, role)| format!("palette.tree_entries.{role}='#{:02x}0203'\n", i + 1))
        .collect::<String>();
    let choice = if explicit {
        format!("tree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}\n")
    } else {
        String::new()
    };
    let config = format!(
        "{choice}{palette}tree.visible=true\nmouse.tree_activate_on_click=false\nbookmarks.save='manual'\nbookmarks.file='{}'\neditor.interaction_profile='{profile}'\ncompletion.automatic=false\n",
        root.join("bookmarks.toml").display()
    );
    // The root table and included table deliberately compose unrelated leaves.
    std::fs::write(root.join("base.toml"), config)?;
    let path = root.join("config.toml");
    std::fs::write(&path, "include=['base.toml']\n")?;
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone())?;
    let id = buffers.visit(&root.join("workspace/main.rs"))?;
    let view = View::with_settings(buffers.get(id)?, &startup.snapshot);
    let mut editor = Session::new(buffers, view, SIZE)?;
    editor.execute(Command::Bookmark(BookmarkCommand::Set));
    editor.handle(Event::Paste("shared-font".into()));
    editor.handle(Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    // Neither consumer may reread edited configuration during a live session.
    std::fs::write(
        &path,
        format!(
            "tree.decorated={}\ngutter.nerd_fonts={}\n",
            !decorated, !nerd_fonts
        ),
    )?;
    Ok(editor)
}

fn prepare_files(root: &Path) -> Result<()> {
    std::fs::create_dir_all(root.join("workspace/folder"))?;
    for name in [
        "main.rs",
        "notes.org",
        "Cargo.toml",
        "data.bin",
        "界é.txt",
        "ctl\x1b[31m.txt",
        "folder/child.txt",
    ] {
        std::fs::write(root.join("workspace").join(name), DOCUMENT)?;
    }
    symlink("folder", root.join("workspace/link.rs"))?;
    Ok(())
}

fn role(kind: Kind) -> (usize, char, char) {
    match kind {
        Kind::Root => (1, '◆', '\u{f1b2}'),
        Kind::Directory => (2, '/', '\u{f07b}'),
        Kind::Source => (3, 'λ', '\u{f121}'),
        Kind::Document => (4, '≡', '\u{f0219}'),
        Kind::Configuration => (5, '=', '\u{e615}'),
        Kind::File => (6, '·', '\u{f15b}'),
        Kind::Symlink => (7, '@', '\u{f481}'),
    }
}

fn presentation(editor: &mut Session, decorated: bool, nerd_fonts: bool) -> Result<()> {
    let frame = editor.frame()?;
    let cells = surface::draw(&frame);
    let style = TreeStyle {
        decorated,
        nerd_fonts,
        palette: rev::config::Palette::default(),
    };
    let mut seen = [false; 7];
    for (row, entry) in editor.tree().entries().iter().enumerate() {
        let channel = style.entry(row, entry, &cells, entry.path().ends_with("main.rs"))?;
        seen[channel - 1] = true;
    }
    assert!(seen.into_iter().all(|seen| seen));
    style.labels(&frame);
    bookmark_marker(editor, &cells, nerd_fonts)?;
    selected_root(editor, &style)?;
    editor.execute(Command::Tree(TreeCommand::Next));
    assert!(
        editor
            .tree()
            .selected()
            .ok_or("missing parent row")?
            .is_parent()
    );
    editor.execute(Command::Tree(TreeCommand::Next));
    editor.execute(Command::Tree(TreeCommand::Expand));
    let frame = editor.frame()?;
    style.directory(&frame);
    editor.execute(Command::Tree(TreeCommand::Collapse));
    Ok(())
}

fn source_color(
    editor: &Session,
    style: &TreeStyle,
    cells: &BTreeMap<(usize, usize), surface::Cell>,
) -> Result<()> {
    let (row, source) = editor
        .tree()
        .entries()
        .iter()
        .enumerate()
        .find(|(_, entry)| entry.path().ends_with("main.rs"))
        .ok_or("missing source entry")?;
    style.entry(row, source, cells, false)?;
    Ok(())
}

fn press(column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn interaction(editor: &mut Session, root: &Path) -> Result<()> {
    let row = u16::try_from(
        editor
            .tree()
            .entries()
            .iter()
            .position(|entry| entry.path().ends_with("界é.txt"))
            .ok_or("missing Unicode entry")?,
    )?;
    let hit = editor.tree().hit(row).ok_or("missing canonical hit")?;
    select_entry(editor, row, &hit)?;
    editor.execute(Command::Tree(TreeCommand::Open));
    assert!(!editor.tree().focused());
    assert!(
        editor
            .buffer()?
            .file()
            .ok_or("missing opened file")?
            .path()
            .ends_with("界é.txt")
    );
    editor.execute(Command::Tree(TreeCommand::Focus));
    cached_tree(editor, root)?;
    Ok(())
}

fn unchanged(editor: &Session, root: &Path) -> Result<()> {
    let buffer = editor.buffer()?;
    assert!(!buffer.is_modified() && buffer.history().is_empty());
    let mut bytes = Vec::new();
    rev::file::write(buffer, &mut bytes)?;
    assert_eq!(bytes, DOCUMENT.as_bytes());
    assert!(std::fs::read_to_string(root.join("config.toml"))?.starts_with("tree.decorated="));
    Ok(())
}

pub(super) fn no_color(root: &Path) -> Result<()> {
    assert!(std::env::var_os("NO_COLOR").is_some());
    // Both native invocations use the same environment and fixed deadline.
    // Keep the complete geometry matrix with this smaller rendering journey.
    configuration::geometry(&root.join("configuration"))?;
    for (decorated, nerd_fonts) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut editor = open(
            &root.join(format!("no-color-{decorated}-{nerd_fonts}")),
            "non-modal",
            decorated,
            nerd_fonts,
            true,
        )?;
        editor.execute(Command::Tree(TreeCommand::Focus));
        let mut bytes = Vec::new();
        editor.frame()?.write(&mut bytes)?;
        let text = String::from_utf8(bytes)?;
        assert!(!text.contains("38;2;") && !text.contains("48;2;"));
        assert!(text.contains("\x1b[7m"));
        assert!(text.contains(if !decorated {
            "v workspace/"
        } else if nerd_fonts {
            "▾ \u{f1b2} workspace/"
        } else {
            "▾ ◆ workspace/"
        }));
    }
    println!(
        "PASS isolated NO_COLOR: all four settings retain symbols and reverse-video selection without RGB"
    );
    Ok(())
}

fn expected_kind(path: &Path) -> Kind {
    match path.file_name().and_then(|name| name.to_str()) {
        Some("workspace") => Kind::Root,
        Some("folder") => Kind::Directory,
        Some("main.rs") => Kind::Source,
        Some("Cargo.toml") => Kind::Configuration,
        Some("data.bin") => Kind::File,
        Some("link.rs") => Kind::Symlink,
        _ => Kind::Document,
    }
}
struct TreeStyle {
    decorated: bool,
    nerd_fonts: bool,
    palette: rev::config::Palette,
}
impl TreeStyle {
    fn entry(
        &self,
        row: usize,
        entry: &Entry,
        cells: &BTreeMap<(usize, usize), surface::Cell>,
        selected: bool,
    ) -> Result<usize> {
        let expected = if entry.is_parent() {
            Kind::Directory
        } else {
            expected_kind(entry.path())
        };
        assert_eq!(entry.kind(), expected);
        let (channel, ordinary, icon) = if entry.is_parent() {
            (2, '↑', '\u{f148}')
        } else {
            role(entry.kind())
        };
        let column = if row == 0 { 2 } else { 4 };
        let cell = cells[&(row, column)];
        if self.decorated {
            assert_eq!(
                cell.character,
                if self.nerd_fonts { icon } else { ordinary }
            );
        }
        assert_eq!(
            (cell.foreground, cell.background),
            (
                Some(if selected {
                    self.palette.tree_selected_text
                } else if self.decorated {
                    Rgb(u8::try_from(channel)?, 2, 3)
                } else {
                    self.palette.foreground
                }),
                Some(if selected {
                    self.palette.tree_selected
                } else {
                    self.palette.tree
                })
            )
        );
        assert_eq!(cells[&(row, column + 2)].foreground, cell.foreground);
        Ok(channel)
    }
    fn labels(&self, frame: &rev::display::Frame) {
        let expected_root = if !self.decorated {
            "v workspace/"
        } else if self.nerd_fonts {
            "▾ \u{f1b2} workspace/"
        } else {
            "▾ ◆ workspace/"
        };
        assert!(frame.rows()[0].text().starts_with(expected_root));
        let parent = if !self.decorated {
            "    .."
        } else if self.nerd_fonts {
            "    \u{f148} .."
        } else {
            "    ↑ .."
        };
        assert!(frame.rows()[1].text().starts_with(parent));
        if !self.decorated {
            for label in ["  > folder/", "    main.rs", "    notes.org", "  @ link.rs"] {
                assert!(frame.rows().iter().any(|row| row.text().starts_with(label)));
            }
        }
        assert!(
            frame
                .rows()
                .iter()
                .any(|row| row.text().contains("ctl^[[31m.txt"))
        );
    }
    fn directory(&self, frame: &rev::display::Frame) {
        let directory = if !self.decorated {
            "  v folder/"
        } else if self.nerd_fonts {
            "  ▾ \u{f07c} folder/"
        } else {
            "  ▾ / folder/"
        };
        assert!(frame.rows()[2].text().starts_with(directory));
        assert!(frame.rows()[3].text().contains("child.txt"));
    }
}
fn bookmark_marker(
    editor: &mut Session,
    cells: &BTreeMap<(usize, usize), surface::Cell>,
    nerd_fonts: bool,
) -> Result<()> {
    let composition = Composition::workspace(editor.windows(), editor.tree(), SIZE, 0)
        .ok_or("missing composition")?;
    let content = composition.panels()[0].1.content();
    let marker = editor
        .view()
        .gutter_geometry(editor.buffer()?, content)?
        .marker()
        .ok_or("missing marker")?;
    assert_eq!(
        cells[&(usize::from(marker.row()), usize::from(marker.column()))].character,
        if nerd_fonts { '\u{f02e}' } else { '■' }
    );
    Ok(())
}
fn selected_root(editor: &mut Session, style: &TreeStyle) -> Result<()> {
    let palette = &style.palette;
    editor.execute(Command::Tree(TreeCommand::Focus));
    let cells = surface::draw(&editor.frame()?);
    for column in 0..24 {
        assert_eq!(
            (
                cells[&(0, column)].foreground,
                cells[&(0, column)].background
            ),
            (
                Some(palette.tree_selected_text),
                Some(palette.tree_selected)
            )
        );
    }
    // Selection and source-kind assertions share this unchanged native frame.
    source_color(editor, style, &cells)
}
fn select_entry(editor: &mut Session, row: u16, hit: &Hit) -> Result<()> {
    for column in [0, 2, 4, 6, 23] {
        editor.handle(press(column, row));
        assert!(
            editor
                .tree()
                .selected()
                .ok_or("missing selection")?
                .path()
                .ends_with("界é.txt")
        );
        assert_eq!(editor.tree().hit(row), Some((*hit).clone()));
        assert!(editor.tree().focused());
        assert!(
            editor
                .buffer()?
                .file()
                .ok_or("missing file")?
                .path()
                .ends_with("main.rs")
        );
    }
    Ok(())
}
fn cached_tree(editor: &mut Session, root: &Path) -> Result<()> {
    let desired = editor.tree().desired_width();
    // Once observed, selection, clipping, movement and resize use cached paths.
    std::fs::remove_dir_all(root.join("workspace"))?;
    for width in [31, 0, 100, 40, 100] {
        editor.handle(Event::Resize(width, if width == 0 { 0 } else { 16 }));
        let frame = editor.frame()?;
        assert_eq!(editor.tree().desired_width(), desired);
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
        if width > 0 {
            assert_eq!(editor.tree().focused(), width >= 32);
        }
    }
    editor.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 4,
        row: 1,
        modifiers: KeyModifiers::NONE,
    }));
    editor.frame()?;
    assert!(
        editor
            .tree()
            .selected()
            .ok_or("missing cached selection")?
            .path()
            .ends_with("界é.txt")
    );
    Ok(())
}
