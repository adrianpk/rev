use super::*;
use crate::buffer::Buffers;
use crate::command::Motion;
use crate::display::Size;
use crate::input::Input;
use crate::session::Pending;
use crate::view::View;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn session() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("abcdef").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 16,
        },
    )
    .unwrap()
}

fn record(session: &mut Session, named: bool, command: &str) {
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    if named {
        session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        session.handle(Event::Paste(command.into()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    } else {
        session.handle(key(KeyCode::Char('f'), KeyModifiers::CONTROL));
    }
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Optional metadata stays with the canonical dynamic identity and is rendered
// as inert single-line text. The record ceiling is independent of shared bodies.
#[test]
fn names_metadata() {
    use crate::command::MacroName;
    use crate::macros::Named;
    let mut session = session();
    record(&mut session, false, "");
    let source = session.macros.ring.front().unwrap().clone();
    for index in 0..256 {
        let name = MacroName::new(&format!("item-{index}")).unwrap();
        let definition = session.macros.snapshot(&source).unwrap();
        session
            .macros
            .name_snapshot(
                name,
                Named {
                    saved: None,
                    definition,
                    source: (source.id, source.revision),
                    description: (index == 0).then(|| "a\n".repeat(256)),
                },
            )
            .unwrap();
    }
    let name = MacroName::new("extra-item").unwrap();
    let definition = session.macros.snapshot(&source).unwrap();
    assert!(
        session
            .macros
            .name_snapshot(
                name,
                Named {
                    saved: None,
                    definition,
                    source: (source.id, source.revision),
                    description: None
                }
            )
            .is_err()
    );
    assert_eq!(session.macros.names.len(), 256);
    let name = MacroName::new("item-0").unwrap();
    let definition = session.macros.snapshot(&source).unwrap();
    assert!(
        session
            .macros
            .name_snapshot(
                name,
                Named {
                    saved: None,
                    definition,
                    source: (source.id, source.revision),
                    description: Some("a".repeat(513))
                }
            )
            .is_err()
    );
    session.sync_macro_catalog();
    session.execute(Command::ExecuteExtended);
    let palette = session.input.palette().unwrap();
    assert_eq!(
        palette.description(Command::MacroNamed(name)),
        Some("a ".repeat(256).as_str())
    );
    assert_eq!(
        palette.description(Command::MacroNamed(MacroName::new("item-1").unwrap())),
        None
    );
    session.frame().unwrap();
}

// Real distinct large bodies reach the aggregate memory ceiling before the
// record ceiling. Admission preserves the last valid ring/name on refusal.
#[test]
fn names_memory() {
    use crate::command::MacroName;
    use crate::macros::{Named, State, Step};
    let entry = session().macro_entry();
    let mut state = State::default();
    let mut refused = false;
    for index in 0..256 {
        let previous = state.ring.front().cloned();
        let mut recorder = Recorder::new(entry);
        recorder
            .stage(Step::Text(format!("{index}{}", "x".repeat(220_000))))
            .unwrap();
        recorder.accept().unwrap();
        match state.publish(recorder, 1) {
            Ok(()) => {}
            Err(reason) => {
                assert_eq!(reason, StopReason::RetainedMemory);
                assert!(Arc::ptr_eq(
                    previous.as_ref().unwrap(),
                    state.ring.front().unwrap()
                ));
                refused = true;
                break;
            }
        }
        let source = state.ring.front().unwrap().clone();
        let definition = match state.snapshot(&source) {
            Ok(definition) => definition,
            Err(reason) => {
                assert_eq!(reason, StopReason::RetainedMemory);
                refused = true;
                break;
            }
        };
        let before = state.names.len();
        if let Err(error) = state.name_snapshot(
            MacroName::new(&format!("item-{index}")).unwrap(),
            Named {
                saved: None,
                definition,
                source: (source.id, source.revision),
                description: None,
            },
        ) {
            assert!(error.contains("48 MiB"));
            assert_eq!(state.names.len(), before);
            refused = true;
            break;
        }
    }
    assert!(refused);
    assert!((2..256).contains(&state.names.len()));
}

// A removed captured command cannot be retargeted by a candidate row or ring
// index. An admitted nested call cannot renew the outer logical-step budget.
#[test]
fn names_admission() {
    use crate::command::MacroName;
    use crate::macros::Named;
    for missing in [false, true] {
        let mut session = session();
        record(&mut session, false, "");
        let source = session.macros.ring.front().unwrap().clone();
        let name = MacroName::new("part").unwrap();
        let definition = session.macros.snapshot(&source).unwrap();
        session
            .macros
            .name_snapshot(
                name,
                Named {
                    saved: None,
                    definition,
                    source: (source.id, source.revision),
                    description: None,
                },
            )
            .unwrap();
        session.sync_macro_catalog();
        session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
        session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        session.handle(Event::Paste("part".into()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
        for _ in 0..16 {
            if session.macros.runner.is_none() {
                break;
            }
            session.expire_input(Instant::now());
        }
        assert!(session.macros.runner.is_none());
        session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
        let before = text(&session);
        if missing {
            session.macros.names.remove(&name);
            session.sync_macro_catalog();
        }
        session.start_macro(1);
        if !missing {
            session.macros.runner.as_ref().unwrap().limits.steps.store(
                crate::macros::RUN_STEPS - 1,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
        session.expire_input(Instant::now());
        if session.macros.runner.is_some() {
            session.expire_input(Instant::now());
        }
        assert_eq!(
            session.macro_stop().unwrap().reason,
            if missing {
                StopReason::MissingCommand
            } else {
                StopReason::StepBudget
            }
        );
        assert_eq!(text(&session), before);
        assert!(session.macros.runner.is_none());
    }
}

// Replacing a directory context carries session definitions and source keys
// with the editor owner; it does not turn the replacement into a fresh session.
#[test]
fn names_context() {
    let mut session = session();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('f'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    session.execute(Command::Macro(MacroCommand::Name));
    session.handle(Event::Paste("move-item".into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.execute(Command::Macro(MacroCommand::Bind));
    session.handle(key(KeyCode::Char('7'), KeyModifiers::NONE));
    let replacement = session.prepare_context(None).unwrap();
    session.begin_context(replacement);
    assert!(session.input.command_named("move-item").is_some());
    assert_eq!(session.macros.names.len(), 1);
    assert_eq!(session.macros.bindings.len(), 1);
}

// Appending replaces an immutable body revision. A previously pinned value
// retains its body/counter even after metadata updates, rotation and eviction.
#[test]
fn ring_snapshot() {
    let mut session = session();
    record(&mut session, false, "");
    let pinned = session.macros.ring.front().unwrap().clone();
    let steps = pinned.steps.len();
    session.handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    let updated = session.macros.ring.front().unwrap();
    assert_eq!(updated.id, pinned.id);
    assert_eq!(updated.revision, pinned.revision + 1);
    assert_eq!(updated.steps.len(), steps + 1);
    assert_eq!(pinned.steps.len(), steps);
    assert_eq!(pinned.counter.current, 0);
    assert_eq!(updated.counter.current, 1);
    session.execute(Command::Macro(MacroCommand::Delete));
    assert!(session.macros.ring.is_empty());
    assert_eq!(pinned.steps.len(), steps);
}

// The same encoded/step admission applies to appended and fresh bodies; an
// append at the exact step ceiling cannot publish a partial new revision.
#[test]
fn ring_append_limit() {
    let mut session = session();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    for _ in 0..crate::macros::STEP_LIMIT {
        session.handle(key(KeyCode::Char('f'), KeyModifiers::CONTROL));
    }
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    let pinned = session.macros.ring.front().unwrap().clone();
    session.handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::CaptureLimit
    );
    assert!(Arc::ptr_eq(&pinned, session.macros.ring.front().unwrap()));
    assert_eq!(text(&session), "abcdef");
}

// Seed only already-consumed budget on a real admitted run. Its final allowed
// step may complete a positive count; nonpositive repetition must stop by budget.
#[test]
fn ring_budget() {
    for (count, expected) in [(1, None), (0, Some(StopReason::StepBudget))] {
        let mut session = session();
        session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
        session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
        session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
        session.start_macro(count);
        session.macros.runner.as_ref().unwrap().limits.steps.store(
            crate::macros::RUN_STEPS - 1,
            std::sync::atomic::Ordering::Relaxed,
        );
        session.expire_input(Instant::now());
        assert_eq!(text(&session), "XXabcdef");
        assert_eq!(session.macro_status(), Status::Idle);
        assert_eq!(session.macro_stop().map(|stop| stop.reason), expected);
    }
}

// A user-entered child run inherits the outer step and text budgets while a
// manual frame is suspended. Resuming a child cannot renew either allowance.
#[test]
fn queries_budget() {
    let mut session = session();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('4'), KeyModifiers::ALT));
    session.handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::Char('q'), KeyModifiers::NONE));
    session.handle(key(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    ));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    replay(&mut session);
    let outer = session.macros.pauses[0]
        .runner
        .as_ref()
        .unwrap()
        .limits
        .clone();
    outer.steps.store(
        crate::macros::RUN_STEPS - 1,
        std::sync::atomic::Ordering::Relaxed,
    );
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    session.expire_input(Instant::now());
    let child = session.macros.pauses[1].runner.as_ref().unwrap();
    assert!(Arc::ptr_eq(&outer, &child.limits));
    assert!(Arc::ptr_eq(&outer.text, &child.budget));
    session.handle(key(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    ));
    session.expire_input(Instant::now());
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::StepBudget);
    assert_eq!(session.macro_status(), Status::Idle);
    assert!(session.macros.pauses.is_empty());
    assert_eq!(text(&session), "Xabcdef");
}

// Original region targets use one shared allowance; reaching the final
// permitted step cannot publish the next line's source edit under a new budget.
#[test]
fn regions_budget() {
    let mut session = session();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    session.replace(0..7, "a\nb\n").unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Region(crate::command::RegionCommand::Set));
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Macro(MacroCommand::ApplyLines));
    session.macros.runner.as_ref().unwrap().limits.steps.store(
        crate::macros::RUN_STEPS - 1,
        std::sync::atomic::Ordering::Relaxed,
    );
    session.expire_input(Instant::now());
    assert_eq!(text(&session), "Xa\nb\n");
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::StepBudget);
    session.execute(Command::History(crate::command::History::Undo));
    assert_eq!(text(&session), "a\nb\n");
}

fn replay(session: &mut Session) {
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    while session
        .remaining_input(Instant::now())
        .is_some_and(|wait| wait.is_zero())
    {
        assert!(Instant::now() < deadline);
        session.expire_input(Instant::now());
    }
}

// Replace only the admitted input map after capture. Saved names remain stable;
// ordinary keys resolve their current map, including unsupported-job admission.
#[test]
fn current_bindings() {
    for (named, target, expected, stop) in [
        (true, "backward-char", "aXbXcdef", None),
        (false, "backward-char", "aXXbcdef", None),
        (
            false,
            "shell-command",
            "aXbcdef",
            Some(StopReason::Unsupported),
        ),
    ] {
        let mut session = session();
        record(&mut session, named, "forward-char");
        let fixture = crate::fixture::Fixture::new();
        let config = fixture.write(
            "config.toml",
            &format!("[keys.editor]\n'C-f' = '{target}'\n'M-x' = 'delete-char'\n"),
        );
        let prepared = crate::config::prepare(&config);
        assert!(prepared.diagnostic.is_none());
        session.input = Input::with_settings(&prepared.snapshot);
        session.sync_input_context();
        replay(&mut session);
        assert_eq!(text(&session), expected);
        assert_eq!(session.macro_stop().map(|stop| stop.reason), stop);
        assert_eq!(session.shell_remaining(), None);
        assert!(!matches!(session.pending, Some(Pending::Shell(_))));
    }
}

// Built-ins cannot disappear in a live session. Mutate only the stored name of
// a real recording to exercise the unknown-name admission boundary directly.
#[test]
fn missing_name() {
    let mut session = session();
    record(&mut session, true, "forward-char");
    let definition = Arc::get_mut(session.macros.ring.front_mut().unwrap()).unwrap();
    let crate::macros::Step::Command(command) = &mut Arc::make_mut(&mut definition.steps)[0] else {
        panic!("named recording")
    };
    command.name = "missing-native-command".into();
    let before = text(&session);
    replay(&mut session);
    assert_eq!(text(&session), before);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::MissingCommand
    );
    assert_eq!(session.input.focus(), Focus::Editing);
    assert!(session.pending.is_none());
}

// A changed key may open another valid prompt, but its accepted argument role
// cannot be reassigned to that prompt or leak into source editing.
#[test]
fn prompt_role() {
    let mut session = session();
    let fixture = crate::fixture::Fixture::new();
    let config = fixture.write("config.toml", "[keys.editor]\n'C-f' = 'goto-line'\n");
    session.input = Input::with_settings(&crate::config::prepare(&config).snapshot);
    session.sync_input_context();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('f'), KeyModifiers::CONTROL));
    session.handle(Event::Paste("1".into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    let config = fixture.write("config.toml", "[keys.editor]\n'C-f' = 'switch-to-buffer'\n");
    session.input = Input::with_settings(&crate::config::prepare(&config).snapshot);
    session.sync_input_context();
    session.execute(Command::Motion(Motion::BufferStart));
    let before = text(&session);
    replay(&mut session);
    assert_eq!(text(&session), before);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::PromptMismatch
    );
    assert_eq!(session.input.focus(), Focus::Editing);
}

// Choose an ambiguous selector row, then change ranking until that row would
// select another command. Replay still runs the resolved recorded identity.
#[test]
fn changed_ranking() {
    let mut session = session();
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste("char".into()));
    let candidates = session.input.palette().unwrap().candidates();
    let index = candidates
        .iter()
        .position(|command| command.name() == "forward-char")
        .unwrap();
    for _ in 0..index {
        session.handle(key(KeyCode::Down, KeyModifiers::NONE));
    }
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('X'), KeyModifiers::NONE));
    session.handle(key(KeyCode::F(4), KeyModifiers::NONE));
    for _ in 0..4 {
        session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        session.handle(Event::Paste("backward-char".into()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    }
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste("char".into()));
    assert_ne!(
        session.input.palette().unwrap().candidates()[index].name(),
        "forward-char"
    );
    session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    session.execute(Command::Motion(Motion::BufferStart));
    replay(&mut session);
    assert_eq!(text(&session), "aXXbcdef");
    assert_eq!(session.macro_stop(), None);
}

// Native search continuation has an ordinary no-match message outside macros.
// During capture its typed failure must prevent publishing a false success.
#[test]
fn search_continuation() {
    let mut session = session();
    let command = Command::Navigate(crate::command::Navigation::new(
        crate::command::navigation::Target::Search(crate::command::navigation::Search::Repeat),
    ));
    session.handle(key(KeyCode::F(3), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    session.handle(Event::Paste(command.name().into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(text(&session), "abcdef");
    assert!(session.macros.ring.is_empty());
}
