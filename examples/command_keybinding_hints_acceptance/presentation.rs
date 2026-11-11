use crate::native::{cancel, editor, send};
use crate::support::{Result, key, source};
use crossterm::event::{Event, KeyCode};
use rev::command::Command;
use rev::command::usage::{History, Store};
use rev::config::Palette;
use rev::display::{Band, Frame};
use rev::session::Session;
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
    monochrome(&mut owner)?;
    colored_rows(&mut owner)?;
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
        geometry(&mut owner, name, width, height, hint)?;
        assert_eq!(owner.view().point, original, "{name}");
    }
    named_usage(&mut owner, &history)?;
    scroll_identity(&mut owner)?;
    bound_usage(&mut owner, &history)?;
    println!(
        "PASS native presentation: NO_COLOR reverse video, normal/selected secondary colors, Unicode cells and exact gap, complete omission, zero/resize restoration, scrolling identity, name-only matching/Tab, exactly-once named usage and direct-binding exclusion"
    );
    Ok(())
}

fn monochrome(owner: &mut Session) -> Result<()> {
    crossterm::style::force_color_output(false);
    let frame = owner.frame()?;
    let mut output = Vec::new();
    frame.write(&mut output)?;
    let output = String::from_utf8(output)?;
    assert!(output.contains("\x1b[7m") && output.contains("\x1b[27m"));
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    Ok(())
}
fn colored_rows(owner: &mut Session) -> Result<()> {
    key(owner, KeyCode::Down);
    let frame = owner.frame()?;
    secondary_colors(&frame)?;
    Ok(())
}
fn secondary_colors(frame: &Frame) -> Result<()> {
    let cells = crate::surface::draw(frame);
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
                } else if column == 0 {
                    palette.syntax.function
                } else {
                    palette.foreground
                })
            );
        }
    }
    Ok(())
}
fn geometry(owner: &mut Session, name: &str, width: u16, height: u16, hint: bool) -> Result<()> {
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
    Ok(())
}
fn named_usage(owner: &mut Session, history: &Path) -> Result<()> {
    key(owner, KeyCode::Tab);
    assert!(!history.exists(), "painting and Tab must not publish usage");
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x save-buffer "))
    );
    key(owner, KeyCode::Enter);
    owner.flush_command_usage()?;
    assert_eq!(Store::open(history)?.1, ["save-buffer"]);
    owner.execute(Command::ExecuteExtended);
    owner.handle(Event::Paste("M-界".into()));
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().trim() == "No matching commands")
    );
    cancel(owner)?;
    Ok(())
}
fn scroll_identity(owner: &mut Session) -> Result<()> {
    owner.execute(Command::ExecuteExtended);
    owner.handle(Event::Paste("kill".into()));
    for _ in 0..7 {
        key(owner, KeyCode::Down);
        owner.frame()?;
    }
    let selected = selected_name(owner)?;
    assert!(
        selected.starts_with("copy-rectangle-as-kill")
            && selected.trim_end().ends_with("[C-x r M-w]")
    );
    restore_selection(owner, &selected)?;
    Ok(())
}
fn selected_name(owner: &mut Session) -> Result<String> {
    Ok(owner
        .frame()?
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .ok_or("missing scrolled selection")?
        .text()
        .to_owned())
}
fn restore_selection(owner: &mut Session, selected: &str) -> Result<()> {
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
    key(owner, KeyCode::Tab);
    assert!(
        owner
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x copy-rectangle-as-kill "))
    );
    cancel(owner)?;
    Ok(())
}
fn bound_usage(owner: &mut Session, history: &Path) -> Result<()> {
    send(owner, "M-界")?;
    owner.flush_command_usage()?;
    assert_eq!(
        Store::open(history)?.1,
        ["save-buffer"],
        "direct hint binding must not add usage"
    );
    assert_eq!(source(owner), "abc\nsecond line\n");
    Ok(())
}
