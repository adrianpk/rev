use crate::support::{self, DOCUMENT, Result, settle};
use crate::surface;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use rev::buffer::Buffers;
use rev::command::{
    BookmarkCommand, Command, CompletionCommand, GutterCommand, LinearCommand, Motion, OrgCommand,
    RegionCommand, Scroll,
};
use rev::config::Rgb;
use rev::coordinates::Scalar;
use rev::display::{Frame, Position};
use rev::gutter::Side;
use rev::session::Session;
use rev::view::{Size, View};
use rev::window::Area;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

pub(super) fn run(root: &Path) -> Result<()> {
    for (name, preferences, glyph, color) in [
        ("factory", "", '■', Rgb(0x7a, 0xa2, 0xf7)),
        (
            "nerd",
            "[gutter]\nnerd_fonts=true\n",
            '\u{f02e}',
            Rgb(0x7a, 0xa2, 0xf7),
        ),
        (
            "legacy",
            "include=['font-base.toml']\n",
            '■',
            Rgb(0xe0, 0xaf, 0x68),
        ),
    ] {
        presentation(root, name, preferences, glyph, color)?;
    }
    folded(root)?;
    completion(root)?;
    println!(
        "PASS native gutter: fixed symbols, legacy input/color, both sides, absolute/relative/hidden numbers, markers, narrow/resize/wrap, folds, completion, source-only copy and unchanged source/history"
    );
    Ok(())
}

fn open(root: &Path, name: &str, source: &str, preferences: &str) -> Result<Session> {
    let mut config = support::config("non-modal");
    if name == "legacy.txt" {
        std::fs::write(
            root.join("font-base.toml"),
            "gutter.nerd_fonts=true\nbookmarks.marker='ignored wide 界'\npalette.gutter.bookmark='#e0af68'\n",
        )?;
        config = format!("{preferences}{config}[gutter]\nnerd_fonts=false\n");
    } else {
        config.push_str(preferences);
    }
    let config_path = root.join("native.toml");
    std::fs::write(&config_path, config)?;
    let startup = rev::config::prepare(&config_path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let path = root.join("workspace").join(name);
    std::fs::write(&path, source)?;
    let mut buffers = Buffers::with_settings(startup.snapshot.clone())?;
    let id = buffers.visit(&path)?;
    let view = View::with_settings(buffers.get(id)?, &startup.snapshot);
    Ok(Session::new(
        buffers,
        view,
        Size {
            width: 40,
            height: 12,
        },
    )?)
}

fn bookmark(editor: &mut Session, action: BookmarkCommand, name: &str) {
    editor.execute(Command::Bookmark(action));
    editor.handle(Event::Paste(name.into()));
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

fn count(frame: &Frame, glyph: char) -> usize {
    frame
        .rows()
        .iter()
        .map(|row| row.text().matches(glyph).count())
        .sum()
}

fn presentation(root: &Path, name: &str, preferences: &str, glyph: char, color: Rgb) -> Result<()> {
    let file = format!("{name}.txt");
    let mut editor = open(root, &file, DOCUMENT, preferences)?;
    bookmark(&mut editor, BookmarkCommand::Set, "A");
    bookmark(&mut editor, BookmarkCommand::Set, "B");
    let records = editor.bookmarks().iter().cloned().collect::<Vec<_>>();
    let before = (editor.buffer()?.stamp(), editor.buffer()?.state());
    for side in [Side::Left, Side::Right] {
        if editor.view().gutter.effective().side != side {
            editor.execute(Command::Gutter(GutterCommand::ToggleSide));
        }
        for mode in [GutterCommand::Absolute, GutterCommand::Relative] {
            editor.execute(Command::Gutter(mode));
            for visible in [true, false] {
                if editor.view().gutter.effective().visible != visible {
                    editor.execute(Command::Gutter(GutterCommand::Toggle));
                }
                for markers in [true, false] {
                    if editor.view().bookmark_markers != markers {
                        editor.execute(Command::Bookmark(BookmarkCommand::Markers));
                    }
                    for width in [3, 4, 9, 10, 40] {
                        geometry(&mut editor, width, glyph, color)?;
                    }
                }
            }
        }
    }
    copied(&mut editor)?;
    assert_eq!(
        editor.bookmarks().iter().cloned().collect::<Vec<_>>(),
        records
    );
    assert_eq!((editor.buffer()?.stamp(), editor.buffer()?.state()), before);
    assert!(editor.buffer()?.history().is_empty());
    let mut bytes = Vec::new();
    rev::file::write(editor.buffer()?, &mut bytes)?;
    assert_eq!(bytes, DOCUMENT.as_bytes());
    assert_eq!(std::fs::read(root.join("workspace").join(file))?, bytes);
    Ok(())
}

fn geometry(editor: &mut Session, width: u16, glyph: char, color: Rgb) -> Result<()> {
    editor.handle(Event::Resize(width, 12));
    let settings = editor.view().gutter.effective();
    let numbers = settings.visible && width >= 9;
    let field = if numbers { 6 } else { 0 };
    let marker = editor.view().bookmark_markers && width - field >= 4;
    let content = Area::new(0, 0, width, 10).ok_or("invalid native content")?;
    let geometry = editor.view().gutter_geometry(editor.buffer()?, content)?;
    assert_eq!(geometry.numbers().is_some(), numbers);
    assert_eq!(geometry.marker().is_some(), marker);
    assert_eq!(geometry.text().width() + field + u16::from(marker), width);
    let frame = editor.frame()?;
    assert_eq!(count(&frame, glyph), usize::from(marker));
    assert_eq!(
        frame.cursor(),
        Some(Position {
            column: geometry.text().column(),
            row: 0
        })
    );
    for row in &frame.rows()[..10] {
        assert_eq!(row.text().width(), usize::from(width));
    }
    if let Some(area) = geometry.marker() {
        assert_eq!(
            area.column(),
            if settings.side == Side::Left {
                0
            } else {
                width - 1
            }
        );
        let cells = surface::draw(&frame);
        let cell = cells[&(0, usize::from(area.column()))];
        assert_eq!((cell.character, cell.foreground), (glyph, Some(color)));
        assert_eq!(
            editor.view().hit(
                editor.buffer()?,
                content,
                Position {
                    column: area.column(),
                    row: 0
                }
            )?,
            None
        );
        crossterm::style::force_color_output(false);
        let mut bytes = Vec::new();
        frame.write(&mut bytes)?;
        let output = String::from_utf8(bytes)?;
        assert!(output.contains(glyph) && !output.contains("\x1b[38;"));
        crossterm::style::force_color_output(true);
    }
    Ok(())
}

fn copied(editor: &mut Session) -> Result<()> {
    let content = Area::new(0, 0, 40, 10).ok_or("invalid copy content")?;
    let text = editor
        .view()
        .gutter_geometry(editor.buffer()?, content)?
        .text();
    for (kind, offset) in [
        (MouseEventKind::Down(MouseButton::Left), 0),
        (MouseEventKind::Drag(MouseButton::Left), 5),
        (MouseEventKind::Up(MouseButton::Left), 5),
    ] {
        editor.handle(Event::Mouse(MouseEvent {
            kind,
            column: text.column() + offset,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }));
    }
    editor.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(
        editor
            .killed_text()
            .ok_or("missing pointer copy")?
            .to_string(),
        "alpha"
    );
    editor.execute(Command::KeyboardQuit);
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Region(RegionCommand::Set));
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(
        editor
            .killed_text()
            .ok_or("missing Unicode source copy")?
            .to_string(),
        DOCUMENT
    );
    Ok(())
}

fn folded(root: &Path) -> Result<()> {
    let source = "* Parent\nbody text that wraps across the viewport\n* Last\n";
    let mut editor = open(root, "folded.org", source, "[gutter]\nnerd_fonts=true\n")?;
    editor.execute(Command::Scroll(Scroll::NextLine));
    bookmark(&mut editor, BookmarkCommand::Set, "body");
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Org(OrgCommand::HideSubtree));
    settle(&mut editor)?;
    assert_eq!(count(&editor.frame()?, '\u{f02e}'), 0);
    bookmark(&mut editor, BookmarkCommand::Jump, "body");
    settle(&mut editor)?;
    assert_eq!(editor.view().point.position, Scalar(9));
    assert_eq!(count(&editor.frame()?, '\u{f02e}'), 1);
    assert!(!editor.buffer()?.is_modified());
    assert!(editor.buffer()?.history().is_empty());
    Ok(())
}

fn completion(root: &Path) -> Result<()> {
    let mut editor = open(
        root,
        "completion.rs",
        "fn cfg_value() {}\nfn main() { cfg }\n",
        "[gutter]\nnerd_fonts=true\n",
    )?;
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Scroll(Scroll::PreviousLine));
    editor.execute(Command::Motion(Motion::LineEnd));
    for _ in 0..2 {
        editor.execute(Command::Motion(Motion::BackwardChar));
    }
    bookmark(&mut editor, BookmarkCommand::Set, "completion");
    editor.execute(Command::Completion(CompletionCommand::AtPoint));
    settle(&mut editor)?;
    // The 40-column pane clamps the popup left; the source cursor stays put.
    for (width, popup_column) in [(80, 23), (40, 18), (80, 23)] {
        editor.handle(Event::Resize(width, 12));
        let frame = editor.frame()?;
        let selected = frame
            .rows()
            .iter()
            .find(|row| row.selection().is_some())
            .ok_or("missing native popup")?;
        assert!(selected.text().contains("cfg_value"));
        assert_eq!(
            selected.selection().ok_or("missing popup range")?.start,
            popup_column
        );
        assert_eq!(frame.cursor().ok_or("missing source cursor")?.column, 22);
    }
    assert!(!editor.buffer()?.is_modified());
    assert!(editor.buffer()?.history().is_empty());
    Ok(())
}
