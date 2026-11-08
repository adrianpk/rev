use super::*;

fn control(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::CONTROL))
}
fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}
fn named(session: &mut Session, name: &str) {
    session.handle(meta('x'));
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}
fn settle(session: &mut Session) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while session
        .remaining_input(std::time::Instant::now())
        .is_some_and(|wait| wait.is_zero())
    {
        assert!(std::time::Instant::now() < deadline);
        session.expire_input(std::time::Instant::now());
    }
}

// Normal/Visual C-u keeps native half-page ownership. Explicit editor helpers
// remain available by name; Escape retires them without an inherited count.
#[test]
fn escape_ownership() {
    let source = "line\n".repeat(60);
    for visual in [false, true] {
        let mut session = session(VIM, &source, 100);
        if visual {
            session.handle(key(KeyCode::Char('v')));
        }
        session.handle(control('u'));
        settle(&mut session);
        assert!(!session.echo().contains("Argument"));
        let point = session.view().point;
        named(&mut session, "prefix-argument");
        session.handle(key(KeyCode::Char('3')));
        session.handle(key(KeyCode::Esc));
        assert_eq!(session.editing_state(), Some(EditingState::Normal));
        assert_eq!(session.view().point, point);
        session.handle(key(KeyCode::Char('j')));
        settle(&mut session);
        assert_eq!(session.view().point.position.0, point.position.0 + 5);
    }
}

// Both orders of a supplied editor argument and native modal count refuse the
// combination. Named helper selection cannot silently discard a modal count.
#[test]
fn count_conflicts() {
    for route in ["binding", "named", "after prefix"] {
        let config = format!("{VIM}\n[keys.vim_normal]\n'M-q'='prefix-argument'");
        let mut session = session(&config, "abcdefghij", 5);
        if route == "after prefix" {
            session.handle(meta('q'));
            session.handle(key(KeyCode::Char('3')));
            session.handle(meta('q'));
            session.handle(key(KeyCode::Char('2')));
            session.handle(key(KeyCode::Char('h')));
        } else {
            session.handle(key(KeyCode::Char('2')));
            if route == "binding" {
                session.handle(meta('q'));
            } else {
                named(&mut session, "prefix-argument");
            }
        }
        assert!(
            session.echo().contains("conflict"),
            "{route}: {}",
            session.echo()
        );
        assert_eq!(session.view().point.position, Scalar(5));
        session.handle(key(KeyCode::Char('h')));
        settle(&mut session);
        assert_eq!(session.view().point.position, Scalar(4));
        assert_eq!(text(&session), "abcdefghij");
    }
}

// Insert/Replace inherit editor entry, while standalone modal counts and dot
// retain their existing operator/repeat owner after an argument is canceled.
#[test]
fn insertion_arguments() {
    for entry in ['i', 'R'] {
        let mut session = session(VIM, "abcd", 0);
        session.handle(key(KeyCode::Char(entry)));
        session.handle(control('u'));
        session.handle(key(KeyCode::Char('2')));
        session.handle(key(KeyCode::Char('界')));
        settle(&mut session);
        assert_eq!(
            text(&session),
            if entry == 'i' {
                "界界abcd"
            } else {
                "界界cd"
            }
        );
        session.handle(control('u'));
        session.handle(key(KeyCode::Esc));
        session.execute(Command::Motion(rev::command::Motion::BufferStart));
        session.handle(key(KeyCode::Char('2')));
        session.handle(key(KeyCode::Char('x')));
        settle(&mut session);
        assert_eq!(text(&session), if entry == 'i' { "abcd" } else { "cd" });
        session.execute(Command::History(History::Undo));
        assert_eq!(
            text(&session),
            if entry == 'i' {
                "界界abcd"
            } else {
                "界界cd"
            }
        );
    }
}

// Large explicit row work yields to cancellation under modal ownership and
// releases all count progress before the next ordinary native motion.
#[test]
fn cooperative_cancel() {
    let source = "line\n".repeat(4000);
    let mut session = session(VIM, &source, 0);
    named(&mut session, "prefix-argument");
    for digit in "1000000".chars() {
        session.handle(key(KeyCode::Char(digit)));
    }
    named(&mut session, "next-line");
    assert_eq!(session.view().point.position, Scalar(0));
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(0));
    session.handle(key(KeyCode::Char('j')));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(5));
    assert!(session.buffer().unwrap().history().is_empty());
}
