#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod config_fixture;
#[path = "commands/indent.rs"]
mod indent;
#[path = "commands/keymap.rs"]
mod keymap;
#[path = "commands/preferences.rs"]
mod preferences;

#[path = "commands/flows.rs"]
mod flows;
#[path = "commands/palette.rs"]
mod palette;
#[path = "commands/paste.rs"]
mod paste;
#[path = "commands/terminal.rs"]
mod terminal;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{
    BufferCommand, Command, KillCommand, LinearCommand, RegionCommand, Split, WindowCommand,
};
use rev::input::{Focus, Input, Key, Outcome, Prefix};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn character(value: char) -> Event {
    key(KeyCode::Char(value))
}

fn control(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::CONTROL))
}

fn feed(input: &mut Input, events: Vec<Event>) -> Vec<Outcome> {
    events
        .into_iter()
        .map(|event| input.handle(event))
        .collect()
}

// A complete binding produces the same implemented enum as its canonical name;
// a nested prefix consumes its next key instead of dispatching an absent command.
#[test]
fn command_sequences() {
    let cases = [
        (
            "quit",
            vec![control('x'), control('c')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Quit),
            ],
        ),
        (
            "find file",
            vec![control('x'), control('f')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::FindFile),
            ],
        ),
        (
            "save buffer",
            vec![control('x'), control('s')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::SaveBuffer),
            ],
        ),
        (
            "write file",
            vec![control('x'), control('w')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::WriteFile),
            ],
        ),
        (
            "switch buffer",
            vec![control('x'), character('b')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Buffer(BufferCommand::Switch)),
            ],
        ),
        (
            "list buffers",
            vec![control('x'), control('b')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Buffer(BufferCommand::List)),
            ],
        ),
        (
            "kill buffer",
            vec![control('x'), character('k')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Buffer(BufferCommand::Kill)),
            ],
        ),
        (
            "split below",
            vec![control('x'), character('2')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Split(Split::Below)),
            ],
        ),
        (
            "split right",
            vec![control('x'), character('3')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Split(Split::Right)),
            ],
        ),
        (
            "delete window",
            vec![control('x'), character('0')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Window(WindowCommand::Delete)),
            ],
        ),
        (
            "keep window",
            vec![control('x'), character('1')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Window(WindowCommand::Keep)),
            ],
        ),
        (
            "other window",
            vec![control('x'), character('o')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Window(WindowCommand::Other)),
            ],
        ),
        (
            "cancel",
            vec![control('x'), control('g')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::KeyboardQuit),
            ],
        ),
        (
            "nested",
            vec![control('x'), character('r'), character('x')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Prefix(Prefix::Rectangle),
                Outcome::Message("Undefined key sequence"),
            ],
        ),
    ];
    for (name, events, expected) in cases {
        let mut input = Input::default();
        assert_eq!(feed(&mut input, events), expected, "{name}");
        assert_eq!(input.prefix(), None, "{name}");
    }
    for command in Command::ALL {
        assert_eq!(Command::named(command.name()), Some(command));
        assert!(
            command
                .name()
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        );
    }
    assert_eq!(Command::named("find-file"), Some(Command::FindFile));
    assert_eq!(Command::named("save-buffer"), Some(Command::SaveBuffer));
    assert_eq!(Command::named("write-file"), Some(Command::WriteFile));
    assert_eq!(Command::named(""), None);
}

// Argument input belongs to the prompt until completion; Enter returns its
// value exactly once and restores editing without forwarding that Enter.
#[test]
fn argument_focus() {
    let mut input = Input::default();
    input.argument("Path: ", "é界").unwrap();
    assert_eq!(input.focus(), Focus::Argument);
    assert_eq!(input.label(), Some("Path: "));
    assert_eq!(input.handle(key(KeyCode::Left)), Outcome::Changed);
    assert_eq!(input.handle(character('x')), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text(), "éx界");
    assert_eq!(input.prompt().unwrap().cursor(), "éx".len());
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Argument("éx界".into())
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert!(input.prompt().is_none());
    assert_eq!(
        input.handle(character('z')),
        Outcome::Editing(Key {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::NONE
        })
    );
}

// A decision has no implicit affirmative default. Invalid input retains focus,
// and only an explicit answer produces a result and closes the decision.
#[test]
fn explicit_decisions() {
    for (name, answer, expected) in [
        ("yes", 'y', true),
        ("no", 'n', false),
        ("upper yes", 'Y', true),
        ("upper no", 'N', false),
    ] {
        let mut input = Input::default();
        input.decision("Overwrite? y/n").unwrap();
        assert_eq!(input.label(), Some("Overwrite? y/n"));
        assert_eq!(
            input.handle(key(KeyCode::Enter)),
            Outcome::Message("Answer y or n"),
            "{name}"
        );
        assert_eq!(input.focus(), Focus::Decision);
        assert_eq!(
            input.handle(character(answer)),
            Outcome::Decision(expected),
            "{name}"
        );
        assert_eq!(input.focus(), Focus::Editing);
    }
}

fn phase(mut event: Event, kind: crossterm::event::KeyEventKind) -> Event {
    if let Event::Key(key) = &mut event {
        key.kind = kind;
    }
    event
}

// Cancellation and invalid suffixes consume their keys, discard all pending
// prefix state, and let the next ordinary character reach editing once.
#[test]
fn prefix_recovery() {
    let cases = [
        (
            "nested cancel",
            vec![control('x'), character('r'), control('g')],
            Outcome::Command(Command::KeyboardQuit),
        ),
        (
            "bare cancel",
            vec![control('g')],
            Outcome::Command(Command::KeyboardQuit),
        ),
        (
            "nested invalid",
            vec![control('x'), character('r'), character('q')],
            Outcome::Message("Undefined key sequence"),
        ),
        (
            "invalid suffix",
            vec![control('x'), character('q')],
            Outcome::Message("Undefined key sequence"),
        ),
        (
            "plain suffix",
            vec![control('x'), character('c')],
            Outcome::Message("Undefined key sequence"),
        ),
        (
            "exchange consumes prefix",
            vec![control('x'), control('x')],
            Outcome::Command(Command::Region(RegionCommand::Exchange)),
        ),
    ];
    for (name, events, expected) in cases {
        let mut input = Input::default();
        let outcomes = feed(&mut input, events);
        assert_eq!(outcomes.last(), Some(&expected), "{name}");
        assert_eq!(input.prefix(), None, "{name}");
        assert_eq!(input.focus(), Focus::Editing, "{name}");
        assert_eq!(
            input.handle(character('z')),
            Outcome::Editing(Key {
                code: KeyCode::Char('z'),
                modifiers: KeyModifiers::NONE
            }),
            "{name}"
        );
        assert_eq!(
            input.handle(control('c')),
            Outcome::Message("Unbound key"),
            "{name}"
        );
    }
}

// Releases are inert. Held prefix keys cannot advance or invalidate a chord,
// while repeated ordinary text remains repeatable editing input.
#[test]
fn key_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let mut input = Input::default();
    let events = vec![
        control('x'),
        phase(control('x'), Repeat),
        phase(control('x'), Release),
        phase(control('c'), Repeat),
        control('c'),
        phase(control('c'), Release),
        phase(control('c'), Repeat),
    ];
    assert_eq!(
        feed(&mut input, events),
        [
            Outcome::Prefix(Prefix::ControlX),
            Outcome::Ignored,
            Outcome::Ignored,
            Outcome::Ignored,
            Outcome::Command(Command::Quit),
            Outcome::Ignored,
            Outcome::Ignored
        ]
    );
    let outcomes = feed(
        &mut input,
        vec![
            character('a'),
            phase(character('a'), Repeat),
            phase(character('a'), Release),
        ],
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, Outcome::Editing(_)))
            .count(),
        2
    );
    assert_eq!(outcomes.last(), Some(&Outcome::Ignored));
    input.handle(control('x'));
    assert_eq!(
        input.handle(character('r')),
        Outcome::Prefix(Prefix::Rectangle)
    );
    assert_eq!(
        input.handle(phase(character('r'), Repeat)),
        Outcome::Ignored
    );
    assert_eq!(input.prefix(), Some(Prefix::Rectangle));
}

// A confirmation requires a fresh press; its repeat cannot become a second
// answer or leak into the document after the decision has closed.
#[test]
fn completion_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let mut input = Input::default();
    input.decision("Continue? y/n").unwrap();
    assert_eq!(
        input.handle(phase(character('y'), Repeat)),
        Outcome::Ignored
    );
    assert_eq!(input.focus(), Focus::Decision);
    assert_eq!(input.handle(character('y')), Outcome::Decision(true));
    for event in [
        phase(character('y'), Repeat),
        phase(character('y'), Release),
    ] {
        assert_eq!(input.handle(event), Outcome::Ignored);
    }
    assert!(matches!(input.handle(character('y')), Outcome::Editing(_)));
    input.argument("Path: ", "value").unwrap();
    assert_eq!(
        input.handle(phase(key(KeyCode::Enter), Repeat)),
        Outcome::Ignored
    );
    assert_eq!(input.focus(), Focus::Argument);
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Argument("value".into())
    );
    assert_eq!(
        input.handle(phase(key(KeyCode::Enter), Release)),
        Outcome::Ignored
    );
    assert_eq!(
        input.handle(phase(key(KeyCode::Enter), Repeat)),
        Outcome::Ignored
    );
}

// Character case is already encoded by the terminal; modifiers cannot smuggle
// control bytes into editing. Non-key events leave the pending chord intact.
#[test]
fn normalized_events() {
    let mut input = Input::default();
    assert_eq!(
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('A'),
            KeyModifiers::SHIFT
        ))),
        Outcome::Editing(Key {
            code: KeyCode::Char('A'),
            modifiers: KeyModifiers::NONE
        })
    );
    assert_eq!(
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('X'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        ))),
        Outcome::Prefix(Prefix::ControlX)
    );
    for event in [Event::Resize(0, 0), Event::FocusLost, Event::FocusGained] {
        assert_eq!(input.handle(event), Outcome::Ignored);
        assert_eq!(input.prefix(), Some(Prefix::ControlX));
    }
    assert_eq!(input.handle(control('c')), Outcome::Command(Command::Quit));
    for modifiers in [
        KeyModifiers::ALT,
        KeyModifiers::CONTROL | KeyModifiers::ALT,
        KeyModifiers::SUPER,
    ] {
        assert_eq!(
            input.handle(Event::Key(KeyEvent::new(KeyCode::Char('q'), modifiers))),
            Outcome::Message("Unbound key")
        );
    }
    for value in ['\0', '\u{1b}', '\n', '\u{7f}', '\u{9b}'] {
        assert_eq!(
            input.handle(character(value)),
            Outcome::Message("Control character rejected")
        );
    }
}

// Cursor edits operate on entire graphemes with both Emacs and terminal keys;
// edits that join adjacent clusters repair the prompt cursor before deletion.
#[test]
fn prompt_graphemes() {
    let cases = [
        (
            "emoji deletion",
            "e\u{301}👩\u{200d}💻界",
            vec![key(KeyCode::Left), key(KeyCode::Backspace)],
            "e\u{301}界",
            "e\u{301}".len(),
        ),
        (
            "home delete",
            "e\u{301}界",
            vec![control('a'), control('d')],
            "界",
            0,
        ),
        (
            "backward forward",
            "e\u{301}界",
            vec![control('b'), control('f')],
            "e\u{301}界",
            "e\u{301}界".len(),
        ),
        (
            "end erase",
            "e\u{301}界",
            vec![control('a'), control('e'), control('h')],
            "e\u{301}",
            "e\u{301}".len(),
        ),
        (
            "ZWJ join",
            "👩💻",
            vec![key(KeyCode::Left), character('\u{200d}')],
            "👩\u{200d}💻",
            "👩\u{200d}💻".len(),
        ),
        (
            "joined delete",
            "👩💻",
            vec![
                key(KeyCode::Left),
                character('\u{200d}'),
                key(KeyCode::Backspace),
            ],
            "",
            0,
        ),
        (
            "combining insertion",
            "a界",
            vec![key(KeyCode::Left), character('\u{301}')],
            "a\u{301}界",
            "a\u{301}".len(),
        ),
        (
            "empty boundaries",
            "",
            vec![
                key(KeyCode::Left),
                key(KeyCode::Backspace),
                key(KeyCode::Delete),
                key(KeyCode::Right),
                key(KeyCode::Home),
                key(KeyCode::End),
            ],
            "",
            0,
        ),
    ];
    for (name, initial, events, expected, cursor) in cases {
        let mut input = Input::default();
        input.argument("Text: ", initial).unwrap();
        assert!(
            feed(&mut input, events)
                .iter()
                .all(|outcome| *outcome == Outcome::Changed),
            "{name}"
        );
        let prompt = input.prompt().unwrap();
        assert_eq!(prompt.text(), expected, "{name}");
        assert_eq!(prompt.cursor(), cursor, "{name}");
    }
}

// Scalar limits reject an entire insertion visibly, preserving cursor and text;
// deleting one multibyte grapheme makes exactly one scalar of capacity available.
#[test]
fn prompt_limit() {
    use rev::input::LIMIT;
    let mut input = Input::default();
    let full = "界".repeat(LIMIT);
    input.argument("Path: ", &full).unwrap();
    assert_eq!(
        input.handle(character('é')),
        Outcome::Message("Prompt input limit exceeded")
    );
    assert_eq!(input.prompt().unwrap().text(), full);
    assert_eq!(input.prompt().unwrap().cursor(), full.len());
    assert_eq!(input.handle(key(KeyCode::Backspace)), Outcome::Changed);
    assert_eq!(input.handle(character('é')), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text().chars().count(), LIMIT);
    assert!(input.prompt().unwrap().text().ends_with('é'));
    assert_eq!(
        input.handle(character('\n')),
        Outcome::Message("Control character rejected")
    );
    let accepted = input.handle(key(KeyCode::Enter));
    assert!(matches!(accepted, Outcome::Argument(value) if value.chars().count() == LIMIT));
}

// Rejected prompt acquisition preserves its current owner or pending prefix.
// Opening a valid prompt clears the prefix before its first character arrives.
#[test]
fn prompt_acquisition() {
    use rev::input::{Error, LIMIT};
    let mut input = Input::default();
    input.handle(control('x'));
    assert_eq!(
        input.argument("Path: ", &"x".repeat(LIMIT + 1)),
        Err(Error::Limit)
    );
    assert_eq!(input.argument("Path: ", "bad\npath"), Err(Error::Control));
    assert_eq!(input.focus(), Focus::Editing);
    assert_eq!(input.prefix(), Some(Prefix::ControlX));
    input.argument("Path: ", "kept").unwrap();
    assert_eq!(input.prefix(), None);
    assert_eq!(input.argument("Other: ", "lost"), Err(Error::Busy));
    assert_eq!(input.decision("Replace? y/n"), Err(Error::Busy));
    assert_eq!(input.prompt().unwrap().text(), "kept");
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    input.decision("Delete? y/n").unwrap();
    assert_eq!(input.argument("Path: ", "lost"), Err(Error::Busy));
    assert_eq!(input.label(), Some("Delete? y/n"));
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert_eq!(input.label(), None);
}

fn deliver(
    buffer: &mut rev::buffer::Buffer,
    point: &mut rev::buffer::edit::Point,
    outcome: Outcome,
) {
    if let Outcome::Editing(Key {
        code: KeyCode::Char(value),
        modifiers,
    }) = outcome
    {
        assert!(modifiers.is_empty());
        let edits = [rev::buffer::edit::Replacement {
            source: buffer.span(point.position.0..point.position.0).unwrap(),
            text: value.to_string(),
        }];
        let prepared = buffer
            .prepare(&edits, std::slice::from_ref(point), None)
            .unwrap();
        buffer
            .publish(prepared, std::slice::from_mut(point))
            .unwrap();
    }
}

// The real buffer edit owner receives only routed editing text. Prefixes,
// arguments, confirmations, and cancellation leave its revision/history intact;
// after focus returns, one ordinary key causes exactly one real publication.
#[test]
fn document_isolation() {
    let mut buffers = rev::buffer::Buffers::new().unwrap();
    let id = buffers.create("original").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let stamp = buffer.stamp();
    let mut point = rev::buffer::edit::Point {
        stamp,
        position: rev::coordinates::Scalar(8),
    };
    let mut input = Input::default();
    for event in [control('x'), character('r'), control('g')] {
        deliver(buffer, &mut point, input.handle(event));
    }
    input.argument("Path: ", "").unwrap();
    for event in [control('x'), character('r'), control('c'), control('g')] {
        deliver(buffer, &mut point, input.handle(event));
    }
    input.argument("Path: ", "file").unwrap();
    deliver(buffer, &mut point, input.handle(key(KeyCode::Enter)));
    input.decision("Overwrite? y/n").unwrap();
    deliver(buffer, &mut point, input.handle(character('y')));
    assert_eq!(buffer.stamp(), stamp);
    assert!(buffer.history().is_empty());
    assert_eq!(point.position, rev::coordinates::Scalar(8));
    deliver(buffer, &mut point, input.handle(character('z')));
    assert_eq!(buffer.stamp().revision, stamp.revision + 1);
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(point.position, rev::coordinates::Scalar(9));
    assert_eq!(
        buffer
            .slice(&buffer.span(0..9).unwrap())
            .unwrap()
            .to_string(),
        "originalz"
    );
}

// A repeated final C-x command key cannot leak into the document after the
// prefix is consumed. A distinct new press restores ordinary editing input.
#[test]
fn window_repeats() {
    use crossterm::event::KeyEventKind;
    for (letter, command) in [
        ('0', WindowCommand::Delete),
        ('1', WindowCommand::Keep),
        ('o', WindowCommand::Other),
    ] {
        let mut input = Input::default();
        input.handle(control('x'));
        assert_eq!(
            input.handle(character(letter)),
            Outcome::Command(Command::Window(command))
        );
        let repeat = Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char(letter),
            KeyModifiers::NONE,
            KeyEventKind::Repeat,
        ));
        assert_eq!(input.handle(repeat), Outcome::Ignored);
        assert!(matches!(input.handle(character('x')), Outcome::Editing(_)));
    }
}

// Reported C-SPC, C-@ and NUL aliases reach the same real command. The exchange
// prefix consumes its suffix; neither selection command inserts document text.
#[test]
fn region_bindings() {
    for event in [control(' '), control('@'), key(KeyCode::Null)] {
        let mut input = Input::default();
        assert_eq!(
            input.handle(event),
            Outcome::Command(Command::Region(RegionCommand::Set))
        );
    }
    let mut input = Input::default();
    assert_eq!(
        feed(&mut input, vec![control('x'), control('x')]).last(),
        Some(&Outcome::Command(Command::Region(RegionCommand::Exchange)))
    );
    for command in [RegionCommand::Set, RegionCommand::Exchange] {
        assert_eq!(
            Command::named(command.name()),
            Some(Command::Region(command))
        );
    }
}

// Terminal control/meta reports and canonical names resolve to the same live
// handlers rather than inserting the command suffix into the document.
#[test]
fn kill_bindings() {
    for (code, modifiers, command) in [
        (KeyCode::Char('k'), KeyModifiers::CONTROL, KillCommand::Line),
        (KeyCode::Char('d'), KeyModifiers::ALT, KillCommand::Word),
        (
            KeyCode::Backspace,
            KeyModifiers::ALT,
            KillCommand::BackwardWord,
        ),
    ] {
        let mut input = Input::default();
        assert_eq!(
            input.handle(Event::Key(KeyEvent::new(code, modifiers))),
            Outcome::Command(Command::Kill(command))
        );
        assert_eq!(Command::named(command.name()), Some(Command::Kill(command)));
    }
}

// Region copy/kill and yank bind canonical native commands; modifier handling
// must distinguish M-w from C-w while sharing reported names with the registry.
#[test]
fn linear_bindings() {
    for (character, modifiers, command) in [
        ('w', KeyModifiers::CONTROL, LinearCommand::Kill),
        ('w', KeyModifiers::ALT, LinearCommand::Copy),
        ('y', KeyModifiers::CONTROL, LinearCommand::Yank),
        ('y', KeyModifiers::ALT, LinearCommand::Pop),
    ] {
        let mut input = Input::default();
        assert_eq!(
            input.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(character),
                modifiers
            ))),
            Outcome::Command(Command::Linear(command))
        );
        assert_eq!(
            Command::named(command.name()),
            Some(Command::Linear(command))
        );
    }
}
