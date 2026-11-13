use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, History, MarkdownCommand, Motion, Split};
use rev::coordinates::{Byte, Coordinates};

use super::*;

pub(super) fn at(config: &str, source: &str, needle: &str) -> (Fixture, Session) {
    at_readonly(config, source, needle, false)
}

pub(super) fn at_readonly(
    config: &str,
    source: &str,
    needle: &str,
    readonly: bool,
) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&fixture.write("notes.md", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(readonly);
    let buffer = buffers.get(id).unwrap();
    let canonical = text(buffer);
    let mut view = View::with_settings(buffer, &startup.snapshot);
    view.point.position = Coordinates::new(buffer)
        .unwrap()
        .from_byte(Byte(if needle.is_empty() {
            canonical.len()
        } else {
            canonical.find(needle).unwrap()
        }))
        .unwrap();
    let mut session = Session::new(buffers, view, SIZE).unwrap();
    settle(&mut session);
    (fixture, session)
}

// Depth comes from admitted container ownership, including mixed ordered lists,
// repeated indentation, quotes, same-line containers and protected literal code.
#[test]
fn marker_depths() {
    let source = "- Parent\n    + Child\n        * Leaf\n            - Deep\n\n3. Ordered\n   - Under ordered\n\n> - Quote\n>   + Nested\n\n- - Inline\n\n```\n- Literal\n```\n";
    for pretty in [true, false] {
        let (_fixture, mut session) =
            editor(&format!("markdown.pretty={pretty}"), "notes.md", source);
        settle(&mut session);
        let frame = session.frame().unwrap();
        for (row, expected) in [
            (0, "• Parent"),
            (1, "    ◦ Child"),
            (2, "        ▪ Leaf"),
            (3, "            • Deep"),
            (6, "   ◦ Under ordered"),
            (8, "> • Quote"),
            (9, ">   ◦ Nested"),
            (11, "• ◦ Inline"),
        ] {
            if pretty {
                assert!(
                    frame.rows()[row]
                        .text()
                        .chars()
                        .skip(7)
                        .collect::<String>()
                        .starts_with(expected),
                    "{row}: {:?}",
                    frame.rows()[row].text()
                );
            }
        }
        assert!(frame.rows()[5].text().contains("3. Ordered"));
        assert!(frame.rows()[14].text().contains("- Literal"));
        assert_eq!(text(session.buffer().unwrap()), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Continue inserts a source-style sibling after the complete item, or exits
// one empty level. Newline identity, ordered rollover and undo remain exact.
#[test]
fn continue_and_exit() {
    for (name, source, needle, expected) in [
        ("bullet", "- one", "one", "- one\n- "),
        ("newline", "+ one\n", "one", "+ one\n+ \n"),
        ("ordered", "9) one\n", "one", "9) one\n10) \n"),
        ("zero padded", "009. one", "one", "009. one\n010. "),
        ("root empty", "- \n", "-", "\n"),
        (
            "nested empty",
            "- Parent\n    + Child\n    + ",
            "",
            "- Parent\n    + Child\n+ ",
        ),
        ("inline parent", "- - ", "- ", "- - \n- "),
        ("inline empty", "- + ", "", "- \n+ "),
        (
            "subtree",
            "- Parent\n  + Child\n    continuation\n- Next\n",
            "Parent",
            "- Parent\n  + Child\n    continuation\n- \n- Next\n",
        ),
        ("quote", "> - one\n", "one", "> - one\n> - \n"),
        (
            "quote after list",
            "- > - one\n",
            "one",
            "- > - one\n  > - \n",
        ),
        ("CRLF", "- 界e\u{301}\r\n", "界", "- 界e\u{301}\r\n- \r\n"),
    ] {
        let (_fixture, mut session) = at("", source, needle);
        let state = session.buffer().unwrap().state();
        let point = session.view().point.position;
        session.execute(Command::Markdown(MarkdownCommand::ContinueItem));
        settle(&mut session);
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
        assert_eq!(session.buffer().unwrap().state(), state);
        assert_eq!(session.view().point.position, point);
        session.execute(Command::History(History::Redo));
        settle(&mut session);
        assert_eq!(text(session.buffer().unwrap()), expected, "{name}");
    }
}

// Indent/outdent retain children and continuation paragraphs. Outdent moves
// after the parent's remaining subtree instead of adopting later siblings.
#[test]
fn move_subtrees() {
    for (name, source, needle, command, expected) in [
        (
            "indent",
            "- first\n- second\n  continuation\n  + child\n",
            "second",
            MarkdownCommand::IndentItem,
            "- first\n  - second\n    continuation\n    + child\n",
        ),
        (
            "ordered parent",
            "10. first\n11. second\n",
            "second",
            MarkdownCommand::IndentItem,
            "10. first\n    11. second\n",
        ),
        (
            "outdent",
            "- Parent\n  + selected\n    child text\n  + remains\n- Next\n",
            "selected",
            MarkdownCommand::OutdentItem,
            "- Parent\n  + remains\n+ selected\n  child text\n- Next\n",
        ),
        (
            "EOF transfer",
            "- Parent\n  + selected\n  + remains",
            "selected",
            MarkdownCommand::OutdentItem,
            "- Parent\n  + remains\n+ selected",
        ),
        (
            "inline",
            "- - selected\n",
            "selected",
            MarkdownCommand::OutdentItem,
            "- \n- selected\n",
        ),
        (
            "quote",
            "> - Parent\n>   + selected\n",
            "selected",
            MarkdownCommand::OutdentItem,
            "> - Parent\n> + selected\n",
        ),
        (
            "CRLF",
            "- Parent\r\n  + 界e\u{301}\r\n",
            "界",
            MarkdownCommand::OutdentItem,
            "- Parent\r\n+ 界e\u{301}\r\n",
        ),
        (
            "quote after list",
            "- > - first\n  > - second\n",
            "second",
            MarkdownCommand::IndentItem,
            "- > - first\n  >   - second\n",
        ),
        (
            "outdent quote container",
            "- Parent\n  > - selected\n  >   text\n- Next\n",
            "selected",
            MarkdownCommand::OutdentItem,
            "- Parent\n> - selected\n>   text\n- Next\n",
        ),
        (
            "inline quote container",
            "- > - selected\n",
            "selected",
            MarkdownCommand::OutdentItem,
            "- > \n> - selected\n",
        ),
        (
            "ordered quote parent",
            "10. > - first\n    >   + selected\n",
            "selected",
            MarkdownCommand::OutdentItem,
            "10. > - first\n    > + selected\n",
        ),
    ] {
        let (_fixture, mut session) = at("", source, needle);
        let point = session.view().point.position;
        session.execute(Command::Markdown(command));
        settle(&mut session);
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
    }
}

// Real profile routes preserve Normal/Visual ownership and inherit explicit
// editor/Insert remaps before factory context defaults. Plain text gets newline.
#[test]
fn editing_routes() {
    for profile in ["non-modal", "modal"] {
        let config = format!("editor.interaction_profile='{profile}'");
        let (_fixture, mut session) = at(&config, "- one", "one");
        if profile == "modal" {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('i'),
                KeyModifiers::NONE,
            )));
        }
        session.execute(Command::Motion(Motion::LineEnd));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        settle(&mut session);
        assert_eq!(
            text(session.buffer().unwrap()),
            "- one\n- ",
            "{profile}: {}",
            session.echo()
        );
        let (_fixture, mut session) = at(&config, "plain", "plain");
        if profile == "modal" {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('i'),
                KeyModifiers::NONE,
            )));
        }
        session.execute(Command::Motion(Motion::LineEnd));
        session.handle(Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)));
        assert_eq!(text(session.buffer().unwrap()), "plain\n", "{profile}");
    }
}

// Retained source maps keep another pane on its Unicode body through a move;
// neither rewritten indentation nor a changed byte width collapses that point.
#[test]
fn shared_points() {
    let source = "- Parent\n  + 界e\u{301} selected\n  + remains\n";
    let (_fixture, mut session) = at("", source, "selected");
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Markdown(MarkdownCommand::OutdentItem));
    settle(&mut session);
    let expected = "- Parent\n  + remains\n+ 界e\u{301} selected\n";
    assert_eq!(text(session.buffer().unwrap()), expected);
    let expected = Coordinates::new(session.buffer().unwrap())
        .unwrap()
        .from_byte(Byte(expected.find("selected").unwrap()))
        .unwrap();
    assert!(
        session
            .windows()
            .iter()
            .all(|(_, view)| view.point.position == expected)
    );
}

// Pretty/source views and disabled colors share canonical list edits. Saving
// a Unicode continuation retains BOM+CRLF; one undo/save restores exact bytes.
#[test]
fn bom_roundtrip() {
    let source = "\u{feff}- 界e\u{301}\r\n";
    for pretty in [true, false] {
        let (fixture, mut session) = at(
            &format!("markdown.pretty={pretty}\nsyntax.enabled=false"),
            source,
            "界",
        );
        let frame = session.frame().unwrap();
        assert_eq!(
            surface::draw(&frame)[&(0, 7)].character,
            if pretty { '•' } else { '-' }
        );
        assert!(frame.rows()[0].syntax().is_empty());
        session.execute(Command::Markdown(MarkdownCommand::ContinueItem));
        settle(&mut session);
        session.execute(Command::SaveBuffer);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
            "\u{feff}- 界e\u{301}\r\n- \r\n"
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        session.execute(Command::SaveBuffer);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
            source
        );
        assert!(session.buffer().unwrap().has_bom());
    }
}
