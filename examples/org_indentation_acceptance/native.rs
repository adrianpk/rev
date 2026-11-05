use crate::support::{self, Result, key, named, open, settle, source};
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::command::{Command, History, LinearCommand, Motion, RegionCommand};
use rev::coordinates::{Cell, Scalar};
use rev::session::Session;
use std::path::Path;

const DOCUMENT: &str = "** Child\nabcdefghijklmno\ntail\n";

pub(super) fn run(root: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(root, profile)?;
    }
    Ok(())
}

pub(super) fn prepare(editor: &mut Session) -> Result<()> {
    named(editor, "gutter-toggle-line-numbers");
    named(editor, "bookmark-markers-mode");
    named(editor, "org-indent-enable");
    settle(editor)
}

fn journey(root: &Path, profile: &str) -> Result<()> {
    let mut editor = open(root, profile, DOCUMENT)?;
    prepare(&mut editor)?;
    editor.handle(Event::Resize(12, 12));
    settle(&mut editor)?;
    enabled(&mut editor)?;
    controls(&mut editor)?;
    pointer(&mut editor)?;
    editing(&mut editor)?;
    copied(&mut editor)?;
    saved(&mut editor, root)?;
    println!(
        "PASS native {profile}: toggle/enable/disable, wrap, virtual-margin pointer, source copy, edit/undo/redo, exact saved bytes and no idle deadline"
    );
    Ok(())
}

fn saved(editor: &mut Session, root: &Path) -> Result<()> {
    editor.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(root.join("workspace/native.org"))?,
        DOCUMENT.as_bytes()
    );
    assert_eq!(source(editor), DOCUMENT);
    assert!(editor.buffer()?.history().is_empty());
    assert!(!editor.buffer()?.is_modified());
    assert!(editor.remaining_input(std::time::Instant::now()).is_none());
    Ok(())
}

fn enabled(editor: &mut Session) -> Result<()> {
    let frame = editor.frame()?;
    assert_eq!(
        frame.rows()[0].text().trim_end(),
        format!("  {} Child", support::MARKERS[1])
    );
    assert_eq!(frame.rows()[1].text().trim_end(), "    abcdefgh");
    assert_eq!(frame.rows()[2].text().trim_end(), "    ijklmno");
    Ok(())
}

fn controls(editor: &mut Session) -> Result<()> {
    policy_controls(editor)?;
    wrapping(editor)
}

fn policy_controls(editor: &mut Session) -> Result<()> {
    let stamp = editor.buffer()?.stamp();
    for name in [
        "org-indent-disable",
        "org-indent-disable",
        "org-indent-mode",
    ] {
        named(editor, name);
        settle(editor)?;
        assert_eq!(editor.buffer()?.stamp(), stamp);
        if name == "org-indent-disable" {
            assert!(!editor.buffer()?.org_indentation());
            assert_eq!(editor.frame()?.rows()[1].text().trim_end(), "abcdefghijkl");
        }
    }
    enabled(editor)
}

fn wrapping(editor: &mut Session) -> Result<()> {
    named(editor, "visual-line-mode");
    settle(editor)?;
    assert!(!editor.buffer()?.soft_wrap());
    assert_eq!(editor.frame()?.rows()[2].text().trim_end(), "    tail");
    named(editor, "visual-line-mode");
    settle(editor)?;
    enabled(editor)
}

fn pointer(editor: &mut Session) -> Result<()> {
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        editor.handle(Event::Mouse(MouseEvent {
            kind,
            column: 0,
            row: 2,
            modifiers: KeyModifiers::NONE,
        }));
    }
    assert_eq!(editor.view().point.position, Scalar(17));
    assert_eq!(
        editor.view().cell(editor.buffer()?)?,
        Cell { row: 1, column: 8 }
    );
    Ok(())
}

fn editing(editor: &mut Session) -> Result<()> {
    editor.insert('X');
    settle(editor)?;
    assert!(source(editor).contains("abcdefghXijklmno"));
    for history in [History::Undo, History::Redo, History::Undo] {
        editor.execute(Command::History(history));
        settle(editor)?;
        assert!(editor.buffer()?.org_indentation());
    }
    assert_eq!(source(editor), DOCUMENT);
    Ok(())
}

fn copied(editor: &mut Session) -> Result<()> {
    editor.execute(Command::Region(RegionCommand::Set));
    for _ in 0..2 {
        editor.execute(Command::Motion(Motion::ForwardChar));
    }
    editor.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(
        editor
            .killed_text()
            .ok_or("missing source copy")?
            .to_string(),
        "ij"
    );
    key(editor, KeyCode::Esc);
    editor.execute(Command::KeyboardQuit);
    settle(editor)
}
