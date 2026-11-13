use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, History, MarkdownCommand as C, Motion, RegionCommand, Split};
use rev::coordinates::{Byte, Coordinates};

use super::lists::{at, at_readonly};
use super::tables::{action, key};
use super::*;

const SOURCE: &str = "| A   | B   |\n| --- | --- |\n| x   | y   |\n| z   | w   |";
const EMPTY: &str = "|     |     |\n";

// Row placement distinguishes above/current from contextual below. Mandatory
// rows, container prefixes and the original EOF newline policy remain intact.
#[test]
fn body_rows() {
    for (name, source, needle, command, expected) in [
        (
            "above",
            SOURCE,
            "x",
            C::TableInsertRow,
            SOURCE.replacen("| x", &format!("{EMPTY}| x"), 1),
        ),
        (
            "below",
            SOURCE,
            "x",
            C::MetaReturn,
            SOURCE.replacen("| z", &format!("{EMPTY}| z"), 1),
        ),
        (
            "first at header",
            SOURCE,
            "A",
            C::TableInsertRow,
            SOURCE.replacen("| x", &format!("{EMPTY}| x"), 1),
        ),
        (
            "delete middle",
            SOURCE,
            "x",
            C::TableDeleteRow,
            SOURCE.replace("| x   | y   |\n", ""),
        ),
        (
            "delete EOF",
            SOURCE,
            "z",
            C::TableDeleteRow,
            "| A   | B   |\n| --- | --- |\n| x   | y   |".into(),
        ),
        (
            "delete only body",
            "| A |\n| --- |\n| x |",
            "x",
            C::TableDeleteRow,
            "| A   |\n| --- |".into(),
        ),
        (
            "header only",
            "| A |\n| --- |",
            "A",
            C::MetaReturn,
            "| A   |\n| --- |\n|     |".into(),
        ),
        (
            "list header",
            "- a|b\n  ---|---\n  x|y\n",
            "a",
            C::TableInsertRow,
            "- | a   | b   |\n  | --- | --- |\n  |     |     |\n  | x   | y   |\n".into(),
        ),
        (
            "quote below",
            "> a|b\n> ---|---\n> x|y\n",
            "x",
            C::MetaReturn,
            "> | a   | b   |\n> | --- | --- |\n> | x   | y   |\n> |     |     |\n".into(),
        ),
    ] {
        let (_fixture, mut session) = at("", source, needle);
        let origin = session.view().point.position;
        action(&mut session, command);
        assert_eq!(
            text(session.buffer().unwrap()),
            expected,
            "{name}: {}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        let row = Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_line(session.view().point.position)
            .unwrap()
            .row;
        let expected_row = match name {
            "below" | "quote below" => 3,
            "delete only body" => 0,
            _ => 2,
        };
        assert_eq!(row, expected_row, "{name}: cursor row");
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert_eq!(session.view().point.position, origin, "{name}");
        session.execute(Command::History(History::Redo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), expected, "{name}");
    }
}

// Whole-column changes preserve surviving alignments and source. Short body
// rows contribute logical empty cells instead of shifting retained values.
#[test]
fn whole_columns() {
    for (name, source, needle, command, expected) in [
        (
            "before first",
            SOURCE,
            "A",
            C::TableInsertColumn,
            "|     | A   | B   |\n| --- | --- | --- |\n|     | x   | y   |\n|     | z   | w   |",
        ),
        (
            "before current",
            SOURCE,
            "y",
            C::TableInsertColumn,
            "| A   |     | B   |\n| --- | --- | --- |\n| x   |     | y   |\n| z   |     | w   |",
        ),
        (
            "delete first",
            SOURCE,
            "x",
            C::TableDeleteColumn,
            "| B   |\n| --- |\n| y   |\n| w   |",
        ),
        (
            "delete last",
            SOURCE,
            "w",
            C::TableDeleteColumn,
            "| A   |\n| --- |\n| x   |\n| z   |",
        ),
        (
            "keep alignments",
            "A|B|C\n:---|:---:|---:\nx|y|z\n",
            "B",
            C::TableDeleteColumn,
            "| A    |    C |\n| :--- | ---: |\n| x    |    z |\n",
        ),
        (
            "ragged insertion",
            "A|B|C\n---|:---:|---:\nx\n",
            "B",
            C::TableInsertColumn,
            "| A   |     |   B   |    C |\n| --- | --- | :---: | ---: |\n| x   |     |       |      |\n",
        ),
        (
            "ragged deletion",
            "A|B|C\n---|:---:|---:\nx\n",
            "B",
            C::TableDeleteColumn,
            "| A   |    C |\n| --- | ---: |\n| x   |      |\n",
        ),
        (
            "delimiter column",
            "A|B\n---|---:\nx|y",
            "---:",
            C::TableInsertColumn,
            "| A   |     |    B |\n| --- | --- | ---: |\n| x   |     |    y |",
        ),
    ] {
        let (_fixture, mut session) = at("", source, needle);
        let origin = session.view().point.position;
        action(&mut session, command);
        assert_eq!(
            text(session.buffer().unwrap()),
            expected,
            "{name}: {}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        let row = Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_line(session.view().point.position)
            .unwrap()
            .row;
        let expected_row = match name {
            "before current" | "delete first" => 2,
            "delete last" => 3,
            _ => 0,
        };
        assert_eq!(row, expected_row, "{name}: cursor row");
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert_eq!(session.view().point.position, origin, "{name}");
    }
}

// Every structural action rejects unavailable/lossy context and read-only
// authority. Protected rows and the last column refuse without point/history.
#[test]
fn structural_refusals() {
    for (name, source, needle, command, readonly) in [
        ("header", SOURCE, "A", C::TableDeleteRow, false),
        ("delimiter delete", SOURCE, "---", C::TableDeleteRow, false),
        ("delimiter insert", SOURCE, "---", C::TableInsertRow, false),
        ("delimiter below", SOURCE, "---", C::MetaReturn, false),
        (
            "last column",
            "A|\n---|\nx|",
            "x",
            C::TableDeleteColumn,
            false,
        ),
        (
            "lossy",
            "A|B\n---|---\nx|y|lost",
            "x",
            C::TableInsertColumn,
            false,
        ),
        (
            "literal",
            "```\nA|B\n---|---\nx|y\n```",
            "x",
            C::TableDeleteColumn,
            false,
        ),
        ("readonly row", SOURCE, "x", C::TableInsertRow, true),
        ("readonly column", SOURCE, "x", C::TableDeleteColumn, true),
    ] {
        let (_fixture, mut session) = at_readonly("", source, needle, readonly);
        let point = session.view().point;
        action(&mut session, command);
        assert_eq!(text(session.buffer().unwrap()), source, "{name}");
        assert_eq!(session.view().point, point, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert!(!session.echo().is_empty(), "{name}");
    }
}

// Retained Unicode/escaped/code source, another pane and a mark follow all four
// operations. One undo restores BOM+CRLF bytes and every pre-operation point.
#[test]
fn retained_structure() {
    let original = "\u{feff}a|b\r\n:---|---:\r\ngone|discard\r\nkeep|界é `x\\|y` &amp;";
    for command in [
        C::TableInsertRow,
        C::TableDeleteRow,
        C::TableInsertColumn,
        C::TableDeleteColumn,
    ] {
        let (fixture, mut session) =
            at("markdown.pretty=false\nsyntax.enabled=false", original, "é");
        session.execute(Command::Region(RegionCommand::Set));
        let mark = session.buffer().unwrap().mark();
        session.execute(Command::Split(Split::Right));
        let other = session
            .windows()
            .iter()
            .find(|(id, _)| *id != session.windows().selected_id())
            .unwrap()
            .0;
        move_to(&mut session, "gone");
        let points: Vec<_> = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.point.position))
            .collect();
        action(&mut session, command);
        let after = text(session.buffer().unwrap());
        assert!(
            after.contains("界é `x\\|y` &amp;"),
            "{command:?}: {}",
            session.echo()
        );
        let expected = Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .from_byte(Byte(after.find("é").unwrap()))
            .unwrap();
        assert_eq!(
            session.windows().get(other).unwrap().point.position,
            expected,
            "{command:?}"
        );
        assert_eq!(
            session.buffer().unwrap().mark(),
            Some(expected),
            "{command:?}"
        );
        assert!(!after.ends_with('\n'), "{command:?}");
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{command:?}");
        session.handle(Event::Resize(52, 12));
        settle(&mut session);
        session.execute(Command::SaveBuffer);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
            format!("\u{feff}{after}")
        );
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        for (id, point) in points {
            assert_eq!(
                session.windows().get(id).unwrap().point.position,
                point,
                "{command:?}"
            );
        }
        session.execute(Command::SaveBuffer);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
            original
        );
    }
}

fn move_to(session: &mut Session, needle: &str) {
    let source = text(session.buffer().unwrap());
    let target = Coordinates::new(session.buffer().unwrap())
        .unwrap()
        .from_byte(Byte(source.find(needle).unwrap()))
        .unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    while session.view().point.position < target {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    assert_eq!(session.view().point.position, target);
}

// A queued operation loses publication authority after cancellation or a
// canonical edit, even if its worker has finished before the next poll.
#[test]
fn structural_retirement() {
    for (command, canceled) in [
        (C::TableInsertRow, true),
        (C::TableDeleteRow, false),
        (C::TableInsertColumn, true),
        (C::TableDeleteColumn, false),
    ] {
        let (_fixture, mut session) = at("", SOURCE, "x");
        let point = session.view().point;
        session.execute(Command::Markdown(command));
        let expected = if canceled {
            session.execute(Command::KeyboardQuit);
            SOURCE.to_owned()
        } else {
            session.handle(Event::Paste("fresh".into()));
            SOURCE.replacen("x", "freshx", 1)
        };
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), expected, "{command:?}");
        assert_eq!(
            session.buffer().unwrap().history().len(),
            usize::from(!canceled)
        );
        if canceled {
            assert_eq!(session.view().point, point);
        }
    }
}

// Both native selectors and editing profiles reach all structural commands.
// Contextual Meta-Return inserts below through the same table action owner.
#[test]
fn structural_routes() {
    for profile in ["non-modal", "modal"] {
        for (code, command) in [
            (KeyCode::Down, C::TableInsertRow),
            (KeyCode::Up, C::TableDeleteRow),
            (KeyCode::Right, C::TableInsertColumn),
            (KeyCode::Left, C::TableDeleteColumn),
            (KeyCode::Enter, C::MetaReturn),
        ] {
            let config = format!("editor.interaction_profile='{profile}'");
            let (_fixture, mut named) = at(&config, SOURCE, "x");
            named.handle(if profile == "modal" {
                key(KeyCode::Char(':'), KeyModifiers::NONE)
            } else {
                key(KeyCode::Char('x'), KeyModifiers::ALT)
            });
            named.handle(Event::Paste(command.name().into()));
            named.handle(key(KeyCode::Enter, KeyModifiers::NONE));
            settle(&mut named);
            assert_eq!(
                named.buffer().unwrap().history().len(),
                1,
                "{profile}/{command:?}: {}",
                named.echo()
            );
            let (_fixture, mut keyed) = at(&config, SOURCE, "x");
            insert_state(&mut keyed, profile);
            let modifiers = if command == C::MetaReturn {
                KeyModifiers::ALT
            } else {
                KeyModifiers::ALT | KeyModifiers::SHIFT
            };
            keyed.handle(key(code, modifiers));
            settle(&mut keyed);
            assert_eq!(
                text(keyed.buffer().unwrap()),
                text(named.buffer().unwrap()),
                "{profile}/{command:?}"
            );
            assert_eq!(keyed.view().point.position, named.view().point.position);
        }
    }
}

fn insert_state(session: &mut Session, profile: &str) {
    if profile == "modal" {
        session.handle(key(KeyCode::Char('i'), KeyModifiers::NONE));
    }
}

// Explicit parent routes precede factory structure keys; explicit local routes
// win last. Normal/Visual/Replace states cannot acquire editing-only defaults.
#[test]
fn structural_precedence() {
    for (profile, parent) in [("non-modal", "editor"), ("modal", "vim_insert")] {
        for local in [false, true] {
            let suffix = if local {
                "keys.markdown.'M-S-Right'='markdown-table-insert-column'"
            } else {
                ""
            };
            let config = format!(
                "editor.interaction_profile='{profile}'\nkeys.{parent}.'M-S-Right'='forward-char'\n{suffix}"
            );
            let (_fixture, mut session) = at(&config, SOURCE, "x");
            insert_state(&mut session, profile);
            session.handle(key(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT));
            settle(&mut session);
            assert_eq!(
                session.buffer().unwrap().history().len(),
                usize::from(local),
                "{profile}/{local}"
            );
            if !local {
                assert_eq!(text(session.buffer().unwrap()), SOURCE);
                assert_eq!(
                    session.view().point.position.0,
                    SOURCE.find('x').unwrap() + 1
                );
            }
        }
    }
    for state in ['\0', 'v', 'R'] {
        let (_fixture, mut session) = at("editor.interaction_profile='modal'", SOURCE, "x");
        if state != '\0' {
            session.handle(key(KeyCode::Char(state), KeyModifiers::NONE));
        }
        session.handle(key(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), SOURCE, "{state:?}");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
