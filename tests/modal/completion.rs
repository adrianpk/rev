use super::{answer, control};
use crate::navigation::{keys, settle};
use crate::*;
use rev::command::{CompletionCommand as C, Motion};
use std::time::{Duration, Instant};

pub(super) fn source(document: &str, settings: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let marker = document.find('|').unwrap();
    let position = document[..marker].chars().count();
    let path = fixture.write("source.rs", &document.replace('|', ""));
    let config = fixture.write("config.toml", &format!("{VIM}\n{settings}"));
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    view.point.position = Scalar(position);
    (
        fixture,
        Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap(),
    )
}

const SOURCE: &str = "fn cfg_value() {}\nfn main() { c| }\n";
const AUTO: &str = "[completion]\ndelay_ms = 1";

// Automatic completion admits only Insert/Replace source input. Native popup
// kinds, source colors, aliases, held Return and atomic undo remain unchanged.
#[test]
fn modal_auto_completion() {
    for entry in ['i', 'R'] {
        let document = if entry == 'R' {
            SOURCE.replace("c| }", "c|_ }")
        } else {
            SOURCE.to_owned()
        };
        let (_fixture, mut editor) = source(&document, AUTO);
        keys(&mut editor, &entry.to_string());
        keys(&mut editor, "f");
        let before = text(&editor);
        let state = editor.editing_state();
        let history = editor.buffer().unwrap().history().len();
        let completion = editor.completion().unwrap();
        assert_eq!(completion.candidates()[0].text(), "cfg_value");
        assert_eq!(
            completion.candidates()[0].kind(),
            rev::session::CompletionKind::Function
        );
        let frame = editor.frame().unwrap();
        assert!(frame.rows().iter().any(|row| !row.syntax().is_empty()));
        control(&mut editor, 'i');
        settle(&mut editor);
        assert!(text(&editor).contains("cfg_value }"));
        assert_eq!(editor.editing_state(), state);
        assert_eq!(editor.buffer().unwrap().history().len(), history + 1);
        editor.handle(Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Repeat,
            state: crossterm::event::KeyEventState::NONE,
        }));
        assert!(text(&editor).contains("cfg_value }"));
        editor.execute(Command::History(History::Undo));
        assert_eq!(text(&editor), before);
    }
}

// Editing-state transitions retire queued automatic work and popup requests.
// Popup Escape is local first; a later Escape performs the Normal transition.
#[test]
fn modal_completion_cancel() {
    for entry in ['i', 'R'] {
        let (_fixture, mut editor) = source(SOURCE, AUTO);
        keys(&mut editor, &entry.to_string());
        let now = Instant::now();
        editor.handle_at(key(KeyCode::Char('f')), now);
        editor.handle_at(key(KeyCode::Esc), now + Duration::from_micros(1));
        settle(&mut editor);
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        assert!(editor.completion().is_none());
        keys(&mut editor, &entry.to_string());
        editor.execute(Command::Completion(C::AtPoint));
        settle(&mut editor);
        assert!(editor.completion().is_some());
        editor.handle(key(KeyCode::Esc));
        assert_eq!(
            editor.editing_state().unwrap().label(),
            if entry == 'i' { "INSERT" } else { "REPLACE" }
        );
        assert!(editor.completion().is_none());
        editor.handle(key(KeyCode::Esc));
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        settle(&mut editor);
        assert!(editor.completion().is_none());
    }
}

// Normal/Visual input never schedules automatic suggestions, even when a
// configured modified editing action publishes text. Manual native invocation works.
#[test]
fn modal_completion_quiet() {
    let config = format!("{AUTO}\n[keys.vim_normal]\nq = 'delete-char'");
    let (_fixture, mut editor) = source(SOURCE, &config);
    keys(&mut editor, "q");
    assert!(editor.completion().is_none());
    assert!(editor.remaining_input(Instant::now()).is_none());
    keys(&mut editor, "vhl");
    assert!(editor.completion().is_none());
    editor.handle(key(KeyCode::Esc));
    editor.execute(Command::Completion(C::AtPoint));
    settle(&mut editor);
    assert!(editor.completion().is_some());
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    editor.handle(key(KeyCode::Esc));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
}

// Insert's one-Normal-command lifetime includes selector invocation, asynchronous
// completion and selected acceptance; it resumes only after that workflow finishes.
#[test]
fn completion_once() {
    let (_fixture, mut editor) = source(SOURCE, AUTO);
    keys(&mut editor, "i");
    control(&mut editor, 'o');
    keys(&mut editor, ":");
    answer(&mut editor, "completion-at-point");
    assert!(editor.completion().is_some());
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    editor.handle(key(KeyCode::Tab));
    settle(&mut editor);
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    assert!(text(&editor).contains("cfg_value }"));
    editor.execute(Command::Motion(Motion::BufferStart));
}
