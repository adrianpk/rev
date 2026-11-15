use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Buffer, Buffers};
use rev::command::{Command, Motion, RegionCommand, Split};
use rev::coordinates::Scalar;
use rev::input::keymap::Sequence;
use rev::pairs::{
    GroupAnswer, GroupOutcome, GroupRequest, Limit, Outcome, Query, SLICE_TIME, Status, Ticket,
};
use serde::Deserialize;

use crate::editor::{Editor, character, key, settle};

mod keymaps;

#[derive(Deserialize)]
struct Reference {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    case: String,
    mode: String,
    initial: String,
    point_before: usize,
    mark_before: Option<usize>,
    active_before: bool,
    read_only: bool,
    keys: String,
    point: usize,
    mark_active: bool,
    error: Option<String>,
}

pub(super) fn marked(source: &str, point: usize) -> String {
    let mut source = source.to_owned();
    let at = source
        .char_indices()
        .nth(point)
        .map_or(source.len(), |(at, _)| at);
    source.insert(at, '|');
    source
}

pub(super) fn keys(e: &mut Editor, input: &str) {
    for chord in input.split_whitespace() {
        if let Ok(number) = chord.parse::<i32>() {
            for c in number.to_string().chars() {
                e.session.handle(character(c));
            }
        } else {
            for key in Sequence::parse(chord).unwrap().chords() {
                e.session
                    .handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
            }
        }
    }
    settle(&mut e.session);
}

pub(super) fn named(e: &mut Editor, name: &str) {
    e.session.execute(Command::ExecuteExtended);
    e.session.handle(Event::Paste(name.into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(&mut e.session);
}

fn reference_editor(case: &Case) -> Editor {
    let language = match case.mode.as_str() {
        "c-mode" => "c",
        "org-mode" => "org",
        _ => "plain",
    };
    let at = case.mark_before.unwrap_or(case.point_before);
    let mut e = Editor::build("", language, &marked(&case.initial, at), case.read_only);
    if case.active_before {
        e.session.execute(Command::Region(RegionCommand::Set));
        e.session.execute(Command::Motion(Motion::BufferStart));
        for _ in 0..case.point_before {
            e.session.execute(Command::Motion(Motion::ForwardChar));
        }
        settle(&mut e.session);
    }
    e
}

// Frozen observations are selected by the two delivered operations. The Lisp
// semicolon-comment case has no native Lisp profile; Plain's independently
// observed semicolon case applies instead of inventing Lisp reader semantics.
#[test]
fn reference() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../ops/default/report/rev-delimiter-navigation/reference-navigation.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in reference.cases.iter().filter(|c| {
        (c.keys.ends_with("C-M-n") || c.keys.ends_with("C-M-p")) && c.case != "group_lisp_comment"
    }) {
        count += 1;
        let mut e = reference_editor(case);
        let stamp = e.session.buffer().unwrap().stamp();
        keys(&mut e, &case.keys);
        assert_eq!(
            e.session.view().point.position,
            Scalar(case.point),
            "{}",
            case.case
        );
        assert_eq!(e.text(), case.initial, "{}", case.case);
        let buffer = e.session.buffer().unwrap();
        assert_eq!(buffer.stamp(), stamp, "{}", case.case);
        assert_eq!(buffer.mark_active(), case.mark_active, "{}", case.case);
        assert_eq!(buffer.mark(), case.mark_before.map(Scalar), "{}", case.case);
        assert!(buffer.history().is_empty(), "{}", case.case);
        if case.error.is_some() {
            assert!(echo(&mut e).contains("No qualified group"), "{}", case.case);
        }
    }
    assert_eq!(count, 31);
}

pub(super) fn echo(e: &mut Editor) -> String {
    e.session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text())
        .collect::<Vec<_>>()
        .join("\n")
}

// Backward traversal uses completed source groups at the current nesting level.
// Neither motion substitutes an atom/string endpoint or crosses an enclosing end.
#[test]
fn endpoints() {
    for (source, point, command, target) in [
        ("(a) (b)", 7, "backward-pair", 4),
        ("(a) (b)", 4, "backward-pair", 0),
        ("(a [b] c)", 6, "backward-pair", 3),
        ("(a [b] c)", 3, "backward-pair", 3),
        ("(a [b] c)", 1, "forward-pair", 6),
        ("(a [b] c)", 4, "forward-pair", 4),
        ("tail", 2, "forward-pair", 4),
        ("tail", 2, "backward-pair", 0),
        ("\"()\" [a]", 8, "backward-pair", 5),
        ("[a)", 3, "backward-pair", 0),
        ("(a) /*hidden ()*/ tail", 22, "backward-pair", 0),
    ] {
        let mut e = Editor::new("", "c", &marked(source, point));
        named(&mut e, command);
        e.expect(&marked(source, target));
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Signed counts are admitted once and refuse atomically across a bad later
// group. Large counts saturate only at normal boundaries, preserving empty marks.
#[test]
fn counts() {
    for (source, point, count, command, target) in [
        ("(a) (b)", 0, "2", "C-M-n", 7),
        ("(a) (b)", 7, "2", "C-M-p", 0),
        ("(a) (b)", 7, "-2", "C-M-n", 0),
        ("(a) (b)", 0, "-2", "C-M-p", 7),
        ("(a)", 1, "0", "C-M-n", 1),
        ("(a) (b", 0, "2", "C-M-n", 0),
        ("(a) (b)", 0, "1000000", "C-M-n", 7),
        ("(a) (b)", 7, "1000000", "C-M-p", 0),
    ] {
        let mut e = Editor::new(
            "pairs.enabled=false\npairs.quotes=false",
            "c",
            &marked(source, point),
        );
        e.session.execute(Command::Region(RegionCommand::Set));
        let mark = e.session.buffer().unwrap().mark();
        keys(&mut e, &format!("C-u {count} {command}"));
        e.expect(&marked(source, target));
        assert_eq!(e.session.buffer().unwrap().mark(), mark);
        assert!(e.session.buffer().unwrap().mark_active());
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Recognized language strings/comments remain excluded with editing disabled.
// Unproved raw/triple/template/embedded contexts refuse rather than guessing.
#[test]
fn profiles() {
    for (language, source) in [
        ("c", "/* (x) */ (a)"),
        ("rust", "/* (x) */ (a)"),
        ("go", "`(x)` (a)"),
        ("javascript", "'(x)' (a)"),
        ("typescript", "\"(x)\" (a)"),
        ("json", "\"(x)\" (a)"),
        ("toml", "'(x)'\n(a)"),
        ("css", "/* (x) */ (a)"),
        ("html", "<b title=\"(x)\"> (a)"),
        ("plain", "; (x)"),
    ] {
        let mut e = Editor::new(
            "pairs.enabled=false\npairs.quotes=false",
            language,
            &marked(source, 0),
        );
        named(&mut e, "forward-pair");
        e.expect(&marked(source, source.chars().count()));
    }
    for (language, source) in [
        ("rust", "r#\"(x)\"# (a)"),
        ("rust", "'life (a)"),
        ("json", "'(x)' (a)"),
        ("html", "\"(x)\" (a)"),
        ("javascript", "`(x)` (a)"),
        ("toml", "\"\"\"(x)\"\"\" (a)"),
        ("html", "<script>(a)</script>"),
    ] {
        let mut e = Editor::new("", language, &marked(source, 0));
        named(&mut e, "forward-pair");
        e.expect(&marked(source, 0));
    }
}

// Group outcomes cannot be borrowed as typed proof. Switching request kinds
// retires the previous ticket without resetting a counted traversal's own budget.
#[test]
fn qualification() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("[a)").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let request = GroupRequest::new(Scalar(0), false, 1).unwrap();
    let ticket = buffer.start_group_request(request).unwrap();
    let (answer, _) = complete(buffer, ticket);
    assert_eq!(answer.outcome, GroupOutcome::Target(Scalar(3)));
    assert!(buffer.pair_answer(ticket).is_none());
    let typed = buffer.start_pair_request(Query::At(Scalar(0))).unwrap();
    assert!(buffer.group_answer(ticket).is_none());
    assert_eq!(
        buffer.poll_pair_request(ticket, Instant::now() + SLICE_TIME),
        Status::Retired
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while buffer.poll_pair_request(typed, Instant::now() + SLICE_TIME) == Status::Pending {
        assert!(Instant::now() < deadline, "typed request did not settle");
    }
    assert!(matches!(
        buffer.pair_answer(typed).unwrap().outcome,
        Outcome::Mismatched { .. }
    ));
    assert!(GroupRequest::new(Scalar(0), false, 1_000_001).is_none());
    assert!(GroupRequest::new(Scalar(0), false, i32::MIN).is_none());
}

// Aggregate count, oversized string and depth exhaustion publish no intermediate
// target or edit, and retain an empty active linear region at the original point.
#[test]
fn limited() {
    for source in [
        "() ".repeat(100_000),
        format!("(\"{}\")", "a".repeat(rev::pairs::MAX_BYTES)),
        format!("{}{}", "(".repeat(257), ")".repeat(257)),
    ] {
        let mut e = Editor::new("", "plain", &marked(&source, 0));
        e.session.execute(Command::Region(RegionCommand::Set));
        let stamp = e.session.buffer().unwrap().stamp();
        keys(&mut e, "C-u 100000 C-M-n");
        assert_eq!(e.text(), source);
        assert_eq!(e.session.view().point.position, Scalar(0));
        assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
        assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(0)));
        assert!(e.session.buffer().unwrap().mark_active());
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Pending motion holds later input in the existing FIFO. Cancellation/source or
// selected-view replacement prevents both partial movement and stale publication.
#[test]
fn lifecycle() {
    let source = format!("({})", "a ".repeat(4096));
    for change in ["cancel", "source", "view"] {
        let mut e = Editor::new("", "plain", &marked(&source, 0));
        if change == "view" {
            e.session.execute(Command::Split(Split::Right));
        }
        e.session.execute(Command::ForwardPair);
        assert_eq!(e.session.view().point.position, Scalar(0));
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
    let mut e = Editor::new("", "plain", &marked(&source, 0));
    let started = Instant::now();
    e.session.handle(key(
        KeyCode::Char('n'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    ));
    let dispatch = started.elapsed();
    e.session.handle(character('x'));
    settle(&mut e.session);
    eprintln!(
        "groups native bytes={} dispatch_us={} completed_us={}",
        source.len(),
        dispatch.as_micros(),
        started.elapsed().as_micros()
    );
    assert_eq!(e.text(), format!("{source}x"));
}

pub(super) fn complete(buffer: &mut Buffer, ticket: Ticket) -> (GroupAnswer, Duration) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut maximum = Duration::ZERO;
    loop {
        assert!(Instant::now() < deadline, "group request did not settle");
        let started = Instant::now();
        let status = buffer.poll_pair_request(ticket, started + SLICE_TIME);
        maximum = maximum.max(started.elapsed());
        match status {
            Status::Pending => (),
            Status::Ready => return (buffer.group_answer(ticket).unwrap().clone(), maximum),
            Status::Retired => panic!("live group request retired"),
        }
    }
}

pub(super) fn bounded(answer: &GroupAnswer) {
    assert!(answer.stats.inspected_bytes <= rev::pairs::MAX_BYTES);
    assert!(answer.stats.retained_bytes <= rev::pairs::MAX_METADATA);
    assert!(answer.stats.maximum_depth <= rev::pairs::MAX_DEPTH);
    assert!(answer.stats.maximum_slice_tokens <= rev::pairs::SLICE_TOKENS);
}

// Each size uses fresh forward/backward requests and one source budget. Depth,
// metadata and oversized string/atom limits must yield refusal without a target.
#[test]
fn resources() {
    let mut buffers = Buffers::new().unwrap();
    for size in [1024, 65_536, 262_144, 1_048_576] {
        for backward in [false, true] {
            for malformed in [false, true] {
                measure(&mut buffers, size, backward, malformed);
            }
        }
    }
    for depth in [256, 257] {
        let id = buffers
            .create(&format!("{}{}", "(".repeat(depth), ")".repeat(depth)))
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let ticket = buffer
            .start_group_request(GroupRequest::new(Scalar(0), false, 1).unwrap())
            .unwrap();
        let (answer, _) = complete(buffer, ticket);
        bounded(&answer);
        assert_eq!(
            matches!(answer.outcome, GroupOutcome::Limited(Limit::Depth)),
            depth == 257
        );
    }
    let source = "()".repeat(rev::pairs::MAX_BYTES / 2);
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let ticket = buffer
        .start_group_request(GroupRequest::new(Scalar(source.len()), true, 1_000_000).unwrap())
        .unwrap();
    let (answer, _) = complete(buffer, ticket);
    bounded(&answer);
    assert_eq!(answer.outcome, GroupOutcome::Limited(Limit::Retention));
}

fn measure(buffers: &mut Buffers, size: usize, backward: bool, malformed: bool) {
    let end = if malformed { " " } else { ")" };
    let id = buffers
        .create(&format!("({}{end}", "a".repeat(size - 2)))
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let request = GroupRequest::new(Scalar(if backward { size } else { 0 }), backward, 1).unwrap();
    let started = Instant::now();
    let ticket = buffer.start_group_request(request).unwrap();
    let (answer, maximum) = complete(buffer, ticket);
    bounded(&answer);
    let expected = if size > rev::pairs::MAX_BYTES {
        GroupOutcome::Limited(Limit::Bytes)
    } else if malformed {
        GroupOutcome::NoGroup
    } else {
        GroupOutcome::Target(Scalar(if backward { 0 } else { size }))
    };
    assert_eq!(answer.outcome, expected);
    eprintln!(
        "groups bytes={size} backward={backward} malformed={malformed} total_us={} max_poll_us={} stats={:?}",
        started.elapsed().as_micros(),
        maximum.as_micros(),
        answer.stats
    );
}
