use super::*;
use crate::filters::settle;
use crate::regions::whole;

fn control(editor: &mut Session, value: char) {
    editor.handle(key(KeyCode::Char(value), KeyModifiers::CONTROL));
}
fn meta(editor: &mut Session, value: char) {
    editor.handle(key(KeyCode::Char(value), KeyModifiers::ALT));
}
fn prompt(editor: &mut Session) -> String {
    editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text())
        .collect::<Vec<_>>()
        .join("\n")
}

// All four effective non-modal chords reach real shell handlers. Insertion and
// filtering remain ordinary one-step edits, while display leaves source intact.
#[test]
fn native_keys() {
    for (prefix, suffix, expected) in [
        (false, '!', "kept"),
        (false, '|', "kept"),
        (true, '!', "keptX"),
        (true, '|', "X"),
    ] {
        let f = Fixture::new();
        let (mut editor, _) = crate::filters::source_editor(&f, "kept", "", false);
        whole(&mut editor);
        if prefix {
            control(&mut editor, 'u');
        }
        meta(&mut editor, suffix);
        assert!(prompt(&mut editor).contains("Shell command: "));
        answer(&mut editor, "printf X");
        settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), expected);
        assert_eq!(
            editor.buffer().unwrap().history().len(),
            usize::from(prefix)
        );
        assert_eq!(output(&mut editor), "X");
    }
}

// Only admitted commands enter the bounded shell history. Cancellation and
// invalid input preserve it; history recall restores the draft and never runs it.
#[test]
fn shell_history() {
    let f = Fixture::new();
    let (mut editor, _) = editor(&f, "");
    let stamp = editor.buffer().unwrap().stamp();
    for i in 0..34 {
        start(&mut editor, &format!(": #{i}"));
        settle(&mut editor);
    }
    editor.execute(Command::ShellCommand);
    editor.handle(Event::Paste("draft".into()));
    meta(&mut editor, 'p');
    assert!(prompt(&mut editor).contains("Shell command: : #33"));
    meta(&mut editor, 'n');
    assert!(prompt(&mut editor).contains("Shell command: draft"));
    for _ in 0..40 {
        meta(&mut editor, 'p');
    }
    assert!(prompt(&mut editor).contains("Shell command: : #2"));
    control(&mut editor, 'g');
    start(&mut editor, " ");
    assert!(editor.remaining_input(Instant::now()).is_none());
    editor.execute(Command::ShellCommand);
    meta(&mut editor, 'p');
    assert!(prompt(&mut editor).contains("Shell command: : #33"));
    control(&mut editor, 'g');
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
    assert!(editor.buffer().unwrap().history().is_empty());
    assert!(editor.remaining_input(Instant::now()).is_none());
    let (mut other, _) = super::editor(&f, "");
    other.execute(Command::ShellCommand);
    meta(&mut other, 'p');
    assert!(other.echo().contains("history is empty"));
}
