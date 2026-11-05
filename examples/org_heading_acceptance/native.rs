use super::support::{self, MARKERS, Result, key, named, open, settle, source};
use crossterm::event::{Event, KeyCode};
use rev::command::{Command, LinearCommand, Motion, RegionCommand};
use rev::session::Session;
use std::path::Path;

pub(super) fn run(root: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        profile_case(root, profile)?;
    }
    Ok(())
}

fn profile_case(root: &Path, profile: &str) -> Result<()> {
    let original = format!("\u{feff}{}", support::document().replace('\n', "\r\n"));
    let mut editor = open(root, profile, &original)?;
    settle(&mut editor)?;
    key(&mut editor, KeyCode::Down);
    key(&mut editor, KeyCode::Down);
    key(&mut editor, KeyCode::Right);
    let point = editor.view().point;
    let stamp = editor.buffer()?.stamp();
    for (width, height) in [(40, 12), (80, 24), (40, 12)] {
        editor.handle(Event::Resize(width, height));
        controls(&mut editor)?;
        assert_eq!(editor.view().point, point);
        assert_eq!(editor.buffer()?.stamp(), stamp);
    }
    copy(&mut editor, profile)?;
    editor.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(root.join("workspace/native.org"))?,
        original.as_bytes()
    );
    let buffer = editor.buffer()?;
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
    assert_eq!(source(&editor), original.trim_start_matches('\u{feff}'));
    println!(
        "PASS native {profile}: 40x12/80x24/resize, source/pretty/reset, prefix point, canonical copy, BOM/CRLF source and empty history"
    );
    Ok(())
}

fn controls(editor: &mut Session) -> Result<()> {
    let cursor = editor.frame()?.cursor();
    levels(editor)?;
    named(editor, "org-pretty-headings-mode");
    assert_eq!(editor.echo(), "Org headings: source");
    let frame = editor.frame()?;
    for (row, line) in frame.rows().iter().take(8).enumerate() {
        assert!(
            line.text()
                .chars()
                .skip(7)
                .collect::<String>()
                .starts_with(&format!("{} Heading {}", "*".repeat(row + 1), row + 1))
        );
    }
    assert_eq!(
        frame.cursor(),
        cursor.map(|mut position| {
            position.column =
                7 + editor.view().cell(editor.buffer().unwrap()).unwrap().column as u16;
            position
        })
    );
    named(editor, "org-pretty-headings-mode");
    assert_eq!(editor.echo(), "Org headings: pretty");
    assert_eq!(editor.view().heading_presentation_override(), Some(true));
    named(editor, "org-reset-heading-presentation");
    assert_eq!(editor.echo(), "Org headings: pretty");
    assert_eq!(editor.view().heading_presentation_override(), None);
    assert_eq!(editor.frame()?.cursor(), cursor);
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Paste("org-pretty-headings-mode".into()));
    editor.handle(Event::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char('g'),
        crossterm::event::KeyModifiers::CONTROL,
    )));
    assert!(editor.view().pretty_headings());
    levels(editor)
}

fn levels(editor: &mut Session) -> Result<()> {
    let frame = editor.frame()?;
    for (row, marker) in MARKERS.iter().enumerate() {
        let expected = format!("{marker} Heading {}", row + 1);
        assert!(
            frame.rows()[row]
                .text()
                .chars()
                .skip(7)
                .collect::<String>()
                .starts_with(&expected)
        );
    }
    Ok(())
}

fn copy(editor: &mut Session, profile: &str) -> Result<()> {
    editor.execute(Command::Motion(Motion::BufferStart));
    if profile == "modal" {
        key(editor, KeyCode::Char('v'));
        key(editor, KeyCode::Char('j'));
        key(editor, KeyCode::Char('y'));
        // Visual from the first source star through the second line's marker.
        assert_eq!(
            editor.killed_text().unwrap().to_string(),
            "* Heading 1\r\n**"
        );
    } else {
        editor.execute(Command::Region(RegionCommand::Set));
        editor.execute(Command::Motion(Motion::ForwardChar));
        editor.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(editor.killed_text().unwrap().to_string(), "*");
    }
    settle(editor)
}
