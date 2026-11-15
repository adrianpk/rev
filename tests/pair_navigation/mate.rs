use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::buffer::{Buffer, Buffers};
use rev::command::{Command, ModalCommand, Motion, RegionCommand, Split, TreeCommand};
use rev::coordinates::{Coordinates, Scalar};
use rev::pairs::{Answer, Limit, Outcome, SLICE_TIME, Status, Ticket};
use serde::Deserialize;

use crate::editor::{Editor, character, key, settle};

#[derive(Deserialize)]
struct Contract {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    case: String,
    profile: String,
    initial: String,
    point_before: usize,
    target: usize,
    result: String,
    #[serde(default)]
    read_only: bool,
    pairs_enabled: Option<bool>,
    pairs_quotes: Option<bool>,
}

fn named(e: &mut Editor) {
    e.session.execute(Command::ExecuteExtended);
    e.session.handle(Event::Paste("matching-delimiter".into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(&mut e.session);
}

fn marked(source: &str, point: usize) -> String {
    let mut marked = source.to_owned();
    let byte = source
        .char_indices()
        .nth(point)
        .map_or(source.len(), |(at, _)| at);
    marked.insert(byte, '|');
    marked
}

// The approved independent native cases, including refusals, run through real
// named dispatch. All outcomes preserve the canonical source stamp and history.
#[test]
fn contract() {
    let contract: Contract = serde_json::from_str(include_str!(
        "../../ops/default/report/rev-delimiter-navigation/mate-contract.json"
    ))
    .unwrap();
    assert_eq!(contract.cases.len(), 19);
    for case in contract.cases {
        let config = format!(
            "pairs.enabled={}\npairs.quotes={}",
            case.pairs_enabled.unwrap_or(true),
            case.pairs_quotes.unwrap_or(true)
        );
        let mut e = Editor::build(
            &config,
            &case.profile,
            &marked(&case.initial, case.point_before),
            case.read_only,
        );
        let stamp = e.session.buffer().unwrap().stamp();
        if case.result == "unsupported-prefix" {
            e.session
                .handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
            e.session.handle(character('2'));
        }
        named(&mut e);
        assert_eq!(
            e.session.view().point.position,
            Scalar(case.target),
            "{}",
            case.case
        );
        assert_eq!(e.text(), case.initial, "{}", case.case);
        assert_eq!(e.session.buffer().unwrap().stamp(), stamp, "{}", case.case);
        assert!(
            e.session.buffer().unwrap().history().is_empty(),
            "{}",
            case.case
        );
    }
}

// Endpoint round trips include strict structural kinds and admitted language
// quotes. Escapes, raw strings, comments and crossed kinds cannot invent a mate.
#[test]
fn endpoints() {
    for (name, language, source, start, end) in [
        ("parenthesis", "plain", "(a\n b)", 0, 5),
        ("bracket", "plain", "[界e\u{301}]", 0, 4),
        ("brace", "plain", "{\r\n}", 0, 3),
        ("escaped quote", "c", "\"a\\\"b\"", 0, 5),
        ("single quote", "toml", "'alpha'", 0, 6),
        ("raw Go quote", "go", "`a\nb`", 0, 4),
        ("adjacent closer", "plain", "()[]", 1, 0),
    ] {
        let mut e = Editor::new("", language, &marked(source, start));
        named(&mut e);
        e.expect(&marked(source, end));
        named(&mut e);
        e.expect(&marked(source, start));
        assert!(e.session.buffer().unwrap().history().is_empty(), "{name}");
    }
    for (name, language, source, point) in [
        ("crossed", "c", "([)]", 0),
        ("unclosed quote", "c", "\"abc", 0),
        ("escaped interior quote", "c", "\"a\\\"b\"", 3),
        ("string punctuation", "c", "\"(a)\"", 1),
        ("comment", "rust", "// (a)\n", 3),
        ("Rust raw", "rust", "r#\"(a)\"#", 3),
        ("template", "javascript", "`(a)`", 1),
        ("lifetime", "rust", "'life (a)", 0),
        ("Markdown literal", "markdown", "`(a)`", 1),
        ("Org literal", "org", "=(a)=", 1),
        ("at point wins over previous", "c", "()[a)", 2),
    ] {
        let mut e = Editor::new("", language, &marked(source, point));
        named(&mut e);
        e.expect(&marked(source, point));
        assert!(e.session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// A native linear mark, including an empty active region, is independent of a
// source jump. The modal source-jump owner records traversal without an edit.
#[test]
fn mark_and_jumps() {
    for profile in ["non-modal", "modal"] {
        for empty in [false, true] {
            let config = format!("editor.interaction_profile='{profile}'");
            let mut e = Editor::new(&config, "plain", "|(ab)");
            if !empty {
                e.session.execute(Command::Motion(Motion::ForwardChar));
            }
            e.session.execute(Command::Region(RegionCommand::Set));
            if !empty {
                e.session.execute(Command::Motion(Motion::BackwardChar));
            }
            let mark = e.session.buffer().unwrap().mark();
            named(&mut e);
            e.expect("(ab|)");
            assert_eq!(e.session.buffer().unwrap().mark(), mark);
            assert!(e.session.buffer().unwrap().mark_active());
            named(&mut e);
            e.expect("|(ab)");
            assert_eq!(e.session.buffer().unwrap().mark(), mark);
            assert!(e.session.buffer().unwrap().mark_active());
            assert!(e.session.buffer().unwrap().history().is_empty());
        }
    }
    let mut e = Editor::new("editor.interaction_profile='modal'", "plain", "|(ab)");
    named(&mut e);
    e.session
        .execute(Command::Navigate(rev::command::Navigation::new(
            rev::command::navigation::Target::Jump(rev::search::Direction::Backward),
        )));
    settle(&mut e.session);
    e.expect("|(ab)");
    e.session
        .execute(Command::Navigate(rev::command::Navigation::new(
            rev::command::navigation::Target::Jump(rev::search::Direction::Forward),
        )));
    settle(&mut e.session);
    e.expect("(ab|)");
}

// Explicit integer/zero/negative/universal prefixes are unsupported, whether
// supplied through M-x or a custom binding. Later commands see no retained count.
#[test]
fn prefixes() {
    for count in ["0", "1", "2", "-1"] {
        let mut e = Editor::new("[keys.editor]\nj='matching-delimiter'", "plain", "|(ab)");
        e.counted(count, 'j');
        e.expect("|(ab)");
        assert!(echo(&mut e).contains("does not support a prefix"));
        named(&mut e);
        e.expect("(ab|)");
    }
    let mut e = Editor::new("", "plain", "|(ab)");
    e.session
        .handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    named(&mut e);
    e.expect("|(ab)");
    assert!(echo(&mut e).contains("does not support a prefix"));
}

fn echo(e: &mut Editor) -> String {
    e.session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|row| row.text())
        .collect::<Vec<_>>()
        .join("\n")
}

// The unbound catalog entry has a real description. An explicit user binding
// supplies the effective hint, and prompts/tree focus cannot move hidden source.
#[test]
fn catalog_and_focus() {
    let command = Command::named("matching-delimiter").unwrap();
    assert_eq!(Command::ALL.len(), 717);
    assert!(
        command
            .description()
            .unwrap()
            .contains("confirmed typed mate")
    );
    let mut e = Editor::new("", "plain", "|(ab)");
    for context in rev::input::keymap::Context::ALL {
        assert!(
            context
                .defaults()
                .iter()
                .all(|(_, name)| *name != "matching-delimiter")
        );
    }
    e.session.execute(Command::FindFile);
    e.session.execute(command);
    e.expect("|(ab)");
    e.session.execute(Command::KeyboardQuit);
    e.session.execute(Command::Tree(TreeCommand::Focus));
    e.session.execute(command);
    e.expect("|(ab)");
    let mut e = Editor::new(
        "[keys.editor]\n'M-j'='matching-delimiter'",
        "plain",
        "|(ab)",
    );
    e.session.execute(Command::ExecuteExtended);
    e.session.handle(Event::Paste("matching-delimiter".into()));
    assert!(echo(&mut e).contains("M-j"));
    e.session.execute(Command::KeyboardQuit);
    e.session.handle(key(KeyCode::Char('j'), KeyModifiers::ALT));
    settle(&mut e.session);
    e.expect("(ab|)");
    for state in [ModalCommand::Insert, ModalCommand::Replace] {
        let mut e = Editor::build("editor.interaction_profile='modal'", "c", "|\"ab\"", true);
        e.session.execute(Command::Modal(state));
        named(&mut e);
        e.expect("\"ab|\"");
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// C-g retires only the owned request. Source and selected-view changes
// invalidate pending publication; queued input runs after a jump.
#[test]
fn lifecycle() {
    let source = format!("({})", "a ".repeat(4096));
    for change in ["cancel", "source", "view"] {
        let mut e = Editor::new("", "plain", &marked(&source, 0));
        if change == "view" {
            e.session.execute(Command::Split(Split::Right));
        }
        e.session.execute(Command::MatchingDelimiter);
        assert_eq!(e.session.view().point.position, Scalar(0), "{change}");
        match change {
            "cancel" => {
                e.session
                    .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
            }
            "source" => {
                e.session.replace(1..1, "changed").unwrap();
            }
            "view" => {
                let selected = e.session.windows().selected_id();
                let peer = e
                    .session
                    .windows()
                    .iter()
                    .find(|(id, _)| *id != selected)
                    .unwrap()
                    .0;
                e.session.select_view(peer).unwrap();
            }
            _ => unreachable!(),
        }
        let point = e.session.view().point;
        settle(&mut e.session);
        assert_eq!(e.session.view().point, point, "{change}");
    }
    let mut e = Editor::new(
        "[keys.editor]\n'M-j'='matching-delimiter'",
        "plain",
        &marked(&source, 0),
    );
    let started = Instant::now();
    e.session.handle(key(KeyCode::Char('j'), KeyModifiers::ALT));
    let dispatch = started.elapsed();
    let queued = Instant::now();
    e.session.handle(character('x'));
    let admission = queued.elapsed();
    settle(&mut e.session);
    eprintln!(
        "mate native source_bytes={} dispatch_us={} queued_key_us={} completed_us={}",
        source.len(),
        dispatch.as_micros(),
        admission.as_micros(),
        started.elapsed().as_micros()
    );
    assert_eq!(e.text(), format!("({}x)", "a ".repeat(4096)));
}

// A mate jump creates no undo item and retains native typing barriers on both
// sides of the jump. Undo reaches each source edit without an intervening motion.
#[test]
fn history() {
    let mut e = Editor::new("", "plain", "|(ab)");
    e.type_key('x');
    let stamp = e.session.buffer().unwrap().stamp();
    let items = e.session.buffer().unwrap().history().len();
    named(&mut e);
    e.expect("x(ab|)");
    assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
    assert_eq!(e.session.buffer().unwrap().history().len(), items);
    e.type_key('y');
    e.session
        .execute(Command::History(rev::command::History::Undo));
    settle(&mut e.session);
    e.expect("x(ab|)");
    e.session
        .execute(Command::History(rev::command::History::Undo));
    settle(&mut e.session);
    e.expect("|(ab)");
}

// The shared command refuses ordinary text at point. The legacy modal percent
// command still searches later on the current line and uses its existing target.
#[test]
fn modal_entry() {
    let mut e = Editor::new("editor.interaction_profile='modal'", "plain", "|hello (ab)");
    named(&mut e);
    e.expect("|hello (ab)");
    let command = Command::named("modal-matching-delimiter").unwrap();
    e.session.execute(command);
    settle(&mut e.session);
    e.expect("hello (ab|)");
}

// A scalar delimiter preceded by a Unicode prepend character belongs to that
// grapheme. Normalize forward/backward without splitting the native cluster.
#[test]
fn graphemes() {
    for (source, start, end) in [("(\u{600})", 0, 3), ("\u{600}(a)", 3, 0)] {
        let mut e = Editor::new("", "plain", &marked(source, start));
        named(&mut e);
        e.expect(&marked(source, end));
        assert!(
            Coordinates::new(e.session.buffer().unwrap())
                .unwrap()
                .is_boundary(e.session.view().point.position)
                .unwrap()
        );
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Safe byte/depth refusal also preserves the visible point and linear mark.
// Oversized strings share the same byte cap as oversized ordinary atoms.
#[test]
fn limited() {
    for source in [
        format!("({})", "a".repeat(rev::pairs::MAX_BYTES)),
        format!("\"{}\"", "a".repeat(rev::pairs::MAX_BYTES)),
        format!("{}{}", "(".repeat(257), ")".repeat(257)),
    ] {
        let mut e = Editor::new("", "plain", &marked(&source, 0));
        e.session.execute(Command::Region(RegionCommand::Set));
        let stamp = e.session.buffer().unwrap().stamp();
        named(&mut e);
        e.expect(&marked(&source, 0));
        assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
        assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(0)));
        assert!(e.session.buffer().unwrap().mark_active());
        assert!(e.session.buffer().unwrap().history().is_empty());
        assert!(echo(&mut e).contains("No confirmed matching"));
    }
}

fn complete(buffer: &mut Buffer, ticket: Ticket) -> (Answer, Duration) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut maximum = Duration::ZERO;
    loop {
        assert!(Instant::now() < deadline, "mate request did not settle");
        let started = Instant::now();
        let status = buffer.poll_pair_request(ticket, started + SLICE_TIME);
        maximum = maximum.max(started.elapsed());
        match status {
            Status::Pending => (),
            Status::Ready => return (buffer.pair_answer(ticket).unwrap().clone(), maximum),
            Status::Retired => panic!("live mate request retired"),
        }
    }
}

// Measurements use fresh buffer-owned requests. Complete/malformed 1 KiB through
// 1 MiB sources assert aggregate caps; the oversized source must refuse safely.
#[test]
fn resources() {
    let mut buffers = Buffers::new().unwrap();
    for size in [1024, 65_536, 262_144, 1_048_576] {
        for malformed in [false, true] {
            let source = format!(
                "({}{}",
                "a".repeat(size - 2),
                if malformed { "]" } else { ")" }
            );
            let id = buffers.create(&source).unwrap();
            let buffer = buffers.get_mut(id).unwrap();
            let started = Instant::now();
            let ticket = buffer.start_mate_request(Scalar(0)).unwrap();
            let (answer, maximum) = complete(buffer, ticket);
            let elapsed = started.elapsed();
            assert!(answer.stats.inspected_bytes <= rev::pairs::MAX_BYTES);
            assert!(answer.stats.retained_bytes <= rev::pairs::MAX_METADATA);
            assert!(answer.stats.maximum_slice_tokens <= rev::pairs::SLICE_TOKENS);
            assert!(
                expected_outcome(&answer.outcome, size, malformed),
                "{answer:?}"
            );
            eprintln!(
                "mate bytes={size} malformed={malformed} total_us={} max_poll_us={} stats={:?}",
                elapsed.as_micros(),
                maximum.as_micros(),
                answer.stats
            );
        }
    }
    for depth in [256, 257] {
        let id = buffers
            .create(&format!("{}{}", "(".repeat(depth), ")".repeat(depth)))
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let ticket = buffer.start_mate_request(Scalar(0)).unwrap();
        let (answer, _) = complete(buffer, ticket);
        assert!(answer.stats.maximum_depth <= rev::pairs::MAX_DEPTH);
        assert_eq!(
            matches!(answer.outcome, Outcome::Confirmed(_)),
            depth == 256
        );
        if depth == 257 {
            assert_eq!(answer.outcome, Outcome::Limited(Limit::Depth));
        }
    }
}

fn expected_outcome(outcome: &Outcome, size: usize, malformed: bool) -> bool {
    if size > rev::pairs::MAX_BYTES {
        matches!(outcome, Outcome::Limited(Limit::Bytes))
    } else if malformed {
        matches!(outcome, Outcome::Mismatched { .. })
    } else {
        matches!(outcome, Outcome::Confirmed(_))
    }
}
