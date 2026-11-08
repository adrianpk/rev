use super::*;
use rev::command::{History, ModalCommand, Motion, RegionCommand};

fn named(editor: &mut Session, name: &str) {
    editor.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    answer(editor, name);
}
fn prefix(editor: &mut Session, number: &str) {
    named(editor, "prefix-argument");
    for digit in number.chars() {
        editor.handle(key(KeyCode::Char(digit), KeyModifiers::NONE));
    }
}

// Named shell prefixes select insertion exactly once in both profiles. A real
// isolated shell records one execution; one undo restores the unsaved source.
#[test]
fn named_insertion() {
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let (mut editor, source) = filters::source_editor(
            &f,
            "base",
            &format!("[editor]\ninteraction_profile='{profile}'"),
            false,
        );
        editor.insert('!');
        let original = text(editor.buffer().unwrap());
        prefix(&mut editor, "4");
        named(&mut editor, "shell-command");
        answer(&mut editor, "printf x >> executions; printf 'é界'");
        filters::settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), "!é界base");
        assert_eq!(fs::read_to_string(f.0.join("executions")).unwrap(), "x");
        editor.execute(Command::History(History::Undo));
        assert_eq!(text(editor.buffer().unwrap()), original);
        assert_eq!(fs::read_to_string(source).unwrap(), "base");
    }
}

// A numeric prefix on named region execution selects the existing filter once,
// even for zero/negative values. The checked source range owns one publication.
#[test]
fn named_filter() {
    for (profile, count) in [("non-modal", "0"), ("modal", "-2")] {
        let f = Fixture::new();
        let (mut editor, source) = filters::source_editor(
            &f,
            "ab界",
            &format!("[editor]\ninteraction_profile='{profile}'"),
            false,
        );
        editor.execute(Command::Region(RegionCommand::Set));
        editor.execute(Command::Motion(Motion::BufferEnd));
        prefix(&mut editor, count);
        named(&mut editor, "shell-command-on-region");
        answer(&mut editor, "printf x >> executions; LC_ALL=C tr a-z A-Z");
        filters::settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), "AB界");
        assert_eq!(editor.buffer().unwrap().history().len(), 1);
        assert_eq!(fs::read_to_string(f.0.join("executions")).unwrap(), "x");
        editor.execute(Command::History(History::Undo));
        assert_eq!(text(editor.buffer().unwrap()), "ab界");
        assert_eq!(fs::read_to_string(source).unwrap(), "ab界");
    }
}

// Existing C-u shell continuations keep precedence in ordinary and modal Insert
// input; no extra acceptance key or numeric expansion launches multiple jobs.
#[test]
fn physical_variants() {
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let (mut editor, _) = filters::source_editor(
            &f,
            "base",
            &format!("[editor]\ninteraction_profile='{profile}'"),
            false,
        );
        if profile == "modal" {
            editor.execute(Command::Modal(ModalCommand::Insert));
        }
        editor.handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
        editor.handle(key(KeyCode::Char('!'), KeyModifiers::ALT));
        answer(&mut editor, "printf x >> executions; printf '<>'");
        filters::settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), "<>base");
        assert_eq!(fs::read_to_string(f.0.join("executions")).unwrap(), "x");
    }
}
