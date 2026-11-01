use super::navigation::{keys, settle};
use super::*;
use rev::command::{Motion, Split, WindowCommand};
use std::time::{Duration, Instant};

fn modified(character: char, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(character), modifiers))
}

// Count and character-target cancellation preserve protected recovery chords,
// terminal aliases and the same component-owned prompt text in both profiles.
#[test]
fn grammar_lifetime() {
    for input in ["23", "f", "3g"] {
        let mut editor = session(VIM, "one two", 0);
        keys(&mut editor, input);
        editor.handle(modified('x', KeyModifiers::ALT));
        keys(&mut editor, "goto-line");
        assert!(
            editor
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().contains("M-x goto-line"))
        );
        editor.handle(modified('g', KeyModifiers::CONTROL));
        assert_eq!(editor.view().point.position, Scalar(0));
        keys(&mut editor, "f");
        editor.handle(modified('[', KeyModifiers::CONTROL));
        editor.handle(key(KeyCode::Right));
        settle(&mut editor);
        assert_eq!(editor.view().point.position, Scalar(1));
    }
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "3");
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(WindowCommand::Other));
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(1));
    keys(&mut editor, "2");
    editor.insert('X');
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(3));
}

// The pending grammar's independent 16-stroke limit includes a counted custom
// sequence and its character argument; the eight-chord keymap limit stays intact.
#[test]
fn stroke_bound() {
    let config = format!("{VIM}\n[keys.vim_normal]\n'q q q q q q q f' = 'modal-find-forward'");
    let mut editor = session(&config, "abcabc", 0);
    keys(&mut editor, "12345678qqqqqqqfc");
    assert!(editor.echo().contains("16 strokes"), "{}", editor.echo());
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(1));
}

// A u32-max request over a substantial rope must finish in source-bounded time,
// reach checked EOF/BOF and create no edit or count-sized command/history queue.
#[test]
fn large_source_count() {
    let source = "word,界 e\u{301}\r\n".repeat(4000);
    let mut editor = session(VIM, &source, 0);
    let start = Instant::now();
    keys(&mut editor, "4294967295w");
    assert_eq!(editor.view().point.position, Scalar(source.chars().count()));
    keys(&mut editor, "4294967295b");
    assert_eq!(editor.view().point.position, Scalar(0));
    assert!(start.elapsed() < Duration::from_secs(5));
    assert!(editor.buffer().unwrap().history().is_empty());
    assert_eq!(text(&editor), source);
}

// Emacs visual-row and word-end commands keep their previous results while
// named logical navigation uses the same native point in either profile.
#[test]
fn emacs_endpoints() {
    let mut editor = session("[tree]\nvisible = false", "abcdefgh ijklmnop\nnext", 0);
    editor.handle(Event::Resize(8, 8));
    editor.execute(Command::Scroll(rev::command::Scroll::NextLine));
    settle(&mut editor);
    assert_eq!(editor.view().cell(editor.buffer().unwrap()).unwrap().row, 0);
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Motion(Motion::ForwardWord));
    assert_eq!(editor.view().point.position, Scalar(8));
    editor.execute(Command::Motion(Motion::BackwardWord));
    assert_eq!(editor.view().point.position, Scalar(0));
}

// Explicitly spelling every factory in the supported sample must preserve
// decimal zeroes after a count, while a custom leading digit remains a binding.
#[test]
fn sample_counts() {
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        include_str!("../../docs/reference/config.toml"),
    );
    let config = format!(
        "include = ['{}']\n[editor]\ninteraction_profile = 'modal'\n[tree]\nvisible = false",
        base.display()
    );
    let source = "line\n".repeat(30);
    let mut editor = session(&config, &source, 0);
    keys(&mut editor, "20j");
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().row,
        20
    );
    keys(&mut editor, "10G");
    assert_eq!(editor.view().cell(editor.buffer().unwrap()).unwrap().row, 9);
    let config = format!("{VIM}\n[keys.vim_normal]\n'3' = 'modal-buffer-end'");
    let mut editor = session(&config, "a\nb\nc", 0);
    keys(&mut editor, "3");
    assert_eq!(editor.view().point.position, Scalar(5));
    editor.execute(Command::Motion(Motion::BufferStart));
    keys(&mut editor, "23j");
    assert_eq!(editor.view().point.position, Scalar(4));
}
