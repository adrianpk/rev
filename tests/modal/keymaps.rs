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
    editor.handle(key(KeyCode::Char('i')));
    editor.handle(modified('g', KeyModifiers::CONTROL));
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    editor.execute(Command::KeyboardQuit);
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
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
        assert_eq!(
            startup.snapshot,
            rev::config::Snapshot::default().with_root(&root),
            "{name}"
        );
    }
}

// The selector captures every modal state before command focus takes over.
// Colon and M-x retain the same Visual owner and canonical completion text.
#[test]
fn state_hints() {
    let config = format!(
        "{VIM}\n[keys.vim_normal]\n'M-j'='save-buffer'\n[keys.vim_insert]\n'M-k'='save-buffer'\n[keys.vim_replace]\n'M-l'='save-buffer'\n[keys.vim_visual_character]\n'M-a'='save-buffer'\n[keys.vim_visual_line]\n'M-b'='save-buffer'\n[keys.vim_visual_block]\n'M-c'='save-buffer'"
    );
    for (case, entry, hint, colon) in [
        ("normal colon", ModalCommand::Normal, "M-j", true),
        ("normal Meta", ModalCommand::Normal, "M-j", false),
        ("insert", ModalCommand::Insert, "M-k", false),
        ("replace", ModalCommand::Replace, "M-l", false),
        (
            "Visual character",
            ModalCommand::VisualCharacter,
            "M-a",
            true,
        ),
        ("Visual line", ModalCommand::VisualLine, "M-b", true),
        ("Visual block", ModalCommand::VisualBlock, "M-c", true),
    ] {
        let mut editor = session(&config, "ab\ncd", 1);
        editor.execute(Command::Modal(entry));
        let state = editor.editing_state();
        let point = editor.view().point;
        let mark = editor.buffer().unwrap().mark();
        let stamp = editor.buffer().unwrap().stamp();
        editor.handle(if colon {
            key(KeyCode::Char(':'))
        } else {
            modified('x', KeyModifiers::ALT)
        });
        editor.handle(Event::Paste("save-buffer".into()));
        let frame = editor.frame().unwrap();
        let first = frame
            .rows()
            .iter()
            .find(|row| row.band() == rev::display::Band::Completion)
            .unwrap();
        let expected_hint = format!("[{hint}]");
        let expected: Vec<_> = ["save-buffer", expected_hint.as_str()]
            .into_iter()
            .chain(
                Command::SaveBuffer
                    .description()
                    .unwrap()
                    .split_whitespace(),
            )
            .collect();
        assert_eq!(
            first.text().split_whitespace().collect::<Vec<_>>(),
            expected,
            "{case}"
        );
        editor.handle(key(KeyCode::Tab));
        assert!(
            editor
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().starts_with("M-x save-buffer ")),
            "{case}"
        );
        editor.handle(modified('g', KeyModifiers::CONTROL));
        assert_eq!(editor.editing_state(), state, "{case}");
        assert_eq!(editor.view().point, point, "{case}");
        assert_eq!(editor.buffer().unwrap().mark(), mark, "{case}");
        assert_eq!(editor.buffer().unwrap().stamp(), stamp, "{case}");
    }
}

// Admitted digit prefixes may still be consumed as counts by actual grammar.
// Read-only map iteration alone would falsely advertise this unreachable leaf.
#[test]
fn consumed_hint() {
    let fixture = Fixture::new();
    let config = fixture.write(
        "config.toml",
        &format!("{VIM}\n[keys.vim_normal]\n'1 a'='mouse-mode'"),
    );
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut input = rev::input::Input::with_settings(&startup.snapshot);
    input.commands().unwrap();
    assert_eq!(input.palette().unwrap().hint(Command::MouseMode), None);
    input.cancel();
    input.handle(key(KeyCode::Char('1')));
    assert_ne!(
        input.handle(key(KeyCode::Char('a'))),
        rev::input::Outcome::Command(Command::MouseMode)
    );
}
