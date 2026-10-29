#![forbid(unsafe_code)]
#[path = "org_tables/edges.rs"]
mod edges;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "org_tables/lifecycle.rs"]
mod lifecycle;
#[path = "org_tables/metadata.rs"]
mod metadata;
#[path = "org_tables/references.rs"]
mod references;
#[path = "org_tables/structure.rs"]
mod structure;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fixture::Fixture;
use rev::{
    buffer::Buffers,
    command::{Command, History, Motion, OrgCommand as C},
    config::prepare,
    session::Session,
    view::{Size, View},
};
use std::time::{Duration, Instant};
fn editor(config: &str, source: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let startup = prepare(&f.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("notes.org", source)).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap();
    (f, s)
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn settle(s: &mut Session) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        s.frame().unwrap();
        if s.remaining_input(Instant::now()).is_none() {
            break;
        }
        assert!(Instant::now() < end, "{}", s.echo());
        s.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn action(s: &mut Session, command: C) {
    s.execute(Command::Org(command));
    settle(s);
}
fn argument(s: &mut Session, text: &str) {
    s.handle(Event::Paste(text.into()));
    s.handle(key(KeyCode::Enter));
    settle(s);
}
fn text(s: &Session) -> String {
    let b = s.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn at(s: &mut Session, scalar: usize) {
    s.execute(Command::Motion(Motion::BufferStart));
    while s.view().point.position.0 < scalar {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
    assert_eq!(s.view().point.position.0, scalar);
}
// Alignment preserves every source cell while changing only table padding/rules.
// A single native transaction and its initiating point survive undo/redo.
#[test]
fn align_source() {
    let source = "|Name|Count|\n|--+--|\n|A|2|\n|Long|11|\n";
    let expected = "| Name | Count |\n|------+-------|\n| A    |     2 |\n| Long |    11 |\n";
    let (_f, mut s) = editor("", source);
    let before = s.view().point.position;
    action(&mut s, C::TableAlign);
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert_eq!(s.view().point.position, before);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    action(&mut s, C::TableAlign);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    assert_eq!(s.view().point.position, before);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(&s), expected);
}
// TAB/S-TAB field motion skips rules and does not create history for an already
// aligned table; RET creates a row before a rule in one ordinary undo unit.
#[test]
fn cell_navigation() {
    let source = "| A | B |\n|---+---|\n| C | D |";
    let (_f, mut s) = editor("", source);
    at(&mut s, 2);
    action(&mut s, C::TableNextField);
    assert_eq!(s.view().point.position.0, 6, "{}", s.echo());
    action(&mut s, C::Cycle);
    assert_eq!(s.view().point.position.0, 22, "{}", s.echo());
    action(&mut s, C::GlobalCycle);
    assert_eq!(s.view().point.position.0, 6, "{}", s.echo());
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    at(&mut s, 2);
    action(&mut s, C::TableNextRow);
    assert_eq!(
        text(&s),
        "| A | B |\n|   |   |\n|---+---|\n| C | D |",
        "{}",
        s.echo()
    );
    assert_eq!(s.view().point.position.0, 12);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
}
// Native row insertion, hlines and end-of-table field motion retain original
// rows and EOF spelling. Each structural insertion undoes in one operation.
#[test]
fn row_insertion() {
    let source = "| A | B |\n| C | D |";
    for (name, command, point, expected) in [
        (
            "above",
            C::TableInsertRow,
            6,
            "|   |   |\n| A | B |\n| C | D |",
        ),
        ("below", C::MetaReturn, 6, "| A | B |\n|   |   |\n| C | D |"),
        (
            "rule",
            C::TableInsertHline,
            6,
            "| A | B |\n|---+---|\n| C | D |",
        ),
        (
            "append",
            C::TableNextField,
            16,
            "| A | B |\n| C | D |\n|   |   |",
        ),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, point);
        action(&mut s, command);
        assert_eq!(text(&s), expected, "{name}: {}", s.echo());
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source, "{name}");
    }
}
// The create prompt publishes a real empty table; cancellation and invalid
// dimensions preserve the source and provide no partial row/history publication.
#[test]
fn create_table() {
    let (_f, mut s) = editor("", "");
    action(&mut s, C::TableCreate);
    argument(&mut s, "2x2");
    assert_eq!(text(&s), "|   |   |\n|---+---|\n|   |   |", "{}", s.echo());
    assert_eq!(s.view().point.position.0, 2);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    action(&mut s, C::TableAlign);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "");
    action(&mut s, C::TableCreate);
    s.execute(Command::KeyboardQuit);
    assert_eq!(text(&s), "");
    action(&mut s, C::TableCreate);
    argument(&mut s, "0x2");
    assert_eq!(text(&s), "");
    assert!(s.buffer().unwrap().history().is_empty());
}
