use super::support::{self, MARKERS, Result, settle, source};
use crate::surface;
use rev::command::{Command, History, MarkdownCommand as M, Motion, RegionCommand, Split};
use rev::config::Rgb;
use rev::coordinates::{Byte, Coordinates};
use rev::display::Frame;
use rev::session::Session;
use rev::view::Size;
use rev::window::Area;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

pub(crate) const COLORS: [Rgb; 6] = [
    Rgb(122, 162, 247),
    Rgb(157, 124, 216),
    Rgb(192, 202, 245),
    Rgb(155, 185, 249),
    Rgb(181, 156, 225),
    Rgb(188, 208, 251),
];

pub(super) fn run(root: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        for crlf in [false, true] {
            journey(root, profile, crlf)?;
        }
    }
    Ok(())
}

fn journey(root: &Path, profile: &str, crlf: bool) -> Result<()> {
    let original = support::document(crlf);
    let mut editor = support::open(root, profile, &original, "editor.soft_wrap=false\n")?;
    let canonical = source(&editor);
    presentation(&mut editor)?;
    geometry(&editor)?;
    copy_and_save(root, &mut editor, &original)?;
    crate::edits::run(&mut editor)?;
    assert_eq!(source(&editor), canonical);
    editor.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(root.join("workspace/native.md"))?,
        original.as_bytes()
    );
    split(&mut editor)?;
    disabled(root, profile, &original)?;
    println!(
        "PASS native Markdown {profile} crlf_bom={crlf} no_color={}: six heading cells, Setext, list depths, literal code, source/pretty, 80x24/32x12, Unicode hit maps, copy/save, list/table transactions, independent panes and syntax-disabled presentation",
        std::env::var_os("NO_COLOR").is_some()
    );
    Ok(())
}

fn presentation(editor: &mut Session) -> Result<()> {
    for pretty in [true, false] {
        headings(editor, pretty)?;
        containers(editor, pretty)?;
        editor.execute(Command::Markdown(M::Pretty));
    }
    Ok(())
}

fn paint(frame: &Frame) -> std::collections::BTreeMap<(usize, usize), surface::Cell> {
    if std::env::var_os("NO_COLOR").is_none() {
        return surface::draw(frame);
    }
    crossterm::style::force_color_output(false);
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let cells = surface::decode(output);
    assert!(
        cells
            .values()
            .all(|cell| cell.foreground.is_none() && cell.background.is_none())
    );
    cells
}

fn headings(editor: &mut Session, pretty: bool) -> Result<()> {
    let frame = editor.frame()?;
    let cells = paint(&frame);
    for row in 0..6 {
        let marker = cells[&(row, 7 + row)];
        assert_eq!(marker.character, if pretty { MARKERS[row] } else { '#' });
        if std::env::var_os("NO_COLOR").is_none() {
            assert_eq!(marker.foreground, Some(COLORS[row]));
            assert_eq!(cells[&(row, 9 + row)].foreground, Some(COLORS[row]));
        }
    }
    assert!(frame.rows()[6].text().contains("####### Plain paragraph"));
    assert!(frame.rows()[9].text().contains("=========="));
    assert!(frame.rows()[12].text().contains("----------"));
    if std::env::var_os("NO_COLOR").is_none() {
        assert_eq!(cells[&(8, 7)].foreground, Some(COLORS[0]));
        assert_eq!(cells[&(11, 7)].foreground, Some(COLORS[1]));
    }
    Ok(())
}

fn containers(editor: &Session, pretty: bool) -> Result<()> {
    let mut view = editor.view().clone();
    view.top = 14;
    let frame = Frame::project(
        editor.buffer()?,
        &view,
        Size {
            width: 80,
            height: 24,
        },
        "",
    )?;
    let cells = paint(&frame);
    for (row, column, glyph, literal) in [(0, 7, '•', '-'), (1, 11, '◦', '+'), (2, 15, '▪', '*')]
    {
        assert_eq!(
            cells[&(row, column)].character,
            if pretty { glyph } else { literal }
        );
    }
    assert!(frame.rows()[4].text().contains("3. Ordered"));
    view.top = 27;
    let frame = Frame::project(
        editor.buffer()?,
        &view,
        Size {
            width: 80,
            height: 24,
        },
        "",
    )?;
    assert!(frame.rows()[1].text().contains("# Literal heading"));
    assert!(frame.rows()[2].text().contains("- Literal item"));
    Ok(())
}

// Projection and hit-testing share canonical source coordinates across both
// sizes and clipping. The combining scalar itself is not a grapheme boundary.
fn geometry(editor: &Session) -> Result<()> {
    let buffer = editor.buffer()?;
    let text = source(editor);
    let coordinates = Coordinates::new(buffer)?;
    let mut view = editor.view().clone();
    for (width, height) in [(80, 24), (32, 12)] {
        for left in [0, 2, 5] {
            view.left = left;
            for needle in ["界", "e\u{301}", "`a\\|b`", "[guide]"] {
                view.point.position = coordinates.from_byte(Byte(text.find(needle).unwrap()))?;
                view.top = coordinates.to_line(view.point.position)?.row;
                geometry_point(buffer, &view, Size { width, height })?;
            }
        }
    }
    Ok(())
}

fn geometry_point(buffer: &rev::buffer::Buffer, view: &rev::view::View, size: Size) -> Result<()> {
    let frame = Frame::project(buffer, view, size, "")?;
    assert!(
        frame
            .rows()
            .iter()
            .all(|row| row.text().width() == usize::from(size.width))
    );
    if let Some(cursor) = frame.cursor() {
        let area = Area::new(0, 0, size.width, size.height - 2).unwrap();
        assert_eq!(
            view.hit(buffer, area, cursor)?.unwrap().position,
            view.point.position
        );
    }
    Ok(())
}

fn copy_and_save(root: &Path, editor: &mut Session, original: &str) -> Result<()> {
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Region(RegionCommand::Set));
    editor.execute(Command::Motion(Motion::LineEnd));
    editor.execute(Command::named("kill-ring-save").unwrap());
    assert_eq!(editor.killed_text().unwrap().to_string(), "# Level 1");
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::named("yank").unwrap());
    assert!(source(editor).ends_with("# Level 1"));
    editor.execute(Command::History(History::Undo));
    settle(editor)?;
    editor.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(root.join("workspace/native.md"))?,
        original.as_bytes()
    );
    Ok(())
}

fn split(editor: &mut Session) -> Result<()> {
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Markdown(M::Pretty));
    let selected = editor.windows().selected_id();
    for (id, view) in editor.windows().iter() {
        assert_eq!(view.pretty_markdown(), id != selected);
        let frame = Frame::project(
            editor.buffer()?,
            view,
            Size {
                width: 32,
                height: 12,
            },
            "",
        )?;
        assert!(frame.rows()[0].text().contains(if id == selected {
            "# Level 1"
        } else {
            "◉ Level 1"
        }));
    }
    Ok(())
}

fn disabled(root: &Path, profile: &str, original: &str) -> Result<()> {
    let mut editor = support::open(
        root,
        profile,
        original,
        "syntax.enabled=false\neditor.soft_wrap=true\n",
    )?;
    let frame = editor.frame()?;
    assert!(frame.rows().iter().all(|row| row.syntax().is_empty()));
    assert_eq!(paint(&frame)[&(0, 7)].character, '◉');
    geometry(&editor)?;
    assert!(!editor.buffer()?.is_modified());
    Ok(())
}
