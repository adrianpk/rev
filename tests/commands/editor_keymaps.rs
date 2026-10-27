use crossterm::event::{KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, Scroll};
use rev::config::{Snapshot, prepare};
use rev::coordinates::Scalar;
use rev::input::{Input, Outcome, Prefix};
use rev::session::{Action, Session};
use rev::view::{Size, View};

use super::{Event, KeyCode, character, control, key};

fn settings(config: &str) -> Snapshot {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(
        startup.diagnostic.is_none(),
        "{config}: {:?}",
        startup.diagnostic
    );
    startup.snapshot
}

fn meta(character: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::ALT))
}

fn editor(config: &str, source: &str) -> Session {
    let mut buffers = Buffers::with_settings(settings(config)).unwrap();
    let id = buffers.create(source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

// A user alias reaches the native handler rather than inserting its suffix.
// The inherited chord remains live and uses the same buffer-qualified motion.
#[test]
fn configured_alias() {
    let mut session = editor("[keys.editor]\n'M-j' = 'next-line'", "ab\ncd\nef");
    let stamp = session.buffer().unwrap().stamp();
    assert_eq!(session.handle(meta('j')), Action::Redraw);
    assert_eq!(session.view().point.position, Scalar(3));
    session.handle(control('n'));
    assert_eq!(session.view().point.position, Scalar(6));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A custom prefix consumes every chord, reports its full normalized label and
// emits one existing command only after its complete bounded sequence arrives.
#[test]
fn custom_sequence() {
    let snapshot = settings("[keys.editor]\n'M-z a b' = 'find-file'");
    let mut input = Input::with_settings(&snapshot);
    assert_eq!(input.handle(meta('z')), Outcome::Prefix(Prefix::Sequence));
    assert_eq!(input.prefix_label().as_deref(), Some("M-z"));
    assert_eq!(
        input.handle(character('a')),
        Outcome::Prefix(Prefix::Sequence)
    );
    assert_eq!(input.prefix_label().as_deref(), Some("M-z a"));
    assert_eq!(
        input.handle(character('b')),
        Outcome::Command(Command::FindFile)
    );
    assert_eq!(input.prefix(), None);
    assert!(matches!(input.handle(character('b')), Outcome::Editing(_)));
}

// Native report aliases and compatible Escape delivery resolve the same custom
// binding, using the existing Meta deadline and repeat owner.
#[test]
fn meta_delivery() {
    let snapshot = settings("[keys.editor]\n'M-j' = 'next-line'\n'C-i' = 'forward-char'");
    let mut input = Input::with_settings(&snapshot);
    input.handle(key(KeyCode::Esc));
    assert_eq!(
        input.handle(character('j')),
        Outcome::Command(Command::Scroll(Scroll::NextLine))
    );
    assert_eq!(
        input.handle(key(KeyCode::Tab)),
        Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
    );
    assert_eq!(
        input.handle(control('i')),
        Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
    );
}

// Exact replacement does not fall through to the hardcoded original handler.
// Independent inherited aliases remain, and omitting the override restores it.
#[test]
fn exact_reassignment() {
    let overridden = settings("[keys.editor]\n'C-f' = 'backward-char'");
    let restored = settings("# No custom binding.\n");
    for (snapshot, expected) in [
        (&overridden, rev::command::Motion::BackwardChar),
        (&restored, rev::command::Motion::ForwardChar),
    ] {
        let mut input = Input::with_settings(snapshot);
        assert_eq!(
            input.handle(control('f')),
            Outcome::Command(Command::Motion(expected))
        );
        assert_eq!(
            input.handle(key(KeyCode::Right)),
            Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
        );
    }
}

// Prompt acquisition and cancellation consume the full custom prefix state.
// Its old suffix becomes prompt text, then ordinary editing after acceptance.
#[test]
fn prompt_reset() {
    let snapshot = settings("[keys.editor]\n'M-z a' = 'find-file'");
    for prompt in ["argument", "commands", "search", "decision", "inventory"] {
        let mut input = Input::with_settings(&snapshot);
        input.handle(meta('z'));
        open_prompt(&mut input, prompt);
        assert_eq!(input.prefix(), None, "{prompt}");
        input.handle(control('g'));
        assert_eq!(input.prefix(), None, "{prompt}");
        assert!(
            matches!(input.handle(character('a')), Outcome::Editing(_)),
            "{prompt}"
        );
    }
}

fn open_prompt(input: &mut Input, prompt: &str) {
    match prompt {
        "argument" => input.argument("Text: ", ""),
        "commands" => input.commands(),
        "search" => input.search(rev::search::Direction::Forward),
        "decision" => input.decision("Continue? y/n"),
        "inventory" => input.inventory(),
        _ => unreachable!(),
    }
    .unwrap();
}

// Editor reassignment cannot change a focused prompt's native cursor binding.
// A mismatch/cancel consumes all prefix chords before editing resumes.
#[test]
fn transient_isolation() {
    let snapshot = settings("[keys.editor]\n'C-f' = 'find-file'\n'M-z a' = 'next-line'");
    let mut input = Input::with_settings(&snapshot);
    input.argument("Text: ", "ab").unwrap();
    input.handle(key(KeyCode::Home));
    assert_eq!(input.handle(control('f')), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().cursor(), 1);
    input.handle(control('g'));
    input.handle(meta('z'));
    assert_eq!(
        input.handle(character('x')),
        Outcome::Message("Undefined key sequence")
    );
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
    input.handle(meta('z'));
    input.handle(control('g'));
    assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
}

// Workspace ownership changes clear pending editor prefixes immediately, even
// when a native host changes focus directly instead of delivering a bound key.
#[test]
fn workspace_reset() {
    use rev::command::TreeCommand;
    let mut session = editor(
        "[keys.editor]\n'C-n' = 'newline'\n'M-z a' = 'newline'",
        "ab\ncd",
    );
    let stamp = session.buffer().unwrap().stamp();
    session.handle(meta('z'));
    session.execute(Command::Tree(TreeCommand::Focus));
    assert!(session.tree().focused());
    session.handle(control('n'));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.execute(Command::Tree(TreeCommand::Toggle));
    assert!(!session.tree().focused());
    session.handle(character('a'));
    assert_eq!(
        session.buffer().unwrap().stamp().revision,
        stamp.revision + 1
    );
    assert_eq!(session.view().point.position, Scalar(1));
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()[0]
            .inserted()
            .to_string(),
        "a"
    );
}

fn phase(event: Event, kind: crossterm::event::KeyEventKind) -> Event {
    let Event::Key(event) = event else {
        unreachable!()
    };
    Event::Key(KeyEvent { kind, ..event })
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Held prefix/completion keys and releases cannot repeat a completed sequence.
// Repetition of a single-chord motion follows its action, including new aliases.
#[test]
fn configured_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let snapshot = settings(
        "[keys.editor]\n'M-z a' = 'newline'\n'M-j' = 'next-line'\n'M-q' = 'split-window-right'",
    );
    let mut input = Input::with_settings(&snapshot);
    assert_eq!(input.handle(meta('z')), Outcome::Prefix(Prefix::Sequence));
    for event in [
        phase(meta('z'), Repeat),
        phase(character('a'), Repeat),
        phase(character('a'), Release),
    ] {
        assert_eq!(input.handle(event), Outcome::Ignored);
    }
    assert_eq!(
        input.handle(character('a')),
        Outcome::Command(Command::Newline)
    );
    assert_eq!(
        input.handle(phase(character('a'), Repeat)),
        Outcome::Ignored
    );
    assert_eq!(
        input.handle(meta('j')),
        Outcome::Command(Command::Scroll(Scroll::NextLine))
    );
    assert_eq!(
        input.handle(phase(meta('j'), Repeat)),
        Outcome::Command(Command::Scroll(Scroll::NextLine))
    );
    input.handle(meta('q'));
    assert_eq!(input.handle(phase(meta('q'), Repeat)), Outcome::Ignored);
}

// The maximum admitted prefix remains transient until its final chord. One
// completion creates exactly one canonical edit; a held suffix cannot add another.
#[test]
fn bounded_sequence() {
    use crossterm::event::KeyEventKind::Repeat;
    let mut session = editor("[keys.editor]\n'M-z a b c d e f g' = 'newline'", "");
    session.handle(meta('z'));
    for value in ['a', 'b', 'c', 'd', 'e', 'f'] {
        session.handle(character(value));
        assert_eq!(text(&session), "");
        assert!(session.buffer().unwrap().history().is_empty());
    }
    assert_eq!(session.echo(), "M-z a b c d e f");
    session.handle(character('g'));
    session.handle(phase(character('g'), Repeat));
    assert_eq!(text(&session), "\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Literal paste never matches a binding or completes a prefix. The exact
// Unicode/control-looking payload is one edit, and its following key is fresh.
#[test]
fn literal_paste() {
    let mut session = editor("[keys.editor]\n'a' = 'newline'\n'M-z b' = 'find-file'", "");
    session.handle(meta('z'));
    let payload = "aé\nC-x\u{7}\u{1b}b";
    session.handle(Event::Paste(payload.into()));
    assert_eq!(text(&session), payload);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.handle(character('b'));
    assert_eq!(text(&session), format!("{payload}b"));
    assert_eq!(
        session.view().point.position,
        Scalar(payload.chars().count() + 1)
    );
}

// Rejected candidates cannot partially activate an earlier valid alias. Input
// still dispatches complete defaults after unknown commands or prefix collisions.
#[test]
fn rejected_candidate() {
    for config in [
        "[keys.editor]\n'M-j' = 'run-shell'",
        "[keys.editor]\n'M-j' = 'next-line'\n'C-x' = 'newline'",
        "[keys.editor]\n'M-j' = 'next-line'\n'C-f a' = 'newline'",
        "[keys.editor]\n'M-j' = 'next-line'\n'C-i' = 'newline'\n'TAB' = 'forward-char'",
    ] {
        let fixture = super::config_fixture::Fixture::new();
        let startup = prepare(&fixture.write("config.toml", config));
        assert!(startup.diagnostic.is_some(), "{config}");
        let mut input = Input::with_settings(&startup.snapshot);
        assert_eq!(input.handle(meta('j')), Outcome::Message("Unbound key"));
        assert_eq!(
            input.handle(key(KeyCode::Tab)),
            Outcome::Command(Command::InsertTab)
        );
        input.handle(control('x'));
        assert_eq!(
            input.handle(control('s')),
            Outcome::Command(Command::SaveBuffer)
        );
    }
}

// Bound editing actions retain canonical read-only admission. A denied edit
// cannot change text, revision, history, point or saved-state identity.
#[test]
fn readonly_binding() {
    let snapshot = settings("[keys.editor]\n'M-j' = 'newline'");
    let mut buffers = Buffers::with_settings(snapshot).unwrap();
    let id = buffers.create("kept").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let stamp = buffer.stamp();
    let saved = buffer.state();
    let view = View::new(buffer);
    let point = view.point;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    session.handle(meta('j'));
    assert!(session.echo().contains("read-only"));
    assert_eq!(text(&session), "kept");
    assert_eq!(session.view().point, point);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().state(), saved);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}

// Escape expiry consumes the complete pending prefix, while a compatible Meta
// suffix before expiry resolves it once. Resize alone preserves editing prefixes.
#[test]
fn prefix_expiry() {
    use rev::input::META_TIMEOUT;
    use std::time::Instant;
    let snapshot = settings("[keys.editor]\n'M-z M-j' = 'next-line'");
    let mut input = Input::with_settings(&snapshot);
    let now = Instant::now();
    input.handle_at(meta('z'), now);
    input.handle_at(key(KeyCode::Esc), now);
    assert_eq!(input.expire(now + META_TIMEOUT), Outcome::Message("Escape"));
    assert_eq!(input.prefix(), None);
    assert!(matches!(
        input.handle_at(character('j'), now + META_TIMEOUT),
        Outcome::Editing(_)
    ));
    input.handle_at(meta('z'), now + META_TIMEOUT);
    input.handle_at(Event::Resize(10, 5), now + META_TIMEOUT);
    input.handle_at(key(KeyCode::Esc), now + META_TIMEOUT);
    assert_eq!(
        input.handle_at(character('j'), now + META_TIMEOUT),
        Outcome::Command(Command::Scroll(Scroll::NextLine))
    );
}

// Named invocation and a configured chord enter the same edit/history owner.
// Undo through either path restores the original saved document without drift.
#[test]
fn named_parity() {
    let mut bound = editor("[keys.editor]\n'M-j' = 'newline'", "ab");
    let mut named = editor("", "ab");
    bound.handle(meta('j'));
    named.handle(meta('x'));
    named.handle(Event::Paste("newline".into()));
    named.handle(key(KeyCode::Enter));
    assert_eq!(text(&bound), text(&named));
    assert_eq!(bound.view().point.position, named.view().point.position);
    assert_eq!(bound.buffer().unwrap().history().len(), 1);
    assert_eq!(named.buffer().unwrap().history().len(), 1);
    bound.execute(Command::History(rev::command::History::Undo));
    named.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&bound), "ab");
    assert_eq!(text(&named), "ab");
    assert!(!bound.buffer().unwrap().is_modified());
    assert!(!named.buffer().unwrap().is_modified());
}

// Every Enter alias requires a fresh prompt acceptance. Phase admission uses
// the normalized key so a held Control-m/j cannot bypass the Enter safeguard.
#[test]
fn acceptance_aliases() {
    use crossterm::event::KeyEventKind::Repeat;
    for accept in [key(KeyCode::Enter), control('m'), control('j')] {
        let mut input = Input::with_settings(&settings(""));
        input.argument("Text: ", "kept").unwrap();
        input.handle(character('a'));
        assert_eq!(
            input.handle(phase(accept.clone(), Repeat)),
            Outcome::Ignored
        );
        assert_eq!(input.prompt().unwrap().text(), "kepta");
        assert_eq!(
            input.handle(accept.clone()),
            Outcome::Argument("kepta".into())
        );
        assert_eq!(input.handle(phase(accept, Repeat)), Outcome::Ignored);
    }
}
