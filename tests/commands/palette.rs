use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Buffers, edit::Replacement};
use rev::command::{Command, KillCommand, LinearCommand, Motion, Split};
use rev::coordinates::Scalar;
use rev::input::{Focus, Input, Outcome};
use rev::session::{Action, Session};
use rev::view::{Size, View};

fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}

fn open() -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
}

fn editor() -> Session {
    Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap()
}

fn named(session: &mut Session, name: &str) -> Action {
    assert_eq!(session.handle(open()), Action::Redraw);
    assert_eq!(session.handle(Event::Paste(name.into())), Action::Redraw);
    session.handle(super::key(KeyCode::Enter))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn populated(readonly: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.initial();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "abc def\nlast".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.mark_saved(buffer.stamp()).unwrap();
    buffer.set_readonly(readonly);
    let mut view = View::new(buffer);
    view.point.position = Scalar(1);
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

fn surface(session: &mut Session) -> Vec<String> {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|row| row.text().to_owned())
        .collect()
}

// Ranking is stable and fixed-name only: prefixes precede lexical substrings;
// no expression, case folding or language completion changes the catalog.
#[test]
fn filter_order() {
    let cases = [
        (
            "kill",
            vec![
                "kill-buffer",
                "kill-line",
                "kill-rectangle",
                "kill-region",
                "kill-ring-save",
                "kill-word",
                "backward-kill-word",
                "copy-rectangle-as-kill",
                "save-buffers-kill-terminal",
            ],
        ),
        ("window-right", vec!["split-window-right"]),
        ("WINDOW", vec![]),
        ("undo; newline", vec![]),
    ];
    for (query, expected) in cases {
        let mut input = Input::default();
        input.commands().unwrap();
        input.handle(Event::Paste(query.into()));
        let actual: Vec<_> = input
            .palette()
            .unwrap()
            .candidates()
            .into_iter()
            .map(Command::name)
            .collect();
        assert_eq!(actual, expected, "{query}");
        assert_eq!(input.focus(), Focus::Command);
        assert_eq!(input.label(), Some("M-x "));
    }
}

// Tab completes the selected name without cycling. Navigation retains the
// original filter and Return consumes the explicitly selected command once.
#[test]
fn selected_completion() {
    let mut input = Input::default();
    assert_eq!(
        input.handle(open()),
        Outcome::Command(Command::ExecuteExtended)
    );
    input.commands().unwrap();
    input.handle(Event::Paste("split-window".into()));
    for (event, expected) in [
        (super::key(KeyCode::Tab), "split-window-below"),
        (super::key(KeyCode::Tab), "split-window-below"),
        (super::key(KeyCode::Down), "split-window-below"),
        (super::key(KeyCode::Tab), "split-window-right"),
    ] {
        assert_eq!(input.handle(event), Outcome::Changed);
        assert_eq!(input.prompt().unwrap().text(), expected);
    }
    assert_eq!(
        input.handle(super::key(KeyCode::Enter)),
        Outcome::Command(Command::Split(Split::Right))
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert!(input.prompt().is_none());
}

// M-x reaches the real edit/history handlers, including redo with no key binding;
// query characters and command acceptance never become document insertions.
#[test]
fn named_history() {
    let mut session = editor();
    named(&mut session, "newline");
    assert_eq!(text(&session), "\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    named(&mut session, "undo");
    assert_eq!(text(&session), "");
    named(&mut session, "undo-redo");
    assert_eq!(text(&session), "\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Empty or unknown names close the prompt with a diagnostic, preserve text and
// qualified point/state, and leave the next fresh editing press usable.
#[test]
fn unknown_names() {
    let cases = [
        ("", "No command entered"),
        ("absent-command", "Unknown command"),
        ("newline;save-buffers-kill-terminal", "Unknown command"),
        ("FIND", "Unknown command"),
        ("界", "Unknown command"),
    ];
    for (query, message) in cases {
        let mut session = populated(false);
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        assert_eq!(named(&mut session, query), Action::Redraw, "{query}");
        assert_eq!(session.echo(), message, "{query}");
        assert_eq!(text(&session), "abc def\nlast", "{query}");
        assert_eq!(*session.view(), view, "{query}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{query}");
        assert_eq!(session.buffer().unwrap().state(), state, "{query}");
        session.handle(super::character('!'));
        assert_eq!(text(&session), "a!bc def\nlast", "{query}");
    }
}

// No-match Tab and empty history retain command focus; Return then reports an
// unknown name without leaving a selected candidate or a pending argument.
#[test]
fn empty_candidates() {
    let mut input = Input::default();
    input.commands().unwrap();
    assert_eq!(
        input.handle(meta('p')),
        Outcome::Message("Command history is empty")
    );
    input.handle(Event::Paste("absent-command".into()));
    assert!(input.palette().unwrap().candidates().is_empty());
    assert_eq!(input.palette().unwrap().selected(), None);
    assert_eq!(
        input.handle(super::key(KeyCode::Tab)),
        Outcome::Message("No matching command")
    );
    assert_eq!(input.focus(), Focus::Command);
    assert_eq!(input.prompt().unwrap().text(), "absent-command");
    assert_eq!(
        input.handle(super::key(KeyCode::Enter)),
        Outcome::Message("Unknown command")
    );
    assert_eq!(input.focus(), Focus::Editing);
}

// History retains distinct accepted names in recency order, clamps at its end,
// restores the draft, detaches edited entries and stays local to one input owner.
#[test]
fn command_history() {
    let mut input = Input::default();
    for name in ["forward-char", "newline", "forward-char"] {
        input.commands().unwrap();
        input.handle(Event::Paste(name.into()));
        assert!(matches!(
            input.handle(super::key(KeyCode::Enter)),
            Outcome::Command(_)
        ));
    }
    input.commands().unwrap();
    input.handle(Event::Paste("draft".into()));
    let steps = [
        (meta('p'), "forward-char"),
        (meta('p'), "newline"),
        (meta('p'), "newline"),
        (meta('n'), "forward-char"),
        (meta('n'), "draft"),
        (meta('p'), "forward-char"),
        (super::key(KeyCode::Backspace), "forward-cha"),
        (meta('n'), "forward-cha"),
        (meta('p'), "forward-char"),
        (meta('n'), "forward-cha"),
    ];
    for (event, expected) in steps {
        assert_eq!(input.handle(event), Outcome::Changed);
        assert_eq!(input.prompt().unwrap().text(), expected);
    }
    input.handle(super::control('g'));
    input.commands().unwrap();
    input.handle(meta('p'));
    assert_eq!(input.prompt().unwrap().text(), "forward-char");
    let mut other = Input::default();
    other.commands().unwrap();
    assert_eq!(
        other.handle(meta('p')),
        Outcome::Message("Command history is empty")
    );
}

// Both direct Meta and a separate Escape reach M-x. Release/held Return cannot
// execute twice, while cancel consumes the query and permits fresh editing input.
#[test]
fn entry_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    use std::time::Instant;
    for (name, events) in [
        ("Alt", vec![open()]),
        (
            "Escape",
            vec![super::key(KeyCode::Esc), super::character('x')],
        ),
    ] {
        let mut session = populated(false);
        let now = Instant::now();
        for event in events {
            session.handle_at(event, now);
        }
        session.handle(Event::Paste("newline".into()));
        for kind in [Release, Repeat] {
            assert_eq!(
                session.handle(super::phase(super::key(KeyCode::Enter), kind)),
                Action::Wait,
                "{name}"
            );
        }
        session.handle(super::key(KeyCode::Enter));
        assert_eq!(text(&session), "a\nbc def\nlast", "{name}");
        assert_eq!(
            session.handle(super::phase(super::key(KeyCode::Enter), Repeat)),
            Action::Wait,
            "{name}"
        );
        session.handle(open());
        session.handle(Event::Paste("delete-char".into()));
        session.handle(super::control('g'));
        assert_eq!(text(&session), "a\nbc def\nlast", "{name}");
        session.handle(super::character('!'));
        assert_eq!(text(&session), "a\n!bc def\nlast", "{name}");
    }
}

// Query edits invalidate the previous completion filter. Oversized or control paste
// rejects wholly, and another focused owner cannot be replaced by M-x entry.
#[test]
fn query_admission() {
    let mut input = Input::default();
    input.commands().unwrap();
    input.handle(Event::Paste("split-window".into()));
    input.handle(super::key(KeyCode::Tab));
    input.handle(super::key(KeyCode::Backspace));
    assert_eq!(input.prompt().unwrap().text(), "split-window-belo");
    for payload in ["bad\ntext".to_owned(), "x".repeat(4097)] {
        assert!(matches!(
            input.handle(Event::Paste(payload)),
            Outcome::Message(_)
        ));
        assert_eq!(input.prompt().unwrap().text(), "split-window-belo");
    }
    input.handle(super::key(KeyCode::Tab));
    assert_eq!(
        input.handle(super::key(KeyCode::Enter)),
        Outcome::Command(Command::Split(Split::Below))
    );
    for focus in [Focus::Argument, Focus::Decision, Focus::Inventory] {
        match focus {
            Focus::Argument => input.argument("Path: ", "kept").unwrap(),
            Focus::Decision => input.decision("Continue?").unwrap(),
            _ => input.inventory().unwrap(),
        }
        assert!(input.commands().is_err());
        assert!(matches!(input.handle(open()), Outcome::Message(_)));
        assert_eq!(input.focus(), focus);
        input.cancel();
    }
}

// Every advertised name reaches its actual shared handler with the same visible
// result as direct enum dispatch. Missing handlers cannot pass as registry stubs.
#[test]
fn catalog_dispatch() {
    for command in Command::ALL {
        let mut direct = editor();
        let mut palette = editor();
        let action = direct.execute(command);
        assert_ne!(
            direct.echo(),
            "Command is not a text edit",
            "{}",
            command.name()
        );
        assert_eq!(
            action,
            named(&mut palette, command.name()),
            "{}",
            command.name()
        );
        assert_eq!(text(&direct), text(&palette), "{}", command.name());
        assert_eq!(
            direct.view().point.position,
            palette.view().point.position,
            "{}",
            command.name()
        );
        // Accepted exit has no subsequent redraw; startup echo is not an
        // observable handler result once both routes have returned Exit.
        if action != Action::Exit {
            assert_eq!(
                surface(&mut direct),
                surface(&mut palette),
                "{}",
                command.name()
            );
        }
        assert_ne!(
            palette.echo(),
            "Command is not a text edit",
            "{}",
            command.name()
        );
    }
}

// Real bound file/window/edit/history paths and named entry produce equivalent
// frames and state, including commands that continue into focused arguments.
#[test]
fn binding_parity() {
    let cases = [
        ("find-file", vec![super::control('x'), super::control('f')]),
        ("write-file", vec![super::control('x'), super::control('w')]),
        (
            "save-buffer",
            vec![super::control('x'), super::control('s')],
        ),
        (
            "switch-to-buffer",
            vec![super::control('x'), super::character('b')],
        ),
        (
            "list-buffers",
            vec![super::control('x'), super::control('b')],
        ),
        (
            "split-window-right",
            vec![super::control('x'), super::character('3')],
        ),
        (
            "delete-other-windows",
            vec![super::control('x'), super::character('1')],
        ),
        (
            "other-window",
            vec![super::control('x'), super::character('o')],
        ),
        ("newline", vec![super::key(KeyCode::Enter)]),
        ("insert-tab", vec![super::key(KeyCode::Tab)]),
        ("backward-delete-char", vec![super::key(KeyCode::Backspace)]),
        ("delete-char", vec![super::control('d')]),
        ("forward-char", vec![super::control('f')]),
        ("undo", vec![super::control('x'), super::character('u')]),
    ];
    for (name, events) in cases {
        let mut bound = populated(false);
        let mut palette = populated(false);
        bound.execute(Command::Split(Split::Right));
        palette.execute(Command::Split(Split::Right));
        let mut action = Action::Wait;
        for event in events {
            action = bound.handle(event);
        }
        assert_eq!(action, named(&mut palette, name), "{name}");
        assert_eq!(surface(&mut bound), surface(&mut palette), "{name}");
        assert_eq!(text(&bound), text(&palette), "{name}");
        assert_eq!(
            bound.buffer().unwrap().is_modified(),
            palette.buffer().unwrap().is_modified(),
            "{name}"
        );
    }
}

// Existing path prompts continue after name acceptance, and cancellation at the
// argument stage leaves the source and selected view usable without disk effects.
#[test]
fn argument_continuation() {
    for (name, label) in [
        ("find-file", "Find file:"),
        ("write-file", "Write file:"),
        ("save-buffer", "Write file:"),
    ] {
        let mut session = populated(false);
        let before = session.view().clone();
        named(&mut session, name);
        assert!(
            surface(&mut session).last().unwrap().starts_with(label),
            "{name}"
        );
        session.handle(Event::Paste("unused-é.txt".into()));
        session.handle(super::control('g'));
        assert_eq!(text(&session), "abc def\nlast", "{name}");
        assert_eq!(*session.view(), before, "{name}");
        session.handle(super::character('!'));
        assert_eq!(text(&session), "a!bc def\nlast", "{name}");
    }
}

// The M-x wrapper is transient: a valid named yank-pop preserves the captured
// range through query entry, while C-g explicitly ends that continuity.
#[test]
fn named_yank() {
    let mut session = editor();
    session.replace(0..0, "aa bb").unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Kill(KillCommand::Word));
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Kill(KillCommand::Word));
    session.execute(Command::Linear(LinearCommand::Yank));
    assert_eq!(text(&session), " bb");
    named(&mut session, "yank-pop");
    assert_eq!(text(&session), "aa");
    assert_eq!(session.echo(), "");
    session.execute(Command::Linear(LinearCommand::Yank));
    session.handle(open());
    session.handle(Event::Paste("yank-pop".into()));
    session.handle(super::control('g'));
    let before = text(&session);
    named(&mut session, "yank-pop");
    assert_eq!(session.echo(), "No current yank to replace");
    assert_eq!(text(&session), before);
}

// Named text mutation observes the same read-only checks, without a revision or
// history change. Query rendering remains bounded across tiny and zero geometry.
#[test]
fn admission_geometry() {
    for name in ["newline", "insert-tab", "delete-char", "kill-line"] {
        let mut session = populated(true);
        let view = session.view().clone();
        let history = session.buffer().unwrap().history().len();
        named(&mut session, name);
        assert_eq!(text(&session), "abc def\nlast", "{name}");
        assert_eq!(*session.view(), view, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), history, "{name}");
    }
    let mut session = populated(false);
    session.handle(open());
    session.handle(Event::Paste("window-right".into()));
    for (width, height) in [(1, 1), (0, 0), (80, 24)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows().len(), usize::from(height));
        assert!(frame.rows().iter().all(|row| row.row() < height));
    }
    assert!(
        surface(&mut session)
            .last()
            .unwrap()
            .contains("split-window-right")
    );
    session.handle(super::key(KeyCode::Enter));
    assert_eq!(session.windows().iter().len(), 2);
    assert_eq!(text(&session), "abc def\nlast");
}

// Public name and binding catalogs must match the live registry exactly once.
// Both formats use this oracle so documentation cannot silently hide drift.
#[test]
fn catalog_reference() {
    use std::collections::BTreeSet;
    let live = Command::ALL
        .into_iter()
        .map(Command::name)
        .collect::<BTreeSet<_>>();
    assert_eq!(Command::ALL.len(), live.len(), "duplicate registry name");
    let cases = [
        (
            "catalog",
            include_str!("../../docs/reference/commands.md"),
            None,
        ),
        (
            "bindings",
            include_str!("../../docs/reference/bindings.md")
                .split("## Configuration keymap contracts")
                .next()
                .unwrap(),
            Some(2),
        ),
    ];
    for (name, source, column) in cases {
        let names = source
            .lines()
            .filter(|line| line.starts_with('|'))
            .map(|line| column.map_or(line, |index| line.split('|').nth(index).unwrap()))
            .flat_map(|line| line.split('`').skip(1).step_by(2))
            .collect::<Vec<_>>();
        let documented = names.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(
            names.len(),
            documented.len(),
            "{name}: duplicate documented name"
        );
        assert_eq!(documented, live, "{name}");
    }
}

// Independent context tables may share global command names. Each table must
// still document its own complete default target set exactly once.
#[test]
fn context_reference() {
    use rev::input::keymap::Context;
    use std::collections::BTreeSet;
    let source = include_str!("../../docs/reference/bindings.md");
    for (context, heading) in [
        (
            Context::Minibuffer,
            "### Minibuffer defaults and availability",
        ),
        (Context::Tree, "### Tree defaults and availability"),
    ] {
        let section = source
            .split_once(heading)
            .unwrap()
            .1
            .split("\n### ")
            .next()
            .unwrap();
        let names = section
            .lines()
            .filter(|line| line.starts_with('|'))
            .flat_map(|line| {
                line.split('|')
                    .nth(2)
                    .unwrap()
                    .split('`')
                    .skip(1)
                    .step_by(2)
            })
            .collect::<Vec<_>>();
        let documented = names.iter().copied().collect::<BTreeSet<_>>();
        let expected = context
            .defaults()
            .iter()
            .map(|(_, target)| *target)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names.len(),
            documented.len(),
            "{}: duplicate target",
            context.name()
        );
        assert_eq!(documented, expected, "{}", context.name());
    }
}
