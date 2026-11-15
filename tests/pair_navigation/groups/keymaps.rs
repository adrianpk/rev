use super::*;
use rev::command::navigation::Target;
use rev::command::{ModalCommand, Navigation};
use rev::search::Direction;

// Only the existing source-map inheritance admits the new chords. Normal and
// Visual defaults remain filtered; named motion stays available on source.
#[test]
fn inheritance() {
    for state in [
        None,
        Some(ModalCommand::Normal),
        Some(ModalCommand::Insert),
        Some(ModalCommand::Replace),
        Some(ModalCommand::VisualCharacter),
        Some(ModalCommand::VisualLine),
        Some(ModalCommand::VisualBlock),
    ] {
        let modal = state.is_some();
        let config = if modal {
            "editor.interaction_profile='modal'"
        } else {
            ""
        };
        let mut e = Editor::build(config, "plain", "|(a)", true);
        if let Some(state) = state {
            e.session.execute(Command::Modal(state));
        }
        let admitted = matches!(
            state,
            None | Some(ModalCommand::Insert | ModalCommand::Replace)
        );
        keys(&mut e, "C-M-n");
        assert_eq!(
            e.session.view().point.position,
            Scalar(if admitted { 3 } else { 0 }),
            "{state:?}"
        );
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
    for language in ["org", "markdown"] {
        let mut e = Editor::new("", language, "|(a)");
        keys(&mut e, "C-M-n");
        e.expect("(a)|");
        keys(&mut e, "C-M-p");
        e.expect("|(a)");
    }
}

// Explicit user assignments win over the new defaults, and the rendered M-x
// hint identifies the effective alternative chord rather than the base map.
#[test]
fn override_hint() {
    let mut e = Editor::new(
        "[keys.editor]\n'C-M-n'='forward-char'\n'M-j'='forward-pair'",
        "plain",
        "|(a)",
    );
    keys(&mut e, "C-M-n");
    e.expect("(|a)");
    e.session.execute(Command::Motion(Motion::BufferStart));
    e.session.execute(Command::ExecuteExtended);
    e.session.handle(Event::Paste("forward-pair".into()));
    assert!(echo(&mut e).contains("M-j"));
    e.session.execute(Command::KeyboardQuit);
    keys(&mut e, "M-j");
    e.expect("(a)|");
}

// Group motion adds no modal jump history. Pending operators reject a custom
// group chord; native M-x explicitly retires the operator before named motion.
#[test]
fn modal_owner() {
    let mut e = Editor::new("editor.interaction_profile='modal'", "plain", "|(a)");
    named(&mut e, "forward-pair");
    e.expect("(a)|");
    e.session
        .execute(Command::Navigate(Navigation::new(Target::Jump(
            Direction::Backward,
        ))));
    settle(&mut e.session);
    e.expect("(a)|");
    let mut e = Editor::new(
        "editor.interaction_profile='modal'\n[keys.vim_normal]\n'M-j'='forward-pair'",
        "plain",
        "|(a)",
    );
    keys(&mut e, "d M-j");
    e.expect("|(a)");
    assert!(e.session.buffer().unwrap().history().is_empty());
    e.session.handle(character('d'));
    e.session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    e.session.handle(Event::Paste("forward-pair".into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(&mut e.session);
    e.expect("(a)|");
    assert!(e.session.buffer().unwrap().history().is_empty());
}

// Group endpoints after a delimiter plus combining mark must advance to a whole
// grapheme; backward scalar targets use the nearest earlier native boundary.
#[test]
fn graphemes() {
    for (source, point, command, target) in [
        ("(a)\u{301}", 0, "forward-pair", 4),
        ("\u{600}(a)", 4, "backward-pair", 0),
        ("(a)\r\n", 0, "forward-pair", 3),
    ] {
        let mut e = Editor::new("", "plain", &marked(source, point));
        named(&mut e, command);
        e.expect(&marked(source, target));
        assert!(
            rev::coordinates::Coordinates::new(e.session.buffer().unwrap())
                .unwrap()
                .is_boundary(e.session.view().point.position)
                .unwrap()
        );
    }
}

// Accepted macro input preserves one signed-count invocation. Replaying it uses
// the current group owner once and cannot turn motion into literal source edits.
#[test]
fn macro_replay() {
    use rev::command::MacroCommand;
    let mut e = Editor::new("", "plain", "|(a) (b)");
    e.session.execute(Command::Macro(MacroCommand::Start));
    keys(&mut e, "C-u 2 C-M-n");
    e.session.execute(Command::Macro(MacroCommand::End));
    e.expect("(a) (b)|");
    e.session.execute(Command::Motion(Motion::BufferStart));
    e.session.execute(Command::Macro(MacroCommand::CallRepeat));
    settle(&mut e.session);
    e.expect("(a) (b)|");
    assert!(e.session.buffer().unwrap().history().is_empty());
}

// The native repeat event takes another whole group through the same bound
// chord, so held input cannot silently lose its ordinary-motion repeat policy.
#[test]
fn held() {
    let mut e = Editor::new("", "plain", "|(a) (b)");
    keys(&mut e, "C-M-n");
    e.expect("(a)| (b)");
    e.session.handle(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char('n'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
        crossterm::event::KeyEventKind::Repeat,
    )));
    settle(&mut e.session);
    e.expect("(a) (b)|");
}
