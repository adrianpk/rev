use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Mode;
use rev::command::{Command, MarkdownCommand, Motion};
use rev::input::{Input, Outcome};

use super::lists::{at, at_readonly};
use super::*;

// Invalid, readonly, literal or over-limit contexts cannot publish partial list
// edits or history. Canceled evidence and explicit Text mode remain unavailable.
#[test]
fn atomic_refusals() {
    for (name, config, source, needle, readonly, command) in [
        (
            "readonly",
            "",
            "\u{feff}- 界\r\n",
            "界",
            true,
            MarkdownCommand::ContinueItem,
        ),
        (
            "first sibling",
            "",
            "- one\n",
            "one",
            false,
            MarkdownCommand::IndentItem,
        ),
        (
            "outer level",
            "",
            "- one\n",
            "one",
            false,
            MarkdownCommand::OutdentItem,
        ),
        (
            "number limit",
            "",
            "999999999. one\n",
            "one",
            false,
            MarkdownCommand::ContinueItem,
        ),
        (
            "literal",
            "",
            "- Parent\n  ```\n  - literal\n  ```\n",
            "literal",
            false,
            MarkdownCommand::ContinueItem,
        ),
        (
            "after list",
            "",
            "- one\n",
            "",
            false,
            MarkdownCommand::ContinueItem,
        ),
        (
            "cutoff",
            "syntax.max_bytes=5",
            "- one\n",
            "one",
            false,
            MarkdownCommand::ContinueItem,
        ),
    ] {
        let (fixture, mut session) = at_readonly(config, source, needle, readonly);
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point;
        let before = text(session.buffer().unwrap());
        session.execute(Command::Markdown(command));
        assert_eq!(text(session.buffer().unwrap()), before, "{name}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
        assert_eq!(session.view().point, point, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert!(!session.echo().is_empty(), "{name}");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
            source
        );
    }
    for mode in [false, true] {
        let (_fixture, mut session) = editor("", "notes.md", "- one\n");
        session.frame().unwrap();
        session.execute(Command::KeyboardQuit);
        settle(&mut session);
        if mode {
            session.execute(Command::Mode(Mode::Text));
        }
        session.execute(Command::Markdown(MarkdownCommand::ContinueItem));
        assert_eq!(text(session.buffer().unwrap()), "- one\n");
        assert!(session.buffer().unwrap().history().is_empty());
        assert_eq!(session.remaining_input(Instant::now()), None);
    }
}

// Explicit inherited editor/profile mappings beat factory Markdown routes;
// explicit local mappings then override their parent, including Enter aliases.
#[test]
fn override_inheritance() {
    for (profile, layer, local, expected) in [
        ("non-modal", "editor", "", "- one\n"),
        ("modal", "vim_insert", "", "- one\n"),
        (
            "non-modal",
            "editor",
            "keys.markdown.RET='markdown-return'",
            "- one\n- ",
        ),
        (
            "modal",
            "vim_insert",
            "keys.markdown.RET='markdown-return'",
            "- one\n- ",
        ),
    ] {
        let config = format!(
            "editor.interaction_profile='{profile}'\nkeys.{layer}.'C-m'='newline'\n{local}"
        );
        let (_fixture, mut session) = at(&config, "- one", "one");
        if profile == "modal" {
            session.handle(key(KeyCode::Char('i'), KeyModifiers::NONE));
        }
        session.execute(Command::Motion(Motion::LineEnd));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
        settle(&mut session);
        assert_eq!(
            text(session.buffer().unwrap()),
            expected,
            "{profile}/{local}"
        );
    }
}

// Native modal ownership remains above the document overlay. Cancellation
// retires an incomplete custom chord before the next ordinary editor input.
#[test]
fn context_boundaries() {
    let (_fixture, mut session) = at("editor.interaction_profile='modal'", "- one\n", "one");
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(text(session.buffer().unwrap()), "- one\n");
    session.handle(key(KeyCode::Char('v'), KeyModifiers::NONE));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(text(session.buffer().unwrap()), "- one\n");
    let fixture = Fixture::new();
    let config = prepare(&fixture.write(
        "config.toml",
        "keys.markdown.'C-c l'='markdown-continue-item'",
    ));
    assert!(config.diagnostic.is_none());
    let mut input = Input::with_settings(&config.snapshot);
    input.set_markdown_context(true);
    input.handle(key(KeyCode::Char('c'), KeyModifiers::CONTROL));
    input.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(matches!(
        input.handle(key(KeyCode::Char('l'), KeyModifiers::NONE)),
        Outcome::Editing(_)
    ));
}

// M-x and modal colon resolve the same five delivered list names, reaching
// checked source publication rather than a profile-specific shortcut path.
#[test]
fn named_actions() {
    for (name, source, needle, expected) in [
        ("markdown-continue-item", "- one", "one", "- one\n- "),
        ("markdown-return", "- one", "one", "- one\n- "),
        ("markdown-meta-return", "- one", "one", "- one\n- "),
        (
            "markdown-indent-item",
            "- first\n- second",
            "second",
            "- first\n  - second",
        ),
        (
            "markdown-outdent-item",
            "- Parent\n  + child",
            "child",
            "- Parent\n+ child",
        ),
    ] {
        for profile in ["non-modal", "modal"] {
            let (code, modifiers) = if profile == "modal" {
                (KeyCode::Char(':'), KeyModifiers::NONE)
            } else {
                (KeyCode::Char('x'), KeyModifiers::ALT)
            };
            let (_fixture, mut session) = at(
                &format!("editor.interaction_profile='{profile}'"),
                source,
                needle,
            );
            session.handle(key(code, modifiers));
            session.handle(Event::Paste(name.into()));
            session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
            settle(&mut session);
            assert_eq!(
                text(session.buffer().unwrap()),
                expected,
                "{name}/{profile}"
            );
            assert_eq!(session.buffer().unwrap().history().len(), 1);
        }
    }
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}
