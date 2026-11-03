use crate::contexts::{answer, control};
use crate::navigation::{keys, settle};
use crate::*;

// A characterwise Visual range remains canonical across colon and the shell
// prompt. Real tr changes exactly that range and ordinary undo restores it.
#[test]
fn visual_shell() {
    let config = format!("{VIM}\n[shell]\nprogram='/bin/sh'");
    let mut editor = session(&config, "ab cd", 0);
    keys(&mut editor, "vl:shell-filter-region");
    editor.handle(key(KeyCode::Enter));
    answer(&mut editor, "tr a-z A-Z");
    assert_eq!(text(&editor), "AB cd");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    editor.execute(Command::History(History::Undo));
    settle(&mut editor);
    assert_eq!(text(&editor), "ab cd");
}

// Both prompt cancellation routes retain the selected source and never start
// work. Normal/Visual C-u remains the native half-page motion after cancellation.
#[test]
fn shell_prompt_cancel() {
    for escape in [false, true] {
        let mut editor = session(VIM, "ab\ncd\nef", 0);
        keys(&mut editor, "vl:shell-command-on-region");
        editor.handle(key(KeyCode::Enter));
        let point = editor.view().point;
        let mark = editor.buffer().unwrap().mark();
        editor.handle(Event::Paste("printf should-not-run".into()));
        if escape {
            editor.handle(key(KeyCode::Esc));
            editor.expire_input(std::time::Instant::now() + rev::input::META_TIMEOUT);
        } else {
            control(&mut editor, 'g');
        }
        assert_eq!(editor.view().point, point);
        assert_eq!(editor.buffer().unwrap().mark(), mark);
        assert_eq!(editor.editing_state(), Some(EditingState::VisualCharacter));
        assert!(editor.remaining_input(std::time::Instant::now()).is_none());
        keys(&mut editor, "G");
        control(&mut editor, 'u');
        settle(&mut editor);
        assert_eq!(editor.view().point.position, Scalar(2));
        assert_eq!(editor.editing_state(), Some(EditingState::VisualCharacter));
        assert_eq!(text(&editor), "ab\ncd\nef");
        assert!(editor.buffer().unwrap().history().is_empty());
    }
}
