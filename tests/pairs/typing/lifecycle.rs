use super::*;
use rev::buffer::Mode;
use rev::command::WindowCommand;
use rev::session::Session;

// The named unbound command toggles one canonical buffer. Shared panes and
// language changes keep its override without source or history publication.
#[test]
fn shared_toggle() {
    let mut e = Editor::new("", "plain", "|");
    let stamp = e.session.buffer().unwrap().stamp();
    e.session.execute(Command::Split(Split::Right));
    named(&mut e.session, "auto-pair-mode");
    assert!(!e.session.buffer().unwrap().pair_policy().enabled);
    e.session.execute(Command::Window(WindowCommand::Other));
    assert!(!e.session.buffer().unwrap().pair_policy().enabled);
    e.session.execute(Command::Mode(Mode::Text));
    assert!(!e.session.buffer().unwrap().pair_policy().enabled);
    assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
    assert!(e.session.buffer().unwrap().history().is_empty());
    e.type_key('(');
    e.expect("(|");
    named(&mut e.session, "auto-pair-mode");
    assert!(e.session.buffer().unwrap().pair_policy().enabled);
    e.type_key('[');
    e.expect("([|]");
}

fn named(session: &mut Session, name: &str) {
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(session);
}

// The frozen inventory plus three editing and seven navigation additions
// stays exact. Additions are literal identities, not copied from live output.
#[test]
fn inventory() {
    use std::collections::BTreeSet;
    let frozen: serde_json::Value = serde_json::from_str(include_str!(
        "../../../ops/default/report/rev-factory-shortcuts/catalog.json"
    ))
    .unwrap();
    let baseline: BTreeSet<_> = frozen["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(baseline.len(), 707);
    let live: BTreeSet<_> = Command::ALL.iter().map(Command::name).collect();
    let added = BTreeSet::from([
        "auto-pair-mode",
        "pair-wrap",
        "pair-unwrap",
        "matching-delimiter",
        "forward-pair",
        "backward-pair",
        "up-pair",
        "down-pair",
        "forward-sexp",
        "backward-sexp",
        "pair-highlight-mode",
        "pair-highlight-style",
    ]);
    assert!(baseline.is_disjoint(&added));
    assert_eq!(baseline, live.difference(&added).copied().collect());
    assert_eq!(live.len(), 719);
    assert!(added.is_subset(&live));
    assert_eq!(
        Command::named("auto-pair-mode"),
        Some(Command::AutoPairMode)
    );
    assert_eq!(
        Command::AutoPairMode.description(),
        Some(
            "Toggle automatic delimiter editing for this buffer and its views without writing settings."
        )
    );
    for context in rev::input::keymap::Context::ALL {
        assert!(
            !context
                .defaults()
                .iter()
                .any(|(_, name)| *name == "auto-pair-mode")
        );
    }
}

// Typed pairing never borrows prompt/search focus. Read-only source rejects
// insertion and overtyping before it can move point or create an undo group.
#[test]
fn focus_and_readonly() {
    for command in [
        Command::FindFile,
        Command::SearchForward,
        Command::ExecuteExtended,
    ] {
        let mut e = Editor::new("", "plain", "|text");
        e.session.execute(command);
        e.session.handle(character('('));
        assert_eq!(e.text(), "text");
        assert!(e.session.buffer().unwrap().history().is_empty());
        e.session.execute(Command::AutoPairMode);
        assert!(e.session.buffer().unwrap().pair_policy().enabled);
    }
    for (source, c) in [("|", '('), ("(|)", ')'), ("|", '"')] {
        let mut e = Editor::build("", "plain", source, true);
        let point = e.session.view().point;
        e.type_key(c);
        e.expect(source);
        assert_eq!(e.session.view().point, point);
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Pending pair work uses the existing native FIFO. Cancellation discards the
// uncommitted edit; source publication invalidates old proof before queued input.
#[test]
fn cancellation_and_revision() {
    let source = format!("|{}", "x".repeat(2048));
    let mut e = Editor::new("", "plain", &source);
    let stamp = e.session.buffer().unwrap().stamp();
    e.session.handle(character('('));
    assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
    e.session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    settle(&mut e.session);
    e.expect(&source);
    assert!(e.session.buffer().unwrap().history().is_empty());
    e.session.handle(character('['));
    e.session.replace(0..0, "changed").unwrap();
    settle(&mut e.session);
    assert_eq!(e.text(), format!("changed{}", "x".repeat(2048)));
    assert_eq!(e.session.buffer().unwrap().history().len(), 1);
}

// Later physical input waits until pair publication. Its resulting cursor is
// used once, so the queued closer skips the generated mate instead of duplicating it.
#[test]
fn queued_typing() {
    let mut e = Editor::new("", "plain", &format!("|{}", "x".repeat(1024)));
    e.session.handle(character('('));
    e.session.handle(character('a'));
    e.session.handle(character(')'));
    settle(&mut e.session);
    e.expect(&format!("(a)|{}", "x".repeat(1024)));
}

// A cold limited ordinary request remains literal. A counted request cannot
// publish its prefix or any generated mate when the shared scan budget exhausts.
#[test]
fn scan_bounds() {
    let source = format!("|{}", "x".repeat(rev::pairs::MAX_BYTES + 1));
    let mut ordinary = Editor::new("", "plain", &source);
    ordinary.type_key('(');
    ordinary.expect(&format!("(|{}", "x".repeat(rev::pairs::MAX_BYTES + 1)));
    let mut counted = Editor::new("", "plain", &source);
    let point = counted.session.view().point;
    let stamp = counted.session.buffer().unwrap().stamp();
    counted.counted("3", '(');
    counted.expect(&source);
    assert_eq!(counted.session.view().point, point);
    assert_eq!(counted.session.buffer().unwrap().stamp(), stamp);
    assert!(counted.session.buffer().unwrap().history().is_empty());
}

// A successful skip still retains the original quote fact. A later context
// query inside that source cannot reinterpret string contents as executable code.
#[test]
fn retained_context() {
    use rev::pairs::{Context, Kind, Outcome, Query, Status, Typed};
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("\"abc\"").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let ticket = buffer
        .start_pair_request(Query::Typing(Typed {
            point: Scalar(4),
            end: Scalar(4),
            character: '"',
            count: 1,
            skip: true,
        }))
        .unwrap();
    complete(buffer, ticket);
    let ticket = buffer
        .start_pair_request(Query::Context(Scalar(2)))
        .unwrap();
    complete(buffer, ticket);
    assert_eq!(
        buffer.pair_answer(ticket).unwrap().outcome,
        Outcome::Context(Context::String(Kind::DoubleQuote))
    );

    fn complete(buffer: &mut rev::buffer::Buffer, ticket: rev::pairs::Ticket) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            assert!(Instant::now() < deadline);
            match buffer.poll_pair_request(ticket, Instant::now() + rev::pairs::SLICE_TIME) {
                Status::Ready => break,
                Status::Pending => (),
                Status::Retired => panic!("live context ticket retired"),
            }
        }
    }
}
