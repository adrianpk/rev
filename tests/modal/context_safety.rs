use super::completion::source;
use super::{control, *};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use rev::command::{CompletionCommand, TreeCommand};
use std::time::{Duration, Instant};

// Editing Escape is immediate, while a selector owns Escape/Meta timing and
// literal pasted punctuation. Protected M-x/C-g and tree input retain priority.
#[test]
fn selector_input_boundary() {
    let mut editor = session(VIM, "source", 0);
    keys(&mut editor, "i");
    let now = Instant::now();
    editor.handle_at(key(KeyCode::Esc), now);
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    keys(&mut editor, "v:");
    editor.handle_at(key(KeyCode::Esc), now);
    editor.handle_at(key(KeyCode::Char('x')), now + Duration::from_millis(1));
    editor.handle(Event::Paste("literal:/".into()));
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("M-x literal:/"))
    );
    control(&mut editor, 'g');
    assert_eq!(editor.editing_state(), Some(EditingState::VisualCharacter));
    assert_eq!(text(&editor), "source");
    editor.handle(key(KeyCode::Esc));
    editor.execute(Command::Tree(TreeCommand::Focus));
    keys(&mut editor, ":i");
    assert!(editor.tree().focused());
    assert!(
        !editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("M-x "))
    );
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("M-x "))
    );
    control(&mut editor, 'g');
    assert!(!editor.tree().focused());
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert_eq!(text(&editor), "source");
}

// Native pointer focus retires an Insert popup before another pane can inherit
// it. Bracketed paste in Insert publishes text without automatic suggestions.
#[test]
fn completion_pointer_boundary() {
    let (_fixture, mut editor) = source(
        "fn cfg_value() {}\nfn main() { cfg| }\n",
        "[mouse]\nenabled = true",
    );
    editor.execute(Command::Split(Split::Right));
    let right = editor.windows().order()[1];
    keys(&mut editor, "i");
    editor.execute(Command::Completion(CompletionCommand::AtPoint));
    settle(&mut editor);
    assert!(editor.completion().is_some());
    let frame = editor.frame().unwrap();
    let row = frame.rows()[0].text();
    let at = row.rfind("fn cfg_value").unwrap();
    let column = unicode_width::UnicodeWidthStr::width(&row[..at]) as u16;
    editor.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row: 0,
        modifiers: KeyModifiers::NONE,
    }));
    settle(&mut editor);
    assert_eq!(editor.windows().selected_id(), right);
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert!(editor.completion().is_none());
    keys(&mut editor, "i");
    editor.handle(Event::Paste(":/cfg".into()));
    settle(&mut editor);
    assert!(text(&editor).starts_with(":/cfgfn"));
    assert!(editor.completion().is_none());
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
}
