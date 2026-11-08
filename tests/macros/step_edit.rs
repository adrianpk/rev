use super::*;
use rev::macros::Wait;
fn start(s: &mut Session) {
    feed(s, [control('x'), control('k'), character(' ')]);
    settle(s);
    assert_eq!(s.macro_wait(), Some(Wait::Step), "{}", s.echo());
    assert!(
        s.frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Step 1:")),
        "the next logical step must be visible"
    );
}
fn answer(s: &mut Session, event: Event) {
    s.handle(event);
    settle(s);
}
fn replay(s: &mut Session) {
    s.handle(key(KeyCode::F(4)));
    settle(s);
}

// Removing changes only the pending body; skipping affects this execution.
// Successful completion publishes once and replay uses the checked result.
#[test]
fn choices() {
    let mut s = editor("", "");
    record(&mut s, [character('A'), character('B'), character('C')]);
    start(&mut s);
    answer(&mut s, character('?'));
    assert!(s.echo().contains("lowercase"));
    answer(&mut s, character('n'));
    answer(&mut s, character('f'));
    answer(&mut s, character('y'));
    assert_eq!(s.macro_status(), Status::Idle, "{}", s.echo());
    assert_eq!(text(&s), "ABCC");
    replay(&mut s);
    assert_eq!(text(&s), "ABCCBC");
}

// Cancellation preserves actual edits and the original definition, including
// changes tentatively made after an executed step. One native undo owns the run.
#[test]
fn cancellation() {
    for cancel in [character('q'), control('g')] {
        let mut s = editor("", "");
        record(&mut s, [character('A'), character('B'), character('C')]);
        start(&mut s);
        answer(&mut s, character(' '));
        answer(&mut s, key(KeyCode::Backspace));
        answer(&mut s, cancel);
        assert_eq!(text(&s), "ABCA");
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), "ABC");
        replay(&mut s);
        assert_eq!(text(&s), "ABCABC");
    }
}

// Text grouping and truncation have bounded deterministic effects; ordinary
// native execution restores modal entry state before publishing the candidate.
#[test]
fn groups() {
    let mut s = editor("", "");
    record(&mut s, [Event::Paste("AB".into()), character('C')]);
    start(&mut s);
    answer(&mut s, key(KeyCode::Tab));
    assert_eq!(text(&s), "ABCABC");
    assert_eq!(s.macro_status(), Status::Idle);
    start(&mut s);
    answer(&mut s, control('k'));
    replay(&mut s);
    assert_eq!(text(&s), "ABCABC");
    for modal in [false, true] {
        let mut s = editor(
            if modal {
                "[editor]\ninteraction_profile='modal'\n"
            } else {
                ""
            },
            "",
        );
        record(
            &mut s,
            if modal {
                vec![character('i'), character('A'), key(KeyCode::Esc)]
            } else {
                vec![character('A')]
            },
        );
        start(&mut s);
        answer(&mut s, character('c'));
        assert_eq!(text(&s), "AA");
        assert_eq!(s.macro_status(), Status::Idle, "{}", s.echo());
    }
}

// Uppercase choices capture native key sequences, execute them once, and resume
// questions at the next original step rather than replaying source effects twice.
#[test]
fn key_input() {
    for (choice, expected, replayed) in [('I', "ACBAC", "ACBACBAC"), ('R', "ACBC", "ACBCBC")] {
        let mut s = editor("", "");
        record(&mut s, [character('A'), character('C')]);
        start(&mut s);
        answer(&mut s, character(choice));
        answer(&mut s, character('B'));
        assert_eq!(s.macro_wait(), Some(Wait::Step), "{}", s.echo());
        answer(&mut s, character('c'));
        assert_eq!(text(&s), expected, "{choice}: {}", s.echo());
        replay(&mut s);
        assert_eq!(text(&s), replayed, "{choice}");
    }
}

// Lowercase reads use native recording and end only at a complete command on
// C-j. Insert, replace, after and append preserve exactly-once source effects.
#[test]
fn recorded_input() {
    for (choice, expected, replayed) in [
        ('i', "ACxyAC", "ACxyACxyAC"),
        ('r', "ACxyC", "ACxyCxyC"),
        ('a', "ACAxyC", "ACAxyCAxyC"),
        ('A', "ACACxy", "ACACxyACxy"),
    ] {
        let mut s = editor("", "");
        record(&mut s, [character('A'), character('C')]);
        start(&mut s);
        answer(&mut s, character(choice));
        s.handle(Event::Paste("xy".into()));
        answer(&mut s, control('j'));
        if s.macro_status() != Status::Idle {
            answer(&mut s, character('c'));
        }
        assert_eq!(text(&s), expected, "{choice}: {}", s.echo());
        assert_eq!(s.macro_status(), Status::Idle, "{choice}: {}", s.echo());
        replay(&mut s);
        assert_eq!(text(&s), replayed, "{choice}: {}", s.echo());
    }
}

// Named selector UI is normalized into one command while native prompt roles
// stay with that command. Canceling a read retains edits and the old snapshot.
#[test]
fn semantic_read() {
    let mut s = editor("", "");
    record(&mut s, [character('A')]);
    start(&mut s);
    answer(&mut s, character('i'));
    feed(
        &mut s,
        [
            meta('x'),
            Event::Paste("goto-line".into()),
            key(KeyCode::Enter),
            character('1'),
            key(KeyCode::Enter),
            character('Z'),
        ],
    );
    answer(&mut s, control('j'));
    answer(&mut s, character('c'));
    assert_eq!(text(&s), "ZAA", "{}", s.echo());
    start(&mut s);
    answer(&mut s, character('r'));
    s.handle(Event::Paste("partial".into()));
    answer(&mut s, control('g'));
    assert!(text(&s).contains("partial"));
    assert_eq!(s.macro_status(), Status::Idle);
    replay(&mut s);
    assert_eq!(text(&s).matches('Z').count(), 2);
}

// Foreign macro controls cannot mutate the ring while step editing is active;
// admission stops before the control and the previous definition remains usable.
#[test]
fn control_stop() {
    let mut s = editor("", "");
    record(&mut s, [character('A')]);
    start(&mut s);
    s.execute(Command::Macro(MacroCommand::Delete));
    assert_eq!(s.macro_status(), Status::Idle);
    assert_eq!(s.macro_stop().unwrap().reason, StopReason::Unsupported);
    replay(&mut s);
    assert_eq!(text(&s), "AA");
}

// Query prompts keep their native answer owner inside step execution. A query
// answer cannot accidentally act as a step-editor choice or republish early.
#[test]
fn query_owner() {
    let mut s = editor("", "");
    record(&mut s, [control('x'), character('q'), character('A')]);
    start(&mut s);
    answer(&mut s, character('y'));
    assert_eq!(s.macro_wait(), Some(Wait::Query));
    answer(&mut s, character('y'));
    assert_eq!(s.macro_wait(), Some(Wait::Step));
    answer(&mut s, character('c'));
    assert_eq!(text(&s), "AA");
    assert_eq!(s.macro_status(), Status::Idle);
}

// Grouping in modal Normal must execute only the state transition. Only native
// text-entry states admit the following adjacent text as one homogeneous group.
#[test]
fn modal_group() {
    let mut s = editor("[editor]\ninteraction_profile='modal'\n", "");
    record(
        &mut s,
        [
            character('i'),
            character('A'),
            character('B'),
            key(KeyCode::Esc),
        ],
    );
    start(&mut s);
    answer(&mut s, key(KeyCode::Tab));
    assert_eq!(text(&s), "AB");
    assert_eq!(s.macro_wait(), Some(Wait::Step));
    answer(&mut s, key(KeyCode::Tab));
    assert_eq!(text(&s), "ABAB");
    answer(&mut s, character('c'));
    assert_eq!(s.macro_status(), Status::Idle);
}

// A manual query can invoke a child macro while the original step runner is
// suspended. Child completion must return to that query, never publish the edit.
#[test]
fn manual_child() {
    let mut s = editor("", "");
    record(&mut s, [character('A')]);
    feed(
        &mut s,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste("part".into()),
            key(KeyCode::Enter),
        ],
    );
    s.handle(key(KeyCode::F(3)));
    feed(&mut s, [meta('4'), control('x'), character('q')]);
    let resume = Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    ));
    s.handle(resume.clone());
    s.handle(character('B'));
    s.handle(key(KeyCode::F(4)));
    start(&mut s);
    answer(&mut s, character('y'));
    assert_eq!(s.macro_wait(), Some(Wait::Editing));
    s.execute(Command::MacroNamed(
        rev::command::MacroName::new("part").unwrap(),
    ));
    settle(&mut s);
    assert_eq!(s.macro_wait(), Some(Wait::Editing), "{}", s.echo());
    assert_eq!(text(&s), "ABA");
    s.handle(resume);
    settle(&mut s);
    assert_eq!(s.macro_wait(), Some(Wait::Step));
    answer(&mut s, character('c'));
    assert_eq!(text(&s), "ABAB");
    assert_eq!(s.macro_status(), Status::Idle);
}
