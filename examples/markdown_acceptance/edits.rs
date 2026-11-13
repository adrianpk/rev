use super::support::{self, Result, settle, source};
use crossterm::event::Event;
use rev::command::{Command, History, MarkdownCommand as M, Motion};
use rev::session::Session;

pub(super) fn run(editor: &mut Session) -> Result<()> {
    for (needle, action, expected) in [
        ("Ordered", M::ContinueItem, "4. "),
        ("Next", M::IndentItem, "   4. Next"),
        ("Child", M::OutdentItem, "+ Child"),
        ("Name", M::TableAlign, "| Name"),
        ("界", M::TableInsertRow, "|      "),
        ("界", M::TableDeleteRow, "| Name"),
        ("Value", M::TableInsertColumn, "---"),
        ("Value", M::TableDeleteColumn, "Name"),
    ] {
        transaction(editor, needle, action, expected)?;
    }
    navigation(editor)?;
    create(editor)?;
    Ok(())
}

// Every action must change source once, undo exactly, redo the same bytes and
// undo again. Merely finding an expected glyph cannot satisfy this workflow.
fn transaction(editor: &mut Session, needle: &str, action: M, expected: &str) -> Result<()> {
    support::point(editor, needle);
    let before = source(editor);
    let history = editor.buffer()?.history().len();
    editor.execute(Command::Markdown(action));
    settle(editor)?;
    let changed = source(editor);
    assert_ne!(changed, before, "{action:?}: {}", editor.echo());
    assert!(changed.contains(expected), "{action:?}: {changed:?}");
    assert_eq!(editor.buffer()?.history().len(), history + 1);
    editor.execute(Command::History(History::Undo));
    settle(editor)?;
    assert_eq!(source(editor), before);
    editor.execute(Command::History(History::Redo));
    settle(editor)?;
    assert_eq!(source(editor), changed);
    editor.execute(Command::History(History::Undo));
    settle(editor)
}

fn navigation(editor: &mut Session) -> Result<()> {
    support::point(editor, "Name");
    editor.execute(Command::Markdown(M::TableAlign));
    settle(editor)?;
    let aligned = source(editor);
    let history = editor.buffer()?.history().len();
    for action in [M::TableNextField, M::TablePreviousField, M::TableNextRow] {
        let point = editor.view().point;
        editor.execute(Command::Markdown(action));
        settle(editor)?;
        assert_ne!(editor.view().point, point, "{action:?} did not navigate");
        assert_eq!(source(editor), aligned);
        assert_eq!(editor.buffer()?.history().len(), history);
    }
    assert!(aligned.contains("`a\\|b`"));
    editor.execute(Command::History(History::Undo));
    settle(editor)
}

fn create(editor: &mut Session) -> Result<()> {
    editor.execute(Command::Motion(Motion::BufferEnd));
    let before = source(editor);
    editor.execute(Command::Markdown(M::TableCreate));
    editor.handle(Event::Paste("2x1".into()));
    support::key(editor, crossterm::event::KeyCode::Enter);
    settle(editor)?;
    assert!(source(editor).contains("|     |     |"));
    editor.execute(Command::History(History::Undo));
    settle(editor)?;
    assert_eq!(source(editor), before);
    Ok(())
}
