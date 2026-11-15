use super::*;
use rev::command::{ModalCommand, Split};

// Helper focus/read-only checks precede source mutation; modal text entry and
// shared views use the same checked transaction rather than a parallel editor.
#[test]
fn ownership() {
    let mut e = Editor::build("", "plain", "(|alpha)", true);
    named(&mut e, "pair-unwrap");
    e.expect("(|alpha)");
    assert!(e.session.buffer().unwrap().history().is_empty());
    for state in [ModalCommand::Insert, ModalCommand::Replace] {
        let mut e = selected("editor.interaction_profile='modal'", "plain", "alpha", true);
        e.session.execute(Command::Modal(state));
        e.type_key('(');
        e.expect("(|alpha)");
    }
    let mut e = selected("", "plain", "alpha", false);
    e.session.execute(Command::Split(Split::Right));
    e.type_key(']');
    e.expect("[alpha]|");
    assert!(
        e.session
            .windows()
            .iter()
            .all(|(_, v)| v.point.stamp == e.session.buffer().unwrap().stamp())
    );
}

// Pending qualification retains the finite native FIFO. Queued text uses the
// wrapped cursor once, while an external edit invalidates the old captured region.
#[test]
fn pending() {
    let text = "a ".repeat(2048);
    let mut e = selected("", "plain", &text, false);
    e.session.handle(character('('));
    e.session.handle(character('x'));
    settle(&mut e.session);
    e.expect(&format!("(x|{text})"));
    let mut e = selected("", "plain", &text, false);
    e.session.handle(character('['));
    e.session.replace(0..0, "changed ").unwrap();
    let point = e.session.view().point;
    settle(&mut e.session);
    assert_eq!(e.text(), format!("changed {text}"));
    assert_eq!(e.session.view().point, point);
    assert_eq!(e.session.buffer().unwrap().history().len(), 1);
}

// A counted request cannot publish wrapping fragments after byte exhaustion.
// C-g cancels an admitted pending operation before either delimiter is inserted.
#[test]
fn limits() {
    let text = "x".repeat(rev::pairs::MAX_BYTES + 1);
    let mut e = selected("", "plain", &text, false);
    let point = e.session.view().point;
    let mark = e.session.buffer().unwrap().mark();
    e.counted("3", '(');
    assert_eq!(e.text().len(), text.len());
    assert_eq!(e.text(), text);
    assert_eq!(e.session.view().point, point);
    assert_eq!(e.session.buffer().unwrap().mark(), mark);
    assert!(e.session.buffer().unwrap().mark_active());
    assert!(e.session.buffer().unwrap().history().is_empty());
    let mut e = selected("", "plain", &"a ".repeat(2048), false);
    let stamp = e.session.buffer().unwrap().stamp();
    e.session.handle(character('('));
    e.session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    settle(&mut e.session);
    assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
    assert!(e.session.buffer().unwrap().history().is_empty());
}

// Rectangular and modal Visual input retain their native owners. Neither the
// automatic trigger nor the explicit ordinary-region helper converts their shape.
#[test]
fn shapes() {
    let mut e = selected("", "plain", "alpha", false);
    e.session
        .execute(Command::Rectangle(rev::command::RectangleCommand::Mark));
    named(&mut e, "pair-wrap");
    assert_eq!(e.text(), "alpha");
    assert!(e.session.buffer().unwrap().rectangle_active());
    e.type_key('(');
    e.expect("alpha(|");
    for state in [ModalCommand::VisualCharacter, ModalCommand::VisualBlock] {
        let mut e = Editor::new("editor.interaction_profile='modal'", "plain", "|alpha");
        e.session.execute(Command::Modal(state));
        let visual = e.session.buffer().unwrap().visual();
        e.type_key('(');
        e.expect("|alpha");
        assert_eq!(e.session.buffer().unwrap().visual(), visual);
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}
