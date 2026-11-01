use super::navigation::{keys, settle};
use super::*;
use rev::command::{KillCommand, LinearCommand, Motion, Split, WindowCommand};

// Read-only admission precedes publication/state entry and preserves the shared
// entry. Copy is allowed, while every mutating family leaves history untouched.
#[test]
fn readonly_operators() {
    for input in ["dw", "cc", "D", "C", "x", "X", "r界", "p", ">>", "J"] {
        let mut editor = source_session(VIM, "one two\nnext", 0, true);
        keys(&mut editor, "yw");
        let retained = editor.killed_text().unwrap().to_string();
        let point = editor.view().point;
        keys(&mut editor, input);
        assert_eq!(text(&editor), "one two\nnext", "{input}");
        assert_eq!(editor.view().point, point, "{input}");
        assert_eq!(
            editor.killed_text().unwrap().to_string(),
            retained,
            "{input}"
        );
        assert!(editor.buffer().unwrap().history().is_empty(), "{input}");
        assert_eq!(
            editor.editing_state(),
            Some(EditingState::Normal),
            "{input}"
        );
    }
}

// A counted native replacement rebases all displayed points once, and history
// restores each participating view's own point rather than the initiator's.
#[test]
fn shared_view_history() {
    let mut editor = session(VIM, "a b c d e f g", 0);
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(WindowCommand::Other));
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Window(WindowCommand::Other));
    let before = positions(&editor);
    keys(&mut editor, "2d3w");
    assert_eq!(text(&editor), "g");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    assert!(
        editor
            .windows()
            .iter()
            .all(|(_, view)| view.cell(editor.buffer().unwrap()).is_ok())
    );
    keys(&mut editor, "u");
    assert_eq!(positions(&editor), before);
    assert_eq!(text(&editor), "a b c d e f g");
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('r'),
        KeyModifiers::CONTROL,
    )));
    settle(&mut editor);
    assert_eq!(text(&editor), "g");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
}

fn positions(editor: &Session) -> Vec<(rev::window::ViewId, Scalar)> {
    editor
        .windows()
        .iter()
        .map(|(id, view)| (id, view.point.position))
        .collect()
}

// View changes and explicit recovery retire an operator before another native
// command can consume its target. Prompt text remains component-owned.
#[test]
fn operator_ownership() {
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "d");
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(WindowCommand::Other));
    keys(&mut editor, "w");
    assert_eq!(text(&editor), "one two");
    assert_eq!(editor.view().point.position, Scalar(4));
    keys(&mut editor, "d");
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    editor.handle(Event::Paste("goto-line".into()));
    editor.handle(key(KeyCode::Enter));
    editor.handle(Event::Paste("1".into()));
    editor.handle(key(KeyCode::Enter));
    keys(&mut editor, "w");
    assert_eq!(text(&editor), "one two");
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Existing coalescence applies only to compatible source/view/shape chains.
// Native kill and yank still read/write the same entries as modal operations.
#[test]
fn shared_transfer_owner() {
    for (config, entries, retained) in [
        (VIM.to_string(), 1, "one two "),
        (format!("{VIM}\n[kill_ring]\ncoalesce = false"), 2, "two "),
    ] {
        let mut editor = session(&config, "one two three", 0);
        keys(&mut editor, "dwdw");
        assert_eq!(editor.kill_ring().len(), entries);
        assert_eq!(editor.killed_text().unwrap().to_string(), retained);
    }
    let mut editor = session(VIM, "one two three", 0);
    keys(&mut editor, "dwd99fZdw");
    assert_eq!(editor.kill_ring().len(), 2);
    assert_eq!(editor.killed_text().unwrap().to_string(), "two ");
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "yy");
    editor.execute(Command::Kill(KillCommand::Word));
    keys(&mut editor, "P");
    assert_eq!(text(&editor), "one two");
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "ywwywP");
    editor.execute(Command::Linear(LinearCommand::Pop));
    assert_eq!(text(&editor), "one one two");
}

// Empty valid interiors can enter Insert without a fabricated edit, while
// linewise delete/paste at an unterminated EOF adds no spurious blank line.
#[test]
fn empty_and_eof() {
    for (source, point, input) in [("", 0, "cc"), ("()", 1, "ci("), ("\r\n", 0, "C")] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), source);
        assert_eq!(editor.editing_state(), Some(EditingState::Insert));
        assert!(editor.buffer().unwrap().history().is_empty());
    }
    let mut editor = session(VIM, "one\ntwo", 4);
    keys(&mut editor, "ddp");
    assert_eq!(text(&editor), "one\ntwo");
    assert_eq!(editor.buffer().unwrap().history().len(), 2);
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "one\n");
}

// Explicit output bounds reject huge repeated paste before allocation and
// publication; multiply overflow and an incomplete find preserve prior transfer.
#[test]
fn rejected_expansion() {
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "yw");
    let retained = editor.killed_text().unwrap().to_string();
    for input in ["4294967295p", "65536d65536w", "d99fZ", "r"] {
        keys(&mut editor, input);
        assert_eq!(text(&editor), "one two", "{input}");
        assert!(editor.buffer().unwrap().history().is_empty(), "{input}");
        assert_eq!(
            editor.killed_text().unwrap().to_string(),
            retained,
            "{input}"
        );
        editor.handle(key(KeyCode::Esc));
    }
}

// Pair-count traversal is bounded by real enclosing delimiters, and complete
// source-sized deletion remains one native transaction even at a large count.
#[test]
fn bounded_objects() {
    let source = format!("{}界{}", "(".repeat(2000), ")".repeat(2000));
    let mut editor = session(VIM, &source, 2000);
    keys(&mut editor, "4294967295di(");
    assert_eq!(text(&editor), source);
    assert!(editor.buffer().unwrap().history().is_empty());
    keys(&mut editor, "d2000a(");
    assert_eq!(text(&editor), "");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "u");
    assert_eq!(text(&editor), source);
}

// Every explicit factory in the shipped sample preserves operator/count
// grammar. The total 16-stroke bound includes prefixes, counts and a target.
#[test]
fn sample_operators() {
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        include_str!("../../docs/reference/config.toml"),
    );
    let config = format!(
        "include = ['{}']\n[editor]\ninteraction_profile = 'modal'\n[tree]\nvisible = false",
        base.display()
    );
    let mut editor = session(&config, "a b c d e f g", 0);
    keys(&mut editor, "2d3w");
    assert_eq!(text(&editor), "g");
    let config = format!("{VIM}\n[keys.vim_normal]\n'q q q q q q q f' = 'modal-find-forward'");
    let mut editor = session(&config, "one two", 0);
    keys(&mut editor, "12345678dqqqqqqqf");
    assert!(editor.echo().contains("16 strokes"), "{}", editor.echo());
    assert_eq!(text(&editor), "one two");
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Refactoring search into a pure operator target must preserve the native
// confirmed word query even when its initial forward search has no match.
#[test]
fn failed_word_query() {
    let mut editor = session(VIM, "cat one cat", 0);
    keys(&mut editor, "/one");
    editor.handle(key(KeyCode::Enter));
    keys(&mut editor, "Gh*");
    assert_eq!(editor.view().point.position, Scalar(10));
    keys(&mut editor, "ggn");
    assert_eq!(editor.view().point.position, Scalar(3));
    assert!(editor.buffer().unwrap().history().is_empty());
}
