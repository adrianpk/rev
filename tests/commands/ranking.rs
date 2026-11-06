use super::{config_fixture::Fixture, control, key};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use rev::command::Command;
use rev::config::prepare;
use rev::input::{Input, Outcome};

fn record(input: &mut Input, name: &str) {
    input.commands().unwrap();
    input.handle(Event::Paste(name.into()));
    let Outcome::Command(command) = input.handle(key(KeyCode::Enter)) else {
        panic!("{name} was not accepted");
    };
    assert_eq!(command.name(), name);
}

fn candidates(input: &mut Input, query: &str) -> Vec<String> {
    input.commands().unwrap();
    input.handle(Event::Paste(query.into()));
    input
        .palette()
        .unwrap()
        .candidates()
        .iter()
        .map(|command| command.name().to_owned())
        .collect()
}

// Literal expected promotions protect distinct recency, count-before-recency
// frequency ties, overlap removal and the exact 256-invocation retention edge.
#[test]
fn usage_groups() {
    let recent = [
        "find-file",
        "toggle-file-tree",
        "query-replace",
        "undo-redo",
    ];
    let frequent: Vec<_> = [
        ("undo", 8),
        ("save-buffer", 7),
        ("newline", 6),
        ("goto-line", 5),
        ("write-file", 4),
        ("revert-buffer", 3),
    ]
    .into_iter()
    .flat_map(|(name, count)| std::iter::repeat_n(name, count))
    .chain(recent)
    .collect();
    let ties = vec![
        "write-file",
        "save-buffer",
        "newline",
        "goto-line",
        "revert-buffer",
        "mouse-mode",
        "undo",
        "undo-redo",
        "toggle-file-tree",
        "find-file",
    ];
    let edge: Vec<_> = std::iter::once("undo")
        .chain(std::iter::repeat_n("newline", 251))
        .chain(recent)
        .collect();
    let evicted: Vec<_> = std::iter::once("undo")
        .chain(std::iter::repeat_n("newline", 252))
        .chain(recent)
        .collect();
    for (case, events, expected) in [
        ("empty", vec![], vec![]),
        ("one", vec!["find-file"], vec!["find-file"]),
        (
            "overlap",
            vec!["undo", "newline", "undo"],
            vec!["undo", "newline"],
        ),
        (
            "frequency",
            frequent,
            vec![
                "undo-redo",
                "query-replace",
                "toggle-file-tree",
                "find-file",
                "undo",
                "save-buffer",
                "newline",
                "goto-line",
            ],
        ),
        (
            "ties",
            ties,
            vec![
                "find-file",
                "toggle-file-tree",
                "undo-redo",
                "undo",
                "mouse-mode",
                "revert-buffer",
                "goto-line",
                "newline",
            ],
        ),
        (
            "retained edge",
            edge,
            vec![
                "undo-redo",
                "query-replace",
                "toggle-file-tree",
                "find-file",
                "newline",
                "undo",
            ],
        ),
        (
            "evicted edge",
            evicted,
            vec![
                "undo-redo",
                "query-replace",
                "toggle-file-tree",
                "find-file",
                "newline",
            ],
        ),
    ] {
        let mut input = Input::default();
        for name in events {
            record(&mut input, name);
        }
        let actual = candidates(&mut input, "");
        assert_eq!(&actual[..expected.len()], expected, "{case}");
        let remainder = &actual[expected.len()..];
        assert!(remainder.windows(2).all(|pair| pair[0] < pair[1]), "{case}");
        assert_eq!(actual.len(), Command::ALL.len(), "{case}");
        for name in expected {
            assert_eq!(
                actual.iter().filter(|other| *other == name).count(),
                1,
                "{case}: {name}"
            );
        }
    }
}

// Match admission and exact names outrank usage in every policy. Filtering
// happens before four/four promotion, so hidden recent names cannot fill slots.
#[test]
fn match_ranks() {
    let fixture = Fixture::new();
    for (policy, query, expected) in [
        (
            "prefix-then-substring",
            "kill",
            vec![
                "kill-word",
                "kill-buffer",
                "kill-line",
                "kill-rectangle",
                "kill-region",
                "kill-ring-save",
                "backward-kill-word",
            ],
        ),
        (
            "prefix",
            "kill",
            vec![
                "kill-word",
                "kill-buffer",
                "kill-line",
                "kill-rectangle",
                "kill-region",
                "kill-ring-save",
            ],
        ),
        (
            "substring",
            "kill",
            vec!["backward-kill-word", "kill-word", "kill-buffer"],
        ),
        ("prefix-then-substring", "undo", vec!["undo", "undo-redo"]),
        ("prefix", "undo", vec!["undo", "undo-redo"]),
        ("substring", "undo", vec!["undo", "undo-redo"]),
        ("substring", "not-a-command", vec![]),
    ] {
        let startup =
            prepare(&fixture.write("config.toml", &format!("minibuffer.match = '{policy}'")));
        assert!(startup.diagnostic.is_none());
        let mut input = Input::with_settings(&startup.snapshot);
        for name in [
            "kill-buffer",
            "kill-word",
            "backward-kill-word",
            "undo-redo",
            "find-file",
            "newline",
            "write-file",
            "goto-line",
        ] {
            record(&mut input, name);
        }
        let actual = candidates(&mut input, query);
        assert_eq!(&actual[..expected.len()], expected, "{policy}/{query}");
        assert!(actual.iter().all(|name| name.contains(query)));
        if expected.is_empty() {
            assert!(actual.is_empty());
        }
    }
}

// Prompt editing, completion, recall, cancellation, unknown names and bound
// commands add no events; custom acceptance adds one despite a held suffix.
#[test]
fn named_events() {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write(
        "config.toml",
        "minibuffer.history_limit = 1\n[keys.minibuffer]\n'M-a' = 'minibuffer-accept'",
    ));
    assert!(startup.diagnostic.is_none());
    let mut input = Input::with_settings(&startup.snapshot);
    for _ in 0..2 {
        record(&mut input, "undo");
    }
    input.commands().unwrap();
    input.handle(Event::Paste("save-buffer".into()));
    let accept = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
    assert_eq!(
        input.handle(Event::Key(accept)),
        Outcome::Command(Command::SaveBuffer)
    );
    input.handle(Event::Key(KeyEvent {
        kind: KeyEventKind::Repeat,
        state: KeyEventState::NONE,
        ..accept
    }));
    for _ in 0..3 {
        input.handle(control('x'));
        assert_eq!(
            input.handle(control('s')),
            Outcome::Command(Command::SaveBuffer)
        );
        input.commands().unwrap();
        input.handle(Event::Paste("save".into()));
        input.handle(key(KeyCode::Down));
        input.handle(key(KeyCode::Tab));
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::ALT,
        )));
        input.handle(control('g'));
        input.commands().unwrap();
        input.handle(Event::Paste("not-a-command".into()));
        assert_eq!(
            input.handle(key(KeyCode::Enter)),
            Outcome::Message("Unknown command")
        );
    }
    for name in ["goto-line", "newline", "write-file", "find-file"] {
        record(&mut input, name);
    }
    let actual = candidates(&mut input, "");
    assert_eq!(
        &actual[..6],
        [
            "find-file",
            "write-file",
            "newline",
            "goto-line",
            "undo",
            "save-buffer"
        ]
    );
}

// Registered AI names use the same bounded identities and ranking path. Pure
// construction starts empty; no provider or user-wide state is needed.
#[test]
fn ai_candidates() {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", "[ai.commands.review]\nprompt='Review text.'\ninput='buffer'\ncontext='input'\noutput='explanation'\n[ai.commands.review-more]\nprompt='Review more text.'\ninput='buffer'\ncontext='input'\noutput='explanation'"));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut input = Input::with_settings(&startup.snapshot);
    record(&mut input, "ai-review-more");
    assert_eq!(
        candidates(&mut input, "ai-review"),
        ["ai-review", "ai-review-more"]
    );
    input.cancel();
    assert_eq!(candidates(&mut input, "")[0], "ai-review-more");
    let mut fresh = Input::default();
    assert!(
        !candidates(&mut fresh, "ai-review")
            .iter()
            .any(|name| name == "ai-review-more")
    );
}
