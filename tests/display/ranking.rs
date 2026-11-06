use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::display::Band;
use rev::session::Session;
use rev::view::Size;

fn key(editor: &mut Session, code: KeyCode) {
    editor.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

fn names(editor: &mut Session) -> Vec<String> {
    editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim().to_owned())
        .collect()
}

// Ranked identity survives repeated paint, narrow/zero-row layouts and growing
// again. Tab and Return still act on the chosen command, even after suspension.
#[test]
fn ranked_resize() {
    let original = Size {
        width: 80,
        height: 24,
    };
    let mut editor = Session::empty(original).unwrap();
    for name in ["undo", "undo-redo"] {
        editor.execute(Command::ExecuteExtended);
        editor.handle(Event::Paste(name.into()));
        key(&mut editor, KeyCode::Enter);
    }
    let stamp = editor.buffer().unwrap().stamp();
    editor.execute(Command::ExecuteExtended);
    assert_eq!(&names(&mut editor)[..2], ["undo-redo", "undo"]);
    key(&mut editor, KeyCode::Down);
    for (case, size) in [
        (
            "narrow",
            Size {
                width: 20,
                height: 8,
            },
        ),
        (
            "suspended",
            Size {
                width: 4,
                height: 2,
            },
        ),
        ("restored", original),
    ] {
        editor.handle(Event::Resize(size.width, size.height));
        let first = names(&mut editor);
        assert_eq!(names(&mut editor), first, "{case}");
        assert_eq!(editor.buffer().unwrap().stamp(), stamp, "{case}");
    }
    let frame = editor.frame().unwrap();
    let selected = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .unwrap();
    assert_eq!(selected.text().trim(), "undo");
    key(&mut editor, KeyCode::Tab);
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x undo "))
    );
    key(&mut editor, KeyCode::Enter);
    editor.execute(Command::ExecuteExtended);
    assert_eq!(&names(&mut editor)[..2], ["undo", "undo-redo"]);
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
}
