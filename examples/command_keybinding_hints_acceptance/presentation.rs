use crate::native::{cancel, editor, send};
use crate::support::{Result, key, source};
use crossterm::event::{Event, KeyCode};
use rev::command::Command;
use rev::command::usage::{History, Store};
use rev::config::Palette;
use rev::display::Band;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

// Actual output bytes protect monochrome selection, then actual cell colors
// protect secondary text and selected foreground/background across both fields.
pub(super) fn run(root: &Path) -> Result<()> {
    let config =
        "minibuffer.candidate_limit=2\n[keys.editor]\n'C-x C-s'='newline'\n'M-界'='save-buffer'";
    let history = root.join("history.toml");
    let mut owner = editor(root, config)?.with_command_usage(History::open(&history));
    owner.execute(Command::ExecuteExtended);
    owner.handle(Event::Paste("save-buffer".into()));
    let original = owner.view().point;
    crossterm::style::force_color_output(false);
    let frame = owner.frame()?;
    let mut output = Vec::new();
    frame.write(&mut output)?;
    let output = String::from_utf8(output)?;
    assert!(output.contains("\x1b[7m") && output.contains("\x1b[27m"));
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    key(&mut owner, KeyCode::Down);
    let frame = owner.frame()?;
    let cells = crate::surface::draw(&frame);
    let palette = Palette::default();
    for selected in [false, true] {
        let row = frame
            .rows()
            .iter()
            .find(|row| row.band() == Band::Completion && row.selection().is_some() == selected)
            .ok_or("missing colored row")?;
        let hint_column = row.text().find('[').ok_or("missing shortcut field")?;
        assert!(hint_column < 40, "shortcut must stay near its command");
        assert!(!row.text().contains('—'), "missing descriptions stay blank");
        for column in [0, hint_column, 78] {
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
                } else if column == hint_column {
                    palette.status_text
                } else {
                    palette.foreground
                })
            );
        }
    }
    cancel(&mut owner)?;
    owner.execute(Command::ExecuteExtended);
    owner.handle(Event::Paste("save-buffer".into()));
    for (name, width, height, hint) in [
        ("wide", 80, 24, true),
        ("exact", 19, 24, true),
        ("one-short", 18, 24, false),
        ("name-only", 11, 24, false),
        ("clipped", 8, 24, false),
        ("zero", 0, 0, false),
        ("restored", 80, 24, true),
    ] {
        owner.handle(Event::Resize(width, height));
        let frame = owner.frame()?;
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width)),
            "{name}"
        );
        if width >= 8 {
            let row = frame
                .rows()
                .iter()
                .find(|row| row.band() == Band::Completion)
                .ok_or("missing geometry row")?;
            assert_eq!(row.text().contains("M-界"), hint, "{name}");
            if hint {
                assert!(
                    row.text().starts_with("save-buffer  ")
                        && row.text().trim_end().ends_with("[M-界]")
                );
            } else {
                assert!(!row.text().contains("M-"), "{name}: partial shortcut");
            }
        }
        assert_eq!(owner.view().point, original, "{name}");
    }
    key(&mut owner, KeyCode::Tab);
    assert!(!history.exists(), "painting and Tab must not publish usage");
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x save-buffer "))
    );
    key(&mut owner, KeyCode::Enter);
    owner.flush_command_usage()?;
    assert_eq!(Store::open(&history)?.1, ["save-buffer"]);
    owner.execute(Command::ExecuteExtended);
    owner.handle(Event::Paste("M-界".into()));
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().trim() == "No matching commands")
    );
    cancel(&mut owner)?;
    owner.execute(Command::ExecuteExtended);
    owner.handle(Event::Paste("kill".into()));
    for _ in 0..7 {
        key(&mut owner, KeyCode::Down);
        owner.frame()?;
    }
    let selected = owner
        .frame()?
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .ok_or("missing scrolled selection")?
        .text()
        .to_owned();
    assert!(
        selected.starts_with("copy-rectangle-as-kill")
            && selected.trim_end().ends_with("[C-x r M-w]")
    );
    owner.handle(Event::Resize(0, 0));
    owner.frame()?;
    owner.handle(Event::Resize(80, 24));
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.selection().is_some() && row.text() == selected)
    );
    key(&mut owner, KeyCode::Tab);
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x copy-rectangle-as-kill "))
    );
    cancel(&mut owner)?;
    send(&mut owner, "M-界")?;
    owner.flush_command_usage()?;
    assert_eq!(
        Store::open(&history)?.1,
        ["save-buffer"],
        "direct hint binding must not add usage"
    );
    assert_eq!(source(&owner), "abc\nsecond line\n");
    println!(
        "PASS native presentation: NO_COLOR reverse video, normal/selected secondary colors, Unicode cells and exact gap, complete omission, zero/resize restoration, scrolling identity, name-only matching/Tab, exactly-once named usage and direct-binding exclusion"
    );
    Ok(())
}
