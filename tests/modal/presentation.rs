use super::*;
use rev::display::Band;

// State is mandatory status context even when buffer identity/access cannot fit.
// Resizing and state presentation never mutate source, revision or history.
#[test]
fn visible_state() {
    for width in [3, 5, 6, 7, 8, 20, 80] {
        let mut editor = session(VIM, "abc", 0);
        editor.handle(Event::Resize(width, 8));
        for (entry, label) in [(None, "NORMAL"), (Some('i'), "INSERT")] {
            if let Some(entry) = entry {
                editor.handle(key(KeyCode::Char(entry)));
            }
            let frame = editor.frame().unwrap();
            let status = frame
                .rows()
                .iter()
                .find(|row| row.band() == Band::Status)
                .unwrap();
            let expected = if width < 6 { &label[..1] } else { label };
            assert!(
                status.text().starts_with(expected),
                "{width}: {:?}",
                status.text()
            );
            assert_eq!(text(&editor), "abc");
            assert!(editor.buffer().unwrap().history().is_empty());
        }
    }
}

// Refused line opening must preserve state and cursor as well as document and
// history; entering Insert alone does not authorize a read-only native edit.
#[test]
fn readonly_entry() {
    let mut editor = source_session(VIM, "abc", 1, true);
    for entry in ['o', 'O'] {
        editor.handle(key(KeyCode::Char(entry)));
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        assert_eq!(editor.view().point.position, Scalar(1));
        assert_eq!(text(&editor), "abc");
        assert!(editor.buffer().unwrap().history().is_empty());
    }
    editor.handle(key(KeyCode::Char('i')));
    editor.handle(Event::Paste("refused".into()));
    assert_eq!(text(&editor), "abc");
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Opening a logical line retains CRLF and one-step undo. Whitespace navigation
// must not split a combining cluster whose leading scalar happens to be a space.
#[test]
fn source_boundaries() {
    let config = format!("{VIM}\n[files]\nnewline = 'crlf'");
    for (name, entry, original, cursor, expected) in [
        ("below CRLF", 'o', "one\r\ntwo", 5, "one\r\n\r\ntwo"),
        ("above CRLF", 'O', "one\r\ntwo", 0, "\r\none\r\ntwo"),
        ("space cluster", 'I', " \u{301}x", 0, " \u{301}x"),
    ] {
        let mut editor = session(&config, original, 0);
        editor.handle(key(KeyCode::Char(entry)));
        assert_eq!(
            editor.editing_state(),
            Some(EditingState::Insert),
            "{name}: {}",
            editor.echo()
        );
        assert_eq!(editor.view().point.position, Scalar(cursor), "{name}");
        assert_eq!(text(&editor), expected, "{name}");
        if entry != 'I' {
            editor.execute(Command::History(History::Undo));
            assert_eq!(text(&editor), original, "{name}");
        }
    }
}
