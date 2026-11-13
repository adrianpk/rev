use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, History, MarkdownCommand as C, Motion, Split};
use rev::coordinates::{Byte, Coordinates};

use super::lists::{at, at_readonly};
use super::*;

const ALIGNED: &str = "| Name | Value |\n| :--- | ----: |\n| 界é  |     7 |\n";

fn action(session: &mut Session, command: C) {
    session.execute(Command::Markdown(command));
    settle(session);
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn dimensions(session: &mut Session, draft: &str) {
    session.handle(Event::Paste(draft.into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(session);
}

// GFM admission, raw escapes/markup, container prefixes and short body rows
// survive canonical alignment; one undo/redo retains the exact source and point.
#[test]
fn source_alignment() {
    for (name, source, needle, expected) in [
        ("unicode", "Name|Value\n:---|---:\n界é|7\n", "界", ALIGNED),
        (
            "inline escapes",
            "a|b\n:---:|---\n`a\\|b`|**&amp;**",
            "`a",
            "|   a    | b         |\n| :----: | --------- |\n| `a\\|b` | **&amp;** |",
        ),
        (
            "short body",
            "A|B\n---|---\nvalue",
            "value",
            "| A     | B   |\n| ----- | --- |\n| value |     |",
        ),
        (
            "quote",
            "> a|b\n> ---|---\n> x|y\n",
            "x",
            "> | a   | b   |\n> | --- | --- |\n> | x   | y   |\n",
        ),
        (
            "list",
            "- a|b\n  ---|---\n  x|y\n",
            "x",
            "- | a   | b   |\n  | --- | --- |\n  | x   | y   |\n",
        ),
    ] {
        let (_fixture, mut session) = at("", source, needle);
        let point = session.view().point.position;
        action(&mut session, C::TableAlign);
        assert_eq!(
            text(session.buffer().unwrap()),
            expected,
            "{name}: {}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert_eq!(session.view().point.position, point, "{name}");
        session.execute(Command::History(History::Redo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), expected, "{name}");
        action(&mut session, C::TableAlign);
        assert_eq!(
            session.buffer().unwrap().history().len(),
            1,
            "idempotent {name}"
        );
    }
}

// Field and row navigation skip the mandatory delimiter. Existing aligned
// movement has no history; final navigation appends one body row in one undo.
#[test]
fn field_navigation() {
    for (name, needle, command, target, append) in [
        ("next header", "Name", C::TableNextField, "Value", false),
        ("skip delimiter", "Value", C::TableNextField, "界", false),
        (
            "previous header",
            "Name",
            C::TablePreviousField,
            "Name",
            false,
        ),
        ("previous row", "界", C::TablePreviousField, "Value", false),
        ("same column", "Value", C::TableNextRow, "7", false),
        ("append field", "7", C::TableNextField, "", true),
        ("append row", "界", C::TableNextRow, "", true),
    ] {
        let (_fixture, mut session) = at("", ALIGNED, needle);
        action(&mut session, command);
        let after = text(session.buffer().unwrap());
        assert_eq!(
            session.buffer().unwrap().history().len(),
            usize::from(append),
            "{name}: {}",
            session.echo()
        );
        let byte = Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_byte(session.view().point.position)
            .unwrap()
            .0;
        if append {
            assert!(after.starts_with(ALIGNED), "{name}: {after:?}");
            assert!(byte > ALIGNED.len(), "{name}");
            session.execute(Command::History(History::Undo));
            settle(&mut session);
            assert_eq!(text(session.buffer().unwrap()), ALIGNED, "{name}");
        } else {
            assert_eq!(after, ALIGNED, "{name}");
            assert_eq!(byte, ALIGNED.find(target).unwrap(), "{name}");
        }
    }
    let (_fixture, mut session) = at("", "| A |\n| --- |", "A");
    action(&mut session, C::TableNextRow);
    assert_eq!(text(session.buffer().unwrap()), "| A   |\n| --- |\n|     |");
}

// Explicit dimensions include a mandatory header/delimiter even with zero body
// rows. Creation replaces only the blank line and preserves its ending policy.
#[test]
fn explicit_creation() {
    for (source, draft, expected) in [
        ("", "2x0", "|     |     |\n| --- | --- |"),
        ("\r\n", "1x1", "|     |\r\n| --- |\r\n|     |\r\n"),
        ("  \n", "1 x 0", "  |     |\n  | --- |\n"),
    ] {
        let (_fixture, mut session) = at("", source, "");
        if !source.is_empty() {
            session.execute(Command::Motion(Motion::BufferStart));
        }
        action(&mut session, C::TableCreate);
        dimensions(&mut session, draft);
        assert_eq!(
            text(session.buffer().unwrap()),
            expected,
            "{draft}: {}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), source);
    }
}

// Lossy, literal, read-only and unavailable structural evidence refuse without
// source, point or history changes. A canceled worker cannot publish a plan.
#[test]
fn table_refusals() {
    for (name, config, source, needle, readonly) in [
        ("extra cells", "", "a|b\n---|---\nx|y|lost\n", "x", false),
        ("mismatch", "", "a|b\n---\nx|y\n", "x", false),
        ("literal", "", "```\na|b\n---|---\nx|y\n```\n", "x", false),
        ("readonly", "", ALIGNED, "界", true),
        ("cutoff", "syntax.max_bytes=5", ALIGNED, "界", false),
    ] {
        let (_fixture, mut session) = at_readonly(config, source, needle, readonly);
        let before = session.view().point;
        action(&mut session, C::TableAlign);
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert_eq!(session.view().point, before, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert!(!session.echo().is_empty(), "{name}");
    }
    let (_fixture, mut session) = at("", "a|b\n---|---\nx|y\n", "x");
    session.execute(Command::Markdown(C::TableAlign));
    session.execute(Command::KeyboardQuit);
    settle(&mut session);
    assert_eq!(text(session.buffer().unwrap()), "a|b\n---|---\nx|y\n");
    assert!(session.buffer().unwrap().history().is_empty());
}

// Both selectors resolve table names and native editing keys navigate only in
// admitted tables. Outside tables, Tab retains ordinary insertion authority.
#[test]
fn table_controls() {
    for profile in ["non-modal", "modal"] {
        let (_fixture, mut session) = at(
            &format!("editor.interaction_profile='{profile}'"),
            ALIGNED,
            "Name",
        );
        for name in [
            "markdown-table-align",
            "markdown-table-next-field",
            "markdown-table-previous-field",
            "markdown-table-next-row",
        ] {
            let event = if profile == "modal" {
                key(KeyCode::Char(':'), KeyModifiers::NONE)
            } else {
                key(KeyCode::Char('x'), KeyModifiers::ALT)
            };
            session.handle(event);
            session.handle(Event::Paste(name.into()));
            session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
            settle(&mut session);
            assert!(
                session.echo().is_empty(),
                "{profile}/{name}: {}",
                session.echo()
            );
        }
        if profile == "modal" {
            session.handle(key(KeyCode::Char('i'), KeyModifiers::NONE));
        }
        session.handle(key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        session.handle(key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        settle(&mut session);
        let before = session.view().point;
        session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
        settle(&mut session);
        assert_ne!(session.view().point, before, "{profile}");
        session.handle(key(KeyCode::BackTab, KeyModifiers::SHIFT));
        settle(&mut session);
        assert_eq!(session.view().point, before, "{profile}");
        session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), ALIGNED);
        assert!(session.buffer().unwrap().history().is_empty());
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
        settle(&mut session);
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{profile}");
    }
}

// Alignment maps retained Unicode cell offsets in both views and saves exact
// BOM/CRLF bytes. Undo restores original bytes and both canonical source points.
#[test]
fn table_fidelity() {
    let original = "\u{feff}a|b\r\n:---|---:\r\n界é|`x\\|y`\r\n";
    let (fixture, mut session) = at("markdown.pretty=false\nsyntax.enabled=false", original, "é");
    let point = session.view().point.position;
    session.execute(Command::Split(Split::Right));
    action(&mut session, C::TableAlign);
    let buffer = session.buffer().unwrap();
    let expected = Coordinates::new(buffer)
        .unwrap()
        .from_byte(Byte(text(buffer).find("é").unwrap()))
        .unwrap();
    assert!(
        session
            .windows()
            .iter()
            .all(|(_, view)| view.point.position == expected)
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
        format!("\u{feff}{}", text(session.buffer().unwrap()))
    );
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert!(
        session
            .windows()
            .iter()
            .all(|(_, view)| view.point.position == point)
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
        original
    );
}

// Invalid dimensions and canceled prompts retain a blank source. An intervening
// canonical edit invalidates the action's captured revision before publication.
#[test]
fn table_admission() {
    for draft in ["", "0x1", "257x0", "1x9999", "256x400", "2x-1", "2x1x3"] {
        let (_fixture, mut session) = at("", "", "");
        action(&mut session, C::TableCreate);
        dimensions(&mut session, draft);
        assert_eq!(text(session.buffer().unwrap()), "", "{draft}");
        assert!(session.buffer().unwrap().history().is_empty(), "{draft}");
    }
    let (_fixture, mut session) = at("", "", "");
    action(&mut session, C::TableCreate);
    session.handle(Event::Paste("2x1".into()));
    session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    settle(&mut session);
    assert_eq!(text(session.buffer().unwrap()), "");
    assert!(session.buffer().unwrap().history().is_empty());
    let (_fixture, mut session) = at("", "a|b\n---|---\nx|y\n", "x");
    session.execute(Command::Markdown(C::TableAlign));
    session.handle(Event::Paste("fresh".into()));
    settle(&mut session);
    assert_eq!(text(session.buffer().unwrap()), "a|b\n---|---\nfreshx|y\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Creation is available through both selectors and the advertised two-key
// chord. Factory field keys preserve parent remaps and modal state authority.
#[test]
fn profile_table_routes() {
    for profile in ["non-modal", "modal"] {
        for named in [true, false] {
            let (_fixture, mut session) =
                at(&format!("editor.interaction_profile='{profile}'"), "", "");
            create_route(&mut session, profile, named);
            dimensions(&mut session, "1x0");
            assert_eq!(
                text(session.buffer().unwrap()),
                "|     |\n| --- |",
                "{profile}/{named}: {}",
                session.echo()
            );
        }
    }
}

fn create_route(session: &mut Session, profile: &str, named: bool) {
    if named {
        let selector = if profile == "modal" {
            key(KeyCode::Char(':'), KeyModifiers::NONE)
        } else {
            key(KeyCode::Char('x'), KeyModifiers::ALT)
        };
        session.handle(selector);
        session.handle(Event::Paste("markdown-table-create".into()));
        session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    } else {
        if profile == "modal" {
            session.handle(key(KeyCode::Char('i'), KeyModifiers::NONE));
        }
        session.handle(key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        session.handle(key(KeyCode::Char('|'), KeyModifiers::NONE));
    }
}

// Contextual factory defaults cannot displace explicit parent Tab mappings.
// Explicit Markdown mappings then restore table navigation in both profiles.
#[test]
fn inherited_table_routes() {
    for (profile, parent) in [("non-modal", "editor"), ("modal", "vim_insert")] {
        for local in [false, true] {
            let override_map = if local {
                "keys.markdown.TAB='markdown-table-next-field'"
            } else {
                ""
            };
            let config = format!(
                "editor.interaction_profile='{profile}'\nkeys.{parent}.TAB='forward-char'\n{override_map}"
            );
            let (_fixture, mut session) = at(&config, ALIGNED, "Name");
            if profile == "modal" {
                session.handle(key(KeyCode::Char('i'), KeyModifiers::NONE));
            }
            session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
            settle(&mut session);
            let expected = if local {
                ALIGNED.find("Value").unwrap()
            } else {
                ALIGNED.find("Name").unwrap() + 1
            };
            let byte = Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .to_byte(session.view().point.position)
                .unwrap()
                .0;
            assert_eq!(byte, expected, "{profile}/{local}");
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

// Outside an admitted table, factory Tab keeps its inherited text behavior.
// A configured output cutoff refuses a valid creation without partial source.
#[test]
fn table_context_limits() {
    for profile in ["non-modal", "modal"] {
        let (_fixture, mut session) = at(
            &format!("editor.interaction_profile='{profile}'"),
            "plain",
            "plain",
        );
        if profile == "modal" {
            session.handle(key(KeyCode::Char('i'), KeyModifiers::NONE));
        }
        session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
        settle(&mut session);
        assert_ne!(text(session.buffer().unwrap()), "plain", "{profile}");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
    let (_fixture, mut session) = at("syntax.max_bytes=1", "", "");
    action(&mut session, C::TableCreate);
    dimensions(&mut session, "1x0");
    assert_eq!(text(session.buffer().unwrap()), "");
    assert!(session.buffer().unwrap().history().is_empty());
    let (_fixture, mut session) = at("editor.interaction_profile='modal'", ALIGNED, "Value");
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('v'), KeyModifiers::NONE));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(&mut session);
    assert_eq!(text(session.buffer().unwrap()), ALIGNED);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Creation cannot absorb adjacent paragraphs or tables into its body. The
// prospective parser check also protects EOF in an unclosed fenced block.
#[test]
fn creation_context() {
    let (_fixture, mut session) = at("", "before\n\nafter\n", "\nafter");
    action(&mut session, C::TableCreate);
    dimensions(&mut session, "1x0");
    assert_eq!(
        text(session.buffer().unwrap()),
        "before\n\n|     |\n| --- |\n\nafter\n"
    );
    action(&mut session, C::TableNextRow);
    assert_eq!(
        text(session.buffer().unwrap()),
        "before\n\n|     |\n| --- |\n|     |\n\nafter\n"
    );
    let (_fixture, mut session) = at("", "```text\n", "");
    action(&mut session, C::TableCreate);
    dimensions(&mut session, "1x0");
    assert_eq!(text(session.buffer().unwrap()), "```text\n");
    assert!(session.buffer().unwrap().history().is_empty());
}
