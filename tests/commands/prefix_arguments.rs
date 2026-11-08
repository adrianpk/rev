use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{ArgumentForm, Command, Motion, Scroll, Split};
use rev::coordinates::Scalar;
use rev::display::Band;
use rev::input::{Input, Outcome};
use rev::session::Session;
use rev::view::{Size, View};

use super::{character, control, key};

fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}

fn editor(config: &str, source: &str, width: u16) -> Session {
    let fixture = super::config_fixture::Fixture::new();
    let prepared = rev::config::prepare(&fixture.write("config.toml", config));
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(buffers, view, Size { width, height: 14 }).unwrap()
}

fn named(session: &mut Session, name: &str) {
    session.handle(meta('x'));
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}

fn count(session: &mut Session, number: &str) {
    session.handle(control('u'));
    for value in number.chars() {
        session.handle(character(value));
    }
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while session
        .remaining_input(Instant::now())
        .is_some_and(|wait| wait == Duration::ZERO)
    {
        assert!(Instant::now() < deadline, "native work did not settle");
        session.expire_input(Instant::now());
    }
}

// Raw forms survive entry and one resolved invocation, including numeric zero.
#[test]
fn entry_forms() {
    let cases = [
        (
            "plain",
            vec![control('u')],
            ArgumentForm::PlainPrefix { depth: 1 },
            4,
        ),
        (
            "depth",
            vec![control('u'), control('u')],
            ArgumentForm::PlainPrefix { depth: 2 },
            16,
        ),
        (
            "integer",
            vec![control('u'), character('4')],
            ArgumentForm::Integer(4),
            4,
        ),
        (
            "meta digits",
            vec![meta('1'), character('2'), meta('3')],
            ArgumentForm::Integer(123),
            123,
        ),
        (
            "negative",
            vec![control('u'), character('-'), character('3')],
            ArgumentForm::Integer(-3),
            -3,
        ),
        (
            "negative only",
            vec![meta('-')],
            ArgumentForm::NegativeOnly,
            -1,
        ),
        ("zero", vec![meta('0')], ArgumentForm::Integer(0), 0),
        (
            "negative zero",
            vec![meta('-'), character('0')],
            ArgumentForm::Integer(0),
            0,
        ),
        (
            "signed zero digits",
            vec![meta('-'), character('0'), character('2')],
            ArgumentForm::Integer(-2),
            -2,
        ),
    ];
    for (name, events, form, value) in cases {
        let mut input = Input::default();
        for event in events {
            assert!(
                matches!(input.handle(event), Outcome::ArgumentPrefix(_)),
                "{name}"
            );
        }
        let argument = input.prefix_argument();
        assert_eq!(argument.form(), form, "{name}");
        assert_eq!(argument.value(), value, "{name}");
        let Outcome::Invocation(invocation) = input.handle(control('f')) else {
            panic!("{name}: missing typed invocation");
        };
        assert_eq!(invocation.argument, argument, "{name}");
        assert_eq!(invocation.command, Command::Motion(Motion::ForwardChar));
        assert!(!input.prefix_argument().supplied());
        assert_eq!(
            input.handle(control('f')),
            Outcome::Command(Command::Motion(Motion::ForwardChar))
        );
    }
}

// Entry remains constant-size; failed arithmetic and invalid sequences retire it.
#[test]
fn entry_bounds() {
    let mut input = Input::default();
    input.handle(meta('0'));
    for _ in 0..1024 {
        input.handle(character('0'));
    }
    assert_eq!(input.prefix_argument().value(), 0);
    input.handle(control('g'));
    input.handle(control('u'));
    for value in "1000000".chars() {
        input.handle(character(value));
    }
    assert_eq!(input.prefix_argument().value(), 1_000_000);
    assert!(matches!(input.handle(character('1')), Outcome::Message(_)));
    assert!(!input.prefix_argument().supplied());
    for _ in 0..9 {
        assert!(matches!(
            input.handle(control('u')),
            Outcome::ArgumentPrefix(_)
        ));
    }
    assert!(matches!(input.handle(control('u')), Outcome::Message(_)));
    assert!(!input.prefix_argument().supplied());
    input.handle(control('u'));
    input.handle(control('x'));
    assert!(matches!(input.handle(character('?')), Outcome::Message(_)));
    assert!(!input.prefix_argument().supplied());
}

// Numeric termination releases the next literal digit to its ordinary owner.
#[test]
fn digit_termination() {
    let mut input = Input::default();
    for event in [control('u'), character('6'), character('4'), control('u')] {
        input.handle(event);
    }
    assert_eq!(input.prefix_argument().value(), 64);
    let Outcome::TextInvocation(invocation) = input.handle(character('1')) else {
        panic!("terminated digits must retain their text argument");
    };
    assert_eq!(invocation.text, "1");
    assert_eq!(invocation.argument.value(), 64);
    assert!(!input.prefix_argument().supplied());
}

// Named helpers use the same state, alias identity and single-digit prompt.
#[test]
fn named_helpers() {
    let mut session = editor("", "abcdefghij", 80);
    named(&mut session, "universal-argument");
    named(&mut session, "digit-argument");
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.band() == Band::Echo && row.text().contains("Digit argument:"))
    );
    session.handle(character('3'));
    session.handle(key(KeyCode::Enter));
    assert!(session.echo().contains("Argument 3"), "{}", session.echo());
    named(&mut session, "forward-char");
    assert_eq!(session.view().point.position, Scalar(3));
    assert_eq!(
        Command::PrefixArgument.description(),
        Command::UniversalArgument.description()
    );
    assert_eq!(
        rev::command::Binding::named("universal-argument"),
        rev::command::Binding::named("prefix-argument")
    );
    assert_eq!(text(&session), "abcdefghij");
    assert!(session.buffer().unwrap().history().is_empty());
}

// Selector edits, paste, arrows, completion and redraw cannot consume a count.
#[test]
fn named_transfer() {
    let mut session = editor("", "abcdefghijklmnop", 80);
    count(&mut session, "5");
    session.handle(meta('x'));
    session.handle(Event::Paste("forward-cha".into()));
    session.handle(key(KeyCode::Up));
    session.handle(key(KeyCode::Tab));
    let stamp = session.buffer().unwrap().stamp();
    session.frame().unwrap();
    session.handle(Event::Resize(90, 16));
    session.handle(key(KeyCode::Enter));
    assert_eq!(session.view().point.position, Scalar(5));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(6));
}

// Quit, unknown names, prompt cancellation and view departure cannot leak input.
#[test]
fn retired_arguments() {
    let mut session = editor("", "abcdef", 80);
    for query in [None, Some("not-a-command")] {
        count(&mut session, "5");
        session.handle(meta('x'));
        if let Some(query) = query {
            session.handle(Event::Paste(query.into()));
            session.handle(key(KeyCode::Enter));
        } else {
            session.handle(control('g'));
        }
        assert_eq!(session.view().point.position, Scalar(0));
    }
    count(&mut session, "5");
    session.handle(control('x'));
    session.handle(control('f'));
    session.handle(control('g'));
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(1));
    session.execute(Command::Split(Split::Right));
    let views = session.windows().order();
    count(&mut session, "4");
    session.select_view(views[1]).unwrap();
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(2));
}

// Native movement keeps grapheme/CRLF and word boundaries and reverses signs.
#[test]
fn signed_movement() {
    let mut session = editor("", "a\u{301}🙂\r\none two three", 80);
    let stamp = session.buffer().unwrap().stamp();
    count(&mut session, "3");
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(5));
    count(&mut session, "-2");
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(2));
    count(&mut session, "0");
    session.handle(control('b'));
    assert_eq!(session.view().point.position, Scalar(2));
    session.execute(Command::Motion(Motion::BufferStart));
    count(&mut session, "3");
    session.handle(meta('f'));
    assert_eq!(session.view().point.position, Scalar(12));
    count(&mut session, "-2");
    session.handle(meta('f'));
    assert_eq!(session.view().point.position, Scalar(5));
    count(&mut session, "1000000");
    session.handle(control('b'));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(0));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Row counts use each pane's ordinary wrapped/logical units and desired column.
#[test]
fn counted_rows() {
    for wrap in [false, true] {
        let config = format!("editor.soft_wrap = {wrap}\ngutter.visible = false");
        let source = "abcdefghijklmno\nxy\nABCDEFGHIJKLMNO\nend";
        let mut counted = editor(&config, source, 10);
        let mut ordinary = editor(&config, source, 10);
        for session in [&mut counted, &mut ordinary] {
            session.execute(Command::Motion(Motion::ForwardChar));
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        count(&mut counted, "3");
        counted.handle(control('n'));
        for _ in 0..3 {
            ordinary.handle(control('n'));
        }
        assert_eq!(
            counted.view().point.position,
            ordinary.view().point.position,
            "wrap={wrap}"
        );
        assert_eq!(counted.view().column, ordinary.view().column, "wrap={wrap}");
        count(&mut counted, "-2");
        counted.handle(control('n'));
        ordinary.handle(control('p'));
        ordinary.handle(control('p'));
        assert_eq!(
            counted.view().point.position,
            ordinary.view().point.position,
            "wrap={wrap}"
        );
        assert_eq!(text(&counted), source);
    }
}

// Effective aliases and dual-role continuations use admitted maps without delay.
#[test]
fn effective_bindings() {
    let config = "[keys.editor]\n'M-q' = 'universal-argument'\n'M-q s' = 'save-buffer'\n'C-u z' = 'forward-char'\n'M-5' = 'backward-char'";
    let mut session = editor(config, "abcdefghij", 80);
    session.handle(meta('q'));
    session.handle(character('3'));
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(3));
    session.handle(meta('5'));
    assert_eq!(session.view().point.position, Scalar(2));
    session.handle(control('u'));
    session.handle(character('z'));
    assert_eq!(session.view().point.position, Scalar(3));
    session.handle(meta('x'));
    session.handle(Event::Paste("universal-argument".into()));
    let frame = session.frame().unwrap();
    let row = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion)
        .unwrap();
    assert!(row.text().contains("[C-u]"), "{}", row.text());
}

// Existing shell/Org continuations still resolve exactly once at input level.
#[test]
fn existing_continuations() {
    let mut input = Input::default();
    input.handle(control('u'));
    assert_eq!(
        input.handle(meta('!')),
        Outcome::Command(Command::ShellCommandInsert)
    );
    assert!(!input.prefix_argument().supplied());
    input.handle(control('u'));
    assert_eq!(
        input.handle(meta('|')),
        Outcome::Command(Command::ShellFilterRegion)
    );
    assert!(!input.prefix_argument().supplied());
    for (role, events, command) in [
        (
            rev::input::OrgContext::Editing,
            vec![control('u'), control('c'), control('v'), character('t')],
            "org-babel-tangle-single-block",
        ),
        (
            rev::input::OrgContext::Agenda,
            vec![control('u'), character('K')],
            "org-habit-toggle-habits-only-for-today",
        ),
    ] {
        input.set_org_context(Some(role));
        let mut outcome = Outcome::Ignored;
        for event in events {
            outcome = input.handle(event);
        }
        assert_eq!(outcome, Outcome::Command(Command::named(command).unwrap()));
        assert!(!input.prefix_argument().supplied());
    }
}

// Each accepted name records one semantic action, including the helper alias.
#[test]
fn usage_identity() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.0.join("command-history.toml");
    let history = rev::command::usage::History::open(&path);
    let mut session = editor("", "abcdefghij", 80).with_command_usage(history);
    named(&mut session, "universal-argument");
    named(&mut session, "forward-char");
    assert_eq!(session.view().point.position, Scalar(4));
    session.flush_command_usage().unwrap();
    let stored: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let names = stored["commands"].as_array().unwrap();
    assert_eq!(names.len(), 2);
    assert_eq!(names[0].as_str(), Some("prefix-argument"));
    assert_eq!(names[1].as_str(), Some("forward-char"));
}

// Deferred planning completes at EOF and revokes a destination after source edits.
#[test]
fn deferred_movement() {
    let mut session = editor("", &"x".repeat(3000), 80);
    count(&mut session, "1000000");
    session.handle(control('f'));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(3000));
    session.execute(Command::Motion(Motion::BufferStart));
    count(&mut session, "1000000");
    session.handle(control('f'));
    session.replace(0..0, "z").unwrap();
    let position = session.view().point.position;
    settle(&mut session);
    assert_eq!(session.view().point.position, position);
    assert_eq!(text(&session).len(), 3001);
    session.handle(control('f'));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(position.0 + 1));
}

// A fresh physical repeat moves once; large pending traversal admits cancellation.
#[test]
fn repeat_and_cancel() {
    let mut session = editor("", &"x".repeat(100_000), 80);
    count(&mut session, "3");
    session.handle(control('f'));
    let mut repeat = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
    repeat.kind = KeyEventKind::Repeat;
    session.handle(Event::Key(repeat));
    assert_eq!(session.view().point.position, Scalar(4));
    let stamp = session.buffer().unwrap().stamp();
    count(&mut session, "1000000");
    session.handle(control('f'));
    assert_eq!(session.view().point.position, Scalar(4));
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(4));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.handle(control('f'));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(5));
}

// Pending input introduces no periodic deadline, and modal C-u keeps its owner.
#[test]
fn idle_and_modal() {
    let mut input = Input::default();
    input.handle(control('u'));
    assert_eq!(input.remaining(Instant::now()), None);
    let mut modal = editor(
        "editor.interaction_profile = 'modal'",
        &"line\n".repeat(80),
        80,
    );
    modal.handle(control('u'));
    assert!(!modal.echo().contains("Argument"));
    named(&mut modal, "prefix-argument");
    modal.handle(key(KeyCode::Esc));
    assert!(!modal.echo().contains("Argument"));
    named(&mut modal, "next-line");
    assert_eq!(modal.view().point.position, Scalar(5));
    assert!(matches!(
        Command::named("next-line"),
        Some(Command::Scroll(Scroll::NextLine))
    ));
}

// A terminated digit is source text with its original numeric argument, and
// the next physical edit uses ordinary single-entry behavior.
#[test]
fn counted_text_transfer() {
    let mut session = editor("", "", 80);
    count(&mut session, "5");
    session.handle(control('u'));
    session.handle(character('7'));
    settle(&mut session);
    assert_eq!(text(&session), "77777");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.handle(character('x'));
    assert_eq!(text(&session), "77777x");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}

// Named counted edits record one accepted command and consume their argument;
// native history replays those existing groups without synthetic grouping.
#[test]
fn counted_edit_selection() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.0.join("command-history.toml");
    let history = rev::command::usage::History::open(&path);
    let mut session = editor("", "a\r\nb", 80).with_command_usage(history);
    count(&mut session, "2");
    named(&mut session, "newline");
    settle(&mut session);
    assert_eq!(text(&session), "\r\n\r\na\r\nb");
    named(&mut session, "newline");
    settle(&mut session);
    assert_eq!(text(&session), "\r\n\r\n\r\na\r\nb");
    count(&mut session, "2");
    named(&mut session, "undo");
    settle(&mut session);
    assert_eq!(text(&session), "a\r\nb");
    assert!(session.buffer().unwrap().history().is_empty());
    session.flush_command_usage().unwrap();
    let stored: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let names = stored["commands"].as_array().unwrap();
    assert_eq!(
        names
            .iter()
            .filter(|name| name.as_str() == Some("newline"))
            .count(),
        2
    );
    assert_eq!(
        names
            .iter()
            .filter(|name| name.as_str() == Some("undo"))
            .count(),
        1
    );
}
