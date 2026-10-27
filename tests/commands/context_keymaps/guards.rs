use super::*;

// Mapped accept/edit actions never grant consent. Cancel prefixes are allowed,
// but a mismatched y suffix is consumed; a fresh explicit answer is required.
#[test]
fn decision_actions() {
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'M-a' = 'minibuffer-accept'\n'M-z a' = 'minibuffer-accept'\n'M-q x' = 'keyboard-quit'",
    ));
    input.decision("Continue? y/n").unwrap();
    assert_eq!(input.handle(meta('a')), Outcome::Message("Answer y or n"));
    assert_eq!(input.handle(meta('z')), Outcome::Message("Answer y or n"));
    assert_eq!(input.prefix(), None);
    assert_eq!(input.handle(meta('q')), Outcome::Prefix(Prefix::Sequence));
    assert_eq!(
        input.handle(character('y')),
        Outcome::Message("Undefined key sequence")
    );
    assert_eq!(input.focus(), Focus::Decision);
    assert_eq!(
        input.handle(phase(character('y'), KeyEventKind::Repeat)),
        Outcome::Ignored
    );
    assert_eq!(input.handle(character('n')), Outcome::Decision(false));
    input.decision("Continue? y/n").unwrap();
    input.handle(meta('q'));
    assert_eq!(
        input.handle(character('x')),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert_eq!(input.prefix(), None);
}

// Aliased query/replacement acceptance advances existing stages. Neither that
// chord, paste nor a held answer can replace; fresh y publishes one native edit.
#[test]
fn replacement_consent() {
    let mut session = editor("[keys.minibuffer]\n'M-a' = 'minibuffer-accept'", "cat cat");
    session.execute(Command::QueryReplace);
    session.handle(Event::Paste("cat".into()));
    session.handle(meta('a'));
    session.handle(Event::Paste("界".into()));
    session.handle(phase(meta('a'), KeyEventKind::Repeat));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Replace with: 界")
    );
    session.handle(meta('a'));
    let stamp = session.buffer().unwrap().stamp();
    for event in [
        meta('a'),
        key(KeyCode::Enter),
        Event::Paste("y".into()),
        phase(character('y'), KeyEventKind::Repeat),
    ] {
        session.handle(event);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&session), "cat cat");
    }
    session.handle(character('y'));
    session.handle(phase(character('y'), KeyEventKind::Repeat));
    assert_eq!(text(&session), "界 cat");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.handle(control('g'));
    assert_eq!(session.echo(), "Quit");
}

// Configured delete still captures a pending filesystem decision. Accept aliases
// and repeated y cannot remove the target; cancellation preserves disk and text.
#[test]
fn filesystem_consent() {
    let (fixture, mut session) = tree_editor(
        "[keys.tree]\n'M-d' = 'tree-delete'\n[keys.minibuffer]\n'M-a' = 'minibuffer-accept'\n'M-q x' = 'keyboard-quit'",
    );
    session.execute(Command::Tree(TreeCommand::Expand));
    session.handle(key(KeyCode::Down));
    let stamp = session.buffer().unwrap().stamp();
    session.handle(meta('d'));
    assert!(session.echo().starts_with("Delete "));
    for event in [
        meta('a'),
        key(KeyCode::Enter),
        Event::Paste("y".into()),
        phase(character('y'), KeyEventKind::Repeat),
    ] {
        session.handle(event);
        assert_eq!(
            std::fs::read(fixture.0.join("root/a.txt")).unwrap(),
            b"retained"
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
    }
    session.handle(meta('q'));
    session.handle(character('y'));
    assert!(session.echo().ends_with("Undefined key sequence"));
    assert!(fixture.0.join("root/a.txt").exists());
    session.handle(control('g'));
    assert!(session.tree().focused());
    assert_eq!(session.echo(), "Quit");
    session.handle(meta('d'));
    session.handle(character('y'));
    assert!(!fixture.0.join("root/a.txt").exists());
    assert_eq!(text(&session), "retained");
}

// Every active minibuffer owner keeps reserved C-g, even when a configured
// prefix is pending. Inherited prompt keys retain their stage-specific limits.
#[test]
fn reserved_cancellation() {
    let snapshot = settings("[keys.minibuffer]\n'M-z a' = 'minibuffer-accept'");
    for owner in ["argument", "commands", "search", "inventory", "decision"] {
        let mut input = Input::with_settings(&snapshot);
        open(&mut input, owner);
        input.handle(meta('z'));
        assert_eq!(
            input.handle(control('g')),
            Outcome::Command(Command::KeyboardQuit),
            "{owner}"
        );
        assert_eq!(input.focus(), Focus::Editing, "{owner}");
        assert_eq!(input.prefix(), None, "{owner}");
        assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
    }
    let mut input = Input::with_settings(&snapshot);
    input.inventory().unwrap();
    assert_eq!(
        input.handle(control('p')),
        Outcome::Message("Use arrows, Enter or C-g")
    );
    assert_eq!(
        input.handle(control('a')),
        Outcome::Message("Use arrows, Enter or C-g")
    );
    assert_eq!(
        input.handle(key(KeyCode::Up)),
        Outcome::Selection(rev::input::Selection::Previous)
    );
}

fn open(input: &mut Input, owner: &str) {
    match owner {
        "argument" => input.argument("Text: ", "draft"),
        "commands" => input.commands(),
        "search" => input.search(rev::search::Direction::Forward),
        "inventory" => input.inventory(),
        "decision" => input.decision("Continue? y/n"),
        _ => unreachable!(),
    }
    .unwrap();
}
