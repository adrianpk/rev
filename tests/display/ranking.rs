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

// Resizing clips fields independently at terminal-cell boundaries. Restoring
// space recovers complete annotations without changing the query or identity.
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
    for (case, width) in [
        ("wide", 80),
        ("compressed", 19),
        ("narrower", 18),
        ("small", 11),
        ("clipped", 8),
        ("zero", 0),
        ("restored", 80),
    ] {
        editor.handle(Event::Resize(width, 24));
        let frame = editor.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(row.text().width(), usize::from(width), "{case}");
        }
        check_completion_hint(&frame, case, width);
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

fn check_completion_hint(frame: &rev::display::Frame, case: &str, width: u16) {
    if width < 8 {
        return;
    }
    let row = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion)
        .unwrap();
    if width == 80 {
        assert!(row.text().starts_with("save-buffer  "), "{case}");
        assert!(row.text().contains("[M-界]"), "{case}: {:?}", row.text());
        assert!(row.text().contains("Save to"), "{case}");
    } else {
        assert!(!row.text().contains("save-buffer"), "{case}");
    }
}

// Filtering and scrolling reuse the complete catalog's column geometry. A
// shortened candidate set cannot pull shortcuts or descriptions to the left.
#[test]
fn stable_columns() {
    let fixture = super::config_fixture::Fixture::new();
    let config = format!(
        "minibuffer.candidate_limit=2\n[keys.editor]\n'M-j a b c d e f g'='mouse-mode'\n[ai.commands.{}]\nprompt='Review text.'\ninput='buffer'\ncontext='input'\noutput='explanation'",
        "r".repeat(64)
    );
    let startup = rev::config::prepare(&fixture.write("config.toml", &config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = rev::buffer::Buffers::with_settings(startup.snapshot).unwrap();
    let view = rev::view::View::new(buffers.get(buffers.initial()).unwrap());
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 160,
            height: 24,
        },
    )
    .unwrap();
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Paste("save".into()));
    let expected = annotation_columns(&editor.frame().unwrap());
    assert_eq!(
        expected.0, 69,
        "registered names contribute their full width"
    );
    for _ in 0..2 {
        key(&mut editor, KeyCode::Down);
    }
    assert_eq!(annotation_columns(&editor.frame().unwrap()), expected);
    let mut previous = "save";
    for query in ["save-buffer", "balance-windows", "save"] {
        replace_query(&mut editor, previous, query);
        assert_eq!(annotation_columns(&editor.frame().unwrap()), expected);
        previous = query;
    }
    editor.handle(Event::Resize(80, 24));
    let resized = annotation_columns(&editor.frame().unwrap());
    replace_query(&mut editor, previous, "balance-windows");
    assert_eq!(annotation_columns(&editor.frame().unwrap()), resized);
}

fn replace_query(editor: &mut Session, previous: &str, query: &str) {
    for _ in previous.chars() {
        key(editor, KeyCode::Backspace);
    }
    editor.handle(Event::Paste(query.into()));
}

#[track_caller]
fn annotation_columns(frame: &rev::display::Frame) -> (usize, usize) {
    let positions: std::collections::BTreeSet<_> = frame
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .filter_map(|row| {
            let start = row.text().find('[')?;
            let end = row.text().find(']')? + 1;
            let description = row.text()[end..].trim_start();
            (!description.is_empty()).then(|| (start, row.text().find(description).unwrap()))
        })
        .collect();
    assert_eq!(
        positions.len(),
        1,
        "{positions:?}: {:?}",
        frame
            .rows()
            .iter()
            .filter(|row| row.band() == Band::Completion)
            .map(|row| row.text())
            .collect::<Vec<_>>()
    );
    *positions.first().unwrap()
}

// The actual frame writer uses status_text for unselected secondary cells and
// selection colors across the row, including the separating blank cells.
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
        let hint = if selected { "[C-x C-c]" } else { "[C-x C-s]" };
        let hint_column = row.text().find(hint).unwrap();
        let description_column = row
            .text()
            .find(if selected { "Exit rev" } else { "Save to" })
            .unwrap();
        assert_eq!(hint_column, 40, "shortcut follows the catalog name field");
        assert!(
            !row.text().contains('—'),
            "descriptions need no placeholder"
        );
        for column in [
            0,
            hint_column,
            description_column - 1,
            description_column,
            79,
        ] {
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
                } else if column == hint_column || column >= description_column {
                    palette.status_text
                } else if column == 0 {
                    palette.syntax.function
                } else {
                    palette.foreground
                })
            );
        }
    }
}

// Startup shares the admitted command origin, ranking and hint resolver. A
// configured action shortcut remains effective while source-edit shortcuts do
// not claim dispatch authority over the read-only screen.
#[test]
fn startup_command_origin() {
    use rev::buffer::Buffers;
    use rev::view::View;
    for profile in ["non-modal", "modal"] {
        let f = crate::config_fixture::Fixture::new();
        let map = if profile == "modal" {
            "vim_normal"
        } else {
            "editor"
        };
        let settings=rev::config::prepare(&f.write("startup.toml", &format!("editor.interaction_profile='{profile}'\n[keys.{map}]\n'M-界'='find-file'\n'M-文'='undo'")));
        assert!(settings.diagnostic.is_none(), "{:?}", settings.diagnostic);
        let buffers = Buffers::with_settings(settings.snapshot).unwrap();
        let view = View::new(buffers.get(buffers.initial()).unwrap());
        let mut s = Session::new(
            buffers,
            view,
            Size {
                width: 100,
                height: 24,
            },
        )
        .unwrap();
        s.execute(Command::DashboardOpen);
        s.execute(Command::ExecuteExtended);
        s.handle(Event::Paste("find-file".into()));
        let frame = s.frame().unwrap();
        let row = frame
            .rows()
            .iter()
            .find(|r| r.band() == Band::Completion && r.text().contains("find-file"))
            .unwrap();
        assert!(row.text().contains("[M-界]"));
        key(&mut s, KeyCode::Enter);
        assert!(s.dashboard_visible());
        s.execute(Command::KeyboardQuit);
        assert!(s.dashboard_visible());
        s.execute(Command::ExecuteExtended);
        s.handle(Event::Paste("undo".into()));
        let frame = s.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .filter(|r| r.band() == Band::Completion && r.text().contains("undo"))
                .all(|r| !r.text().contains("[M-文]"))
        );
        s.execute(Command::KeyboardQuit);
        assert!(s.dashboard_visible());
        s.execute(Command::ExecuteExtended);
        let ranked = names(&mut s);
        assert_eq!(ranked[0], "find-file");
        s.execute(Command::KeyboardQuit);
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('界'),
            KeyModifiers::ALT,
        )));
        assert!(s.dashboard_visible());
        assert!(
            s.frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("Find file:"))
        );
    }
}
