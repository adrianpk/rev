use super::navigation::{keys, settle};
use super::*;
use rev::command::{Motion, editing::Action as EditAction};

fn escape(editor: &mut Session) {
    editor.handle(key(KeyCode::Esc));
    settle(editor);
}

fn answer(editor: &mut Session, value: &str) {
    editor.handle(Event::Paste(value.into()));
    editor.handle(key(KeyCode::Enter));
    settle(editor);
}

// Replace has its own native factory and scoped overrides. The exhaustive
// sample admits R/dot, preserves inactive-profile isolation and shows REPLACE.
#[test]
fn replacement_maps() {
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        include_str!("../../docs/reference/config.toml"),
    );
    for config in [
        format!(
            "{VIM}\n[keys.editor]\n'C-f' = 'delete-char'\n[keys.vim_replace]\n'M-j' = 'newline'"
        ),
        format!(
            "include = ['{}']\n{VIM}\n[keys.vim_replace]\n'M-j' = 'newline'",
            base.display()
        ),
    ] {
        let mut editor = session(&config, "abcd", 0);
        keys(&mut editor, "RX");
        let frame = editor.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .any(|r| r.band() == rev::display::Band::Status && r.text().starts_with("REPLACE"))
        );
        editor.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL,
        )));
        keys(&mut editor, "Y");
        assert_eq!(text(&editor), "XbYd");
        editor.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::ALT,
        )));
        assert_eq!(text(&editor), "XbY\nd");
        escape(&mut editor);
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    }
}

// A block paste retains actual row data and logical-grapheme policy. Later
// linear copy changes shared retained text without changing dot's block payload.
#[test]
fn repeat_block_paste() {
    let mut editor = session(VIM, "abcd\nefgh\nijkl", 0);
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    )));
    keys(&mut editor, "jlyP");
    assert_eq!(text(&editor), "abcd\neabfgh\niefjkl");
    keys(&mut editor, "gg0yw");
    let before = text(&editor);
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "ababcd\nefeabfgh\niefjkl");
    keys(&mut editor, "u");
    assert_eq!(text(&editor), before);
}

// A non-repeatable native replacement publishes normally and retires an old
// record. Dot cannot replay query text, consent or a partial command workflow.
#[test]
fn repeat_prompt_boundary() {
    let mut editor = session(VIM, "abcd", 0);
    keys(&mut editor, "x");
    editor.execute(Command::QueryReplace);
    answer(&mut editor, "b");
    answer(&mut editor, "Y");
    editor.handle(key(KeyCode::Char('y')));
    settle(&mut editor);
    assert_eq!(text(&editor), "Ycd");
    let history = editor.buffer().unwrap().history().len();
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "Ycd");
    assert_eq!(editor.buffer().unwrap().history().len(), history);
}

// Cold counted repeat uses the unchanged native layout queue. No early edit
// occurs, and later typed input executes only after the complete repeated change.
#[test]
fn deferred_repeat() {
    let source = "abcdefgh ijklmnop\n".repeat(4000);
    let mut editor = session(VIM, &source, 0);
    keys(&mut editor, "d");
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('f'),
        KeyModifiers::CONTROL,
    )));
    settle(&mut editor);
    let before = text(&editor);
    assert!(before.len() < source.len());
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    editor.handle(Event::Resize(12, 10));
    let stamp = editor.view().point.stamp;
    let mut request = rev::command::Editing::new(EditAction::Repeat);
    request.count = Some(u32::MAX);
    editor.execute(Command::Edit(request));
    assert_eq!(editor.view().point.stamp, stamp);
    editor.handle(key(KeyCode::Char('i')));
    editor.handle(key(KeyCode::Char('X')));
    settle(&mut editor);
    assert_eq!(text(&editor), "X");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    assert_eq!(editor.buffer().unwrap().history().len(), 3);
    escape(&mut editor);
    keys(&mut editor, "uu");
    assert_eq!(text(&editor), before);
    editor.execute(Command::Motion(Motion::BufferStart));
}
