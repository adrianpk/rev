use super::*;
use rev::command::{Mode, Motion, Split};

fn modified(value: char, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), modifiers))
}

// Inactive Emacs overrides cannot shadow Vim; Insert inherits native modified
// factories, while Normal cannot execute their conflicting text-edit meanings.
#[test]
fn profile_isolation() {
    let config = format!(
        "{VIM}\n[keys.editor]\n'i' = 'newline'\n'C-f' = 'delete-char'\n[keys.vim_insert]\n'M-j' = 'newline'"
    );
    let mut editor = session(&config, "ab", 0);
    editor.handle(modified('f', KeyModifiers::CONTROL));
    editor.handle(key(KeyCode::Enter));
    assert_eq!(text(&editor), "ab");
    assert_eq!(editor.view().point.position, Scalar(0));
    editor.handle(key(KeyCode::Char('i')));
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    editor.handle(modified('f', KeyModifiers::CONTROL));
    assert_eq!(editor.view().point.position, Scalar(1));
    assert_eq!(text(&editor), "ab");
    editor.handle(modified('j', KeyModifiers::ALT));
    assert_eq!(text(&editor), "a\nb");
}

// Org editing uses its local layer above the selected state map, and the
// command prompt keeps literal text and protected cancellation in either state.
#[test]
fn local_priority() {
    let config = format!("{VIM}\n[keys.org]\n'i' = 'modal-enter-insert-line-end'");
    let mut editor = session(&config, "abc", 0);
    editor.execute(Command::Mode(Mode::Org));
    editor.handle(key(KeyCode::Char('i')));
    assert_eq!(editor.view().point.position, Scalar(3));
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    editor.handle(modified('x', KeyModifiers::ALT));
    editor.handle(Event::Paste("modal-enter-normal".into()));
    editor.handle(key(KeyCode::Enter));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert_eq!(text(&editor), "abc");
    editor.handle(modified('x', KeyModifiers::ALT));
    editor.handle(Event::Paste("literal:/i".into()));
    editor.handle(modified('g', KeyModifiers::CONTROL));
    assert_eq!(text(&editor), "abc");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
}

// Native view switches retire text-entry state; commands and history never
// borrow an insertion state that was admitted for a different pane.
#[test]
fn view_retirement() {
    let mut editor = session(VIM, "abc", 0);
    editor.handle(key(KeyCode::Char('i')));
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(rev::command::WindowCommand::Other));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    editor.handle(key(KeyCode::Char('z')));
    assert_eq!(text(&editor), "abc");
    editor.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(editor.view().point.position, Scalar(3));
}

// Bad mappings reject the whole candidate, including the requested profile;
// protected recovery and inactive-profile validation remain deterministic.
#[test]
fn map_fallback() {
    let fixture = Fixture::new();
    for (name, mapping) in [
        ("unknown", "[keys.vim_normal]\n'q' = 'not-a-command'"),
        ("unbinding", "[keys.vim_insert]\n'M-j' = ''"),
        ("prefix", "[keys.vim_normal]\n'i x' = 'undo'"),
        ("cancel", "[keys.vim_normal]\n'C-g' = 'undo'"),
        ("catalog", "[keys.vim_insert]\n'M-x' = 'undo'"),
        ("local catalog", "[keys.org]\n'M-x' = 'undo'"),
        (
            "terminal alias",
            "[keys.vim_insert]\n'TAB' = 'insert-tab'\n'C-i' = 'newline'",
        ),
    ] {
        let root = fixture.write("config.toml", &format!("{VIM}\n{mapping}"));
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(startup.snapshot, rev::config::Snapshot::default(), "{name}");
    }
}
