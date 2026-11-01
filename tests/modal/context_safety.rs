use super::completion::source;
use super::{control, *};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use rev::command::{CompletionCommand, TreeCommand};
use rev::input::META_TIMEOUT;
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
    let now = Instant::now();
    editor.handle_at(key(KeyCode::Esc), now);
    editor.handle_at(key(KeyCode::Esc), now + Duration::from_millis(1));
    assert!(editor.tree().focused());
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert_eq!(text(&editor), "source");
}

// An isolated component Escape cancels only in the modal profile. Native
// cancellation restores a search origin and never edits the source or history.
#[test]
fn escape_components() {
    for (profile, config, expected) in [
        ("modal", VIM, "Quit"),
        ("non-modal", "[tree]\nvisible = false", "Escape"),
    ] {
        for (name, draft) in [
            ("execute-extended-command", "save"),
            ("goto-line", "42"),
            ("find-file", "no-such"),
            ("switch-to-buffer", "other"),
            ("list-buffers", ""),
            ("isearch-forward", "three"),
            ("query-replace", "one"),
        ] {
            let mut editor = session(config, "one\ntwo\nthree", 1);
            editor.execute(Command::named(name).unwrap());
            if !draft.is_empty() {
                editor.handle(Event::Paste(draft.into()));
            }
            settle(&mut editor);
            let now = Instant::now();
            editor.handle_at(key(KeyCode::Esc), now);
            editor.expire_input(now + META_TIMEOUT);
            assert_eq!(editor.echo(), expected, "{profile}: {name}");
            assert_eq!(text(&editor), "one\ntwo\nthree", "{profile}: {name}");
            assert!(editor.buffer().unwrap().history().is_empty());
            if profile == "modal" {
                assert_eq!(editor.view().point.position, Scalar(1), "{name}");
                assert_eq!(editor.editing_state(), Some(EditingState::Normal));
            }
        }
    }
}

// Cancellation is dispatched before a key arriving at the deadline. That fresh
// key belongs to Normal editing, not the retired prompt or a stale Meta chord.
#[test]
fn escape_late_key() {
    let mut editor = session(VIM, "source", 0);
    keys(&mut editor, ":save");
    let now = Instant::now();
    editor.handle_at(key(KeyCode::Esc), now);
    editor.handle_at(key(KeyCode::Char('x')), now + META_TIMEOUT);
    assert_eq!(text(&editor), "ource");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
}

// Escape cancels a pending save/quit decision without accepting discard,
// marking the dirty buffer clean, writing it, or exiting the session.
#[test]
fn escape_quit_decision() {
    let mut editor = session(VIM, "source", 0);
    editor.insert('X');
    editor.execute(Command::Quit);
    assert!(editor.echo().contains("n=discard"));
    let now = Instant::now();
    editor.handle_at(key(KeyCode::Esc), now);
    assert_ne!(
        editor.expire_input(now + META_TIMEOUT),
        rev::session::Action::Exit
    );
    assert_eq!(editor.echo(), "Quit");
    assert_eq!(text(&editor), "Xsource");
    assert!(editor.buffer().unwrap().is_modified());
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
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
