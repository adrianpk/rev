use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::display::Band;
use rev::session::Session;
use rev::view::Size;

fn key(editor: &mut Session, code: KeyCode) {
    editor.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

fn names(editor: &mut Session) -> Vec<String> {
    editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim().split("  ").next().unwrap().to_owned())
        .collect()
}

// Ranked identity survives repeated paint, narrow/zero-row layouts and growing
// again. Tab and Return still act on the chosen command, even after suspension.
#[test]
fn ranked_resize() {
    let original = Size {
        width: 80,
        height: 24,
    };
    let mut editor = Session::empty(original).unwrap();
    for name in ["undo", "undo-redo"] {
        editor.execute(Command::ExecuteExtended);
        editor.handle(Event::Paste(name.into()));
        key(&mut editor, KeyCode::Enter);
    }
    let stamp = editor.buffer().unwrap().stamp();
    editor.execute(Command::ExecuteExtended);
    assert_eq!(&names(&mut editor)[..2], ["undo-redo", "undo"]);
    key(&mut editor, KeyCode::Down);
    for (case, size) in [
        (
            "narrow",
            Size {
                width: 20,
                height: 8,
            },
        ),
        (
            "suspended",
            Size {
                width: 4,
                height: 2,
            },
        ),
        ("restored", original),
    ] {
        editor.handle(Event::Resize(size.width, size.height));
        let first = names(&mut editor);
        assert_eq!(names(&mut editor), first, "{case}");
        assert_eq!(editor.buffer().unwrap().stamp(), stamp, "{case}");
    }
    let frame = editor.frame().unwrap();
    let selected = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .unwrap();
    assert_eq!(selected.text().split_whitespace().next(), Some("undo"));
    key(&mut editor, KeyCode::Tab);
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x undo "))
    );
    key(&mut editor, KeyCode::Enter);
    editor.execute(Command::ExecuteExtended);
    assert_eq!(&names(&mut editor)[..2], ["undo", "undo-redo"]);
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
}

// Complete Unicode shortcuts appear only after a full name and two cells;
// shrinking and restoring the frame never edits the query or clips a hint.
#[test]
fn hint_geometry() {
    use rev::buffer::Buffers;
    use rev::view::View;
    use unicode_width::UnicodeWidthStr;
    let fixture = super::config_fixture::Fixture::new();
    let startup = rev::config::prepare(&fixture.write(
        "config.toml",
        "[keys.editor]\n'C-x C-s'='newline'\n'M-界'='save-buffer'",
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Paste("save-buffer".into()));
    let stamp = editor.buffer().unwrap().stamp();
    for (case, width, visible) in [
        ("wide", 80, true),
        ("exact", 17, true),
        ("one short", 16, false),
        ("name only", 11, false),
        ("clipped", 8, false),
        ("zero", 0, false),
        ("restored", 80, true),
    ] {
        editor.handle(Event::Resize(width, 24));
        let frame = editor.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(row.text().width(), usize::from(width), "{case}");
        }
        if width >= 8 {
            let row = frame
                .rows()
                .iter()
                .find(|row| row.band() == Band::Completion)
                .unwrap();
            assert_eq!(
                row.text().contains("M-界"),
                visible,
                "{case}: {:?}",
                row.text()
            );
            if visible {
                assert!(row.text().starts_with("save-buffer  "), "{case}");
                assert!(row.text().ends_with("M-界"), "{case}");
            } else {
                assert!(!row.text().contains("M-"), "{case}");
            }
        }
        assert_eq!(editor.buffer().unwrap().stamp(), stamp, "{case}");
    }
    key(&mut editor, KeyCode::Tab);
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x save-buffer "))
    );
    key(&mut editor, KeyCode::Enter);
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().starts_with("Write file: "))
    );
    editor.execute(Command::KeyboardQuit);
    editor.execute(Command::ExecuteExtended);
    assert_eq!(names(&mut editor)[0], "save-buffer");
}

// The actual frame writer uses status_text for unselected secondary cells and
// selection colors across both fields, including the separating blank cells.
#[test]
fn hint_colors() {
    let mut editor = Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap();
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Paste("save-buffer".into()));
    key(&mut editor, KeyCode::Down);
    let frame = editor.frame().unwrap();
    let cells = super::surface::draw(&frame);
    let palette = rev::config::Palette::default();
    for selected in [false, true] {
        let row = frame
            .rows()
            .iter()
            .find(|row| row.band() == Band::Completion && row.selection().is_some() == selected)
            .unwrap();
        assert!(
            row.text()
                .ends_with(if selected { "C-x C-c" } else { "C-x C-s" })
        );
        for column in [0, 40, 79] {
            let cell = cells[&(usize::from(row.row()), column)];
            assert_eq!(
                cell.background,
                Some(if selected {
                    palette.selection
                } else {
                    palette.background
                })
            );
            assert_eq!(
                cell.foreground,
                Some(if selected {
                    palette.selection_text
                } else if column == 79 {
                    palette.status_text
                } else {
                    palette.foreground
                })
            );
        }
    }
}
