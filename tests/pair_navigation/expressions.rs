use std::time::{Duration, Instant};

use rev::buffer::Buffers;
use rev::command::{Command, Motion, RegionCommand};
use rev::coordinates::Scalar;
use rev::pairs::{GroupOutcome, GroupRequest, Limit, Outcome, Query};
use serde::Deserialize;

use super::groups::{bounded, complete, echo, keys, marked, named};
use crate::editor::{Editor, settle};

mod views;

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

// The approved Plain profile follows text-mode word apostrophes. The frozen
// fundamental-mode apostrophe observation remains unchanged and explicitly maps
// to the independently observed text-mode endpoint instead.
#[test]
fn reference() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../ops/default/report/rev-delimiter-navigation/reference-navigation.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in reference
        .cases
        .iter()
        .filter(|c| c.case.starts_with("sexp_"))
    {
        count += 1;
        let mut e = reference_editor(case);
        let stamp = e.session.buffer().unwrap().stamp();
        keys(&mut e, &case.keys);
        let point = if case.case == "sexp_plain_apostrophe" {
            5
        } else {
            case.point
        };
        assert_eq!(
            e.session.view().point.position,
            Scalar(point),
            "{}",
            case.case
        );
        assert_eq!(e.text(), case.initial, "{}", case.case);
        let buffer = e.session.buffer().unwrap();
        assert_eq!(buffer.stamp(), stamp, "{}", case.case);
        assert_eq!(buffer.mark(), case.mark_before.map(Scalar), "{}", case.case);
        assert_eq!(buffer.mark_active(), case.mark_active, "{}", case.case);
        assert!(buffer.history().is_empty(), "{}", case.case);
        if case.error.is_some() {
            assert!(echo(&mut e).contains("No qualified group"), "{}", case.case);
        }
    }
    assert_eq!(count, 37);
}

fn reference_editor(case: &Case) -> Editor {
    let language = match case.mode.as_str() {
        "c-mode" => "c",
        "js-mode" => "javascript",
        "css-mode" => "css",
        "html-mode" => "html",
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

// Syntax classes, not parser token extents, determine atom boundaries. Reverse
// motion from within a unit uses its qualified prefix, not the previous group.
#[test]
fn atoms() {
    for (language, source, point, backward, target) in [
        ("c", "12.5 + 7", 2, false, 4),
        ("c", "foo::bar", 4, true, 0),
        ("c", "alpha_beta.next", 8, true, 0),
        ("c", "alpha_beta.next", 3, false, 10),
        ("c", "*value + 7", 6, true, 1),
        ("c", "one + two", 9, true, 6),
        ("plain", "a+b/c=d next", 0, false, 7),
        ("plain", "can't next", 4, true, 0),
        ("css", "font-size: 12px", 9, true, 0),
        ("css", ".class-name next", 0, false, 11),
        ("javascript", "$value.next", 6, true, 0),
        ("typescript", "$value.next", 0, false, 6),
        ("tsx", "$value.next", 0, false, 6),
        ("rust", "value_12.next", 0, false, 8),
        ("go", "value_12.next", 0, false, 8),
        ("json", "12.5", 0, false, 2),
        ("toml", "name_12.other", 0, false, 7),
        ("unknown", "alpha-beta next", 0, false, 10),
        ("org", "can't next", 0, false, 5),
        ("markdown", "can't next", 0, false, 5),
    ] {
        let mut e = Editor::build(
            "pairs.enabled=false\npairs.quotes=false",
            language,
            &marked(source, point),
            true,
        );
        named(
            &mut e,
            if backward {
                "backward-sexp"
            } else {
                "forward-sexp"
            },
        );
        e.expect(&marked(source, target));
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Whole quotes and forms are expression units; interior words remain syntax
// units when point already lies in an ordinary string. Escapes hide punctuation.
#[test]
fn strings_and_forms() {
    for (language, source, point, backward, target) in [
        ("c", "(a [b]) tail", 0, false, 7),
        ("c", "(a [b]) tail", 7, true, 0),
        ("c", "(a [b]) tail", 4, false, 5),
        ("c", "[a)", 0, false, 3),
        ("c", "\"abc def\" tail", 3, false, 4),
        ("c", "\"abc def\" tail", 3, true, 1),
        ("c", "\"a \\\" ) b\" next", 0, false, 10),
        ("go", "`(hidden)` tail", 0, false, 10),
        ("rust", "'x' tail", 0, false, 3),
        ("toml", "'raw \\ word' tail", 0, false, 12),
        ("html", "<div title=\"a\">x</div>", 15, true, 0),
        ("html", "<div title=\"a\">x</div>", 15, false, 16),
        ("html", "<div title=\"a\">x</div>", 16, false, 22),
        ("html", "<div title=\"a\">x</div>", 11, false, 14),
        ("html", "<div title=\"abc def\">x", 14, false, 15),
        ("html", "<div title=\"abc def\">x", 14, true, 12),
        ("c", "/* fake() */ alpha", 0, false, 18),
        ("rust", "/* a /* b */ c */ real", 0, false, 22),
        ("javascript", "// hidden()\r\nreal", 0, false, 17),
    ] {
        let mut e = Editor::new("", language, &marked(source, point));
        keys(&mut e, if backward { "C-M-b" } else { "C-M-f" });
        e.expect(&marked(source, target));
    }
    let source = format!("{}<div title=\"x\"> tail", " ".repeat(255));
    let mut e = Editor::new("", "html", &marked(&source, 0));
    keys(&mut e, "C-M-f");
    e.expect(&marked(&source, source.find('>').unwrap() + 1));
}

// Counts share one private destination; a later malformed or ambiguous unit
// cannot publish the earlier atom. Successful movement preserves empty marks.
#[test]
fn counts_and_refusal() {
    for (language, source, point, input, target) in [
        ("c", "alpha beta", 0, "C-u 2 C-M-f", 10),
        ("c", "alpha beta", 10, "C-u -2 C-M-f", 0),
        ("c", "alpha beta", 0, "C-u -2 C-M-b", 10),
        ("c", "alpha beta", 3, "C-u 0 C-M-f", 3),
        ("c", "alpha beta", 0, "C-u 1000000 C-M-f", 10),
        ("c", "alpha beta", 10, "C-u 1000000 C-M-b", 0),
        ("c", "alpha (b", 0, "C-u 2 C-M-f", 0),
        ("c", "alpha )", 0, "C-u 2 C-M-f", 0),
        ("c", "\"abc", 0, "C-M-f", 4),
        ("c", "alpha \"abc", 0, "C-u 2 C-M-f", 10),
        ("rust", "alpha r#\"x\"#", 0, "C-u 2 C-M-f", 0),
        ("rust", "'life value", 0, "C-M-f", 0),
        ("javascript", "`template`", 0, "C-M-f", 0),
        ("javascript", "<Component/>", 0, "C-M-f", 0),
        ("toml", "\"\"\"multiline\"\"\"", 0, "C-M-f", 0),
        ("html", "<script>x</script>", 0, "C-M-f", 0),
        ("html", "<div title=\"unterminated", 0, "C-M-f", 0),
        ("html", "< unknown", 0, "C-M-f", 0),
        ("html", "<div<inner>", 0, "C-M-f", 0),
        ("json", "'invalid'", 0, "C-M-f", 0),
        ("org", "=literal()= next", 2, "C-M-f", 2),
        ("markdown", "`literal()` next", 2, "C-M-f", 2),
    ] {
        let mut e = Editor::build("", language, &marked(source, point), true);
        e.session.execute(Command::Region(RegionCommand::Set));
        let stamp = e.session.buffer().unwrap().stamp();
        keys(&mut e, input);
        assert_eq!(
            e.session.view().point.position,
            Scalar(target),
            "{language} {source} {input}"
        );
        let buffer = e.session.buffer().unwrap();
        assert_eq!(buffer.mark(), Some(Scalar(point)));
        assert!(buffer.mark_active());
        assert_eq!(buffer.stamp(), stamp);
        assert!(buffer.history().is_empty());
    }
}

// Scalar endpoints are normalized in movement direction before publication,
// including combining, prepend and CRLF boundaries.
#[test]
fn graphemes() {
    for (source, point, backward, target) in [
        ("e\u{301} next", 0, false, 2),
        ("e\u{301} next", 2, true, 0),
        ("\u{600}a next", 0, false, 2),
        ("\u{600}a next", 2, true, 0),
        ("a\r\nb", 1, false, 4),
        ("a\r\nb", 3, true, 0),
        ("a\u{2028}b", 0, false, 1),
    ] {
        let mut e = Editor::new("", "plain", &marked(source, point));
        keys(&mut e, if backward { "C-M-b" } else { "C-M-f" });
        e.expect(&marked(source, target));
    }
}

// Navigation's loose group endpoint and a known unterminated-string EOF cannot
// become editing authority through the strict typed-pair answer API.
#[test]
fn qualification() {
    let mut buffers = Buffers::new().unwrap();
    for source in ["[a)", "\"abc"] {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let ticket = buffer
            .start_group_request(GroupRequest::expression(Scalar(0), false, 1).unwrap())
            .unwrap();
        let _ = complete(buffer, ticket);
        assert!(buffer.pair_answer(ticket).is_none());
        let typed = buffer.start_pair_request(Query::At(Scalar(0))).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while buffer.poll_pair_request(typed, Instant::now() + rev::pairs::SLICE_TIME)
            == rev::pairs::Status::Pending
        {
            assert!(Instant::now() < deadline, "typed request did not settle");
        }
        assert!(!matches!(
            buffer.pair_answer(typed).unwrap().outcome,
            Outcome::Confirmed(_)
        ));
        assert!(buffer.group_answer(ticket).is_none());
    }
}

// A limited later expression never publishes the private earlier destination.
// Byte/depth/retention refusal preserves source, stamp, active mark and history.
#[test]
fn limited_motion() {
    for (source, backward) in [
        ("a ".repeat(150_000), false),
        ("a ".repeat(100_000), true),
        (format!("{}{}", "(".repeat(257), ")".repeat(257)), false),
        (format!("\"{}\"", "a".repeat(rev::pairs::MAX_BYTES)), false),
    ] {
        let point = if backward { source.len() } else { 0 };
        let mut e = Editor::build("", "c", &marked(&source, point), true);
        e.session.execute(Command::Region(RegionCommand::Set));
        let stamp = e.session.buffer().unwrap().stamp();
        keys(
            &mut e,
            if backward {
                "C-u 1000000 C-M-b"
            } else {
                "C-u 1000000 C-M-f"
            },
        );
        e.expect(&marked(&source, point));
        let buffer = e.session.buffer().unwrap();
        assert_eq!(buffer.stamp(), stamp);
        assert_eq!(buffer.mark(), Some(Scalar(point)));
        assert!(buffer.mark_active());
        assert!(buffer.history().is_empty());
        assert!(echo(&mut e).contains("No qualified group"));
    }
}

// Every source shape shares the byte/depth/retention and cooperative-work caps.
// Long atoms and ordinary strings cannot bypass charging through token skips.
#[test]
fn resources() {
    let mut buffers = Buffers::new().unwrap();
    for size in [1024, 65536, 262144, 1048576] {
        for shape in ["form", "malformed", "atom", "string"] {
            for backward in [false, true] {
                measure(&mut buffers, size, shape, backward);
            }
        }
    }
    for depth in [256, 257] {
        let source = format!("{}{}", "(".repeat(depth), ")".repeat(depth));
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let ticket = buffer
            .start_group_request(GroupRequest::expression(Scalar(0), false, 1).unwrap())
            .unwrap();
        let (answer, _) = complete(buffer, ticket);
        bounded(&answer);
        let expected = if depth == 257 {
            GroupOutcome::Limited(Limit::Depth)
        } else {
            GroupOutcome::Target(Scalar(source.len()))
        };
        assert_eq!(answer.outcome, expected);
    }
    let source = "a ".repeat(rev::pairs::MAX_BYTES / 2);
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let ticket = buffer
        .start_group_request(
            GroupRequest::expression(Scalar(source.len()), true, 1_000_000).unwrap(),
        )
        .unwrap();
    let (answer, _) = complete(buffer, ticket);
    bounded(&answer);
    assert_eq!(answer.outcome, GroupOutcome::Limited(Limit::Retention));
}

fn measure(buffers: &mut Buffers, size: usize, shape: &str, backward: bool) {
    let source = match shape {
        "form" => format!("({})", "a".repeat(size - 2)),
        "malformed" => format!("({}", "a".repeat(size - 1)),
        "string" => format!("\"{}\"", "a".repeat(size - 2)),
        _ => "a".repeat(size),
    };
    let point = if backward { source.len() } else { 0 };
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let started = Instant::now();
    let ticket = buffer
        .start_group_request(GroupRequest::expression(Scalar(point), backward, 1).unwrap())
        .unwrap();
    let (answer, maximum) = complete(buffer, ticket);
    bounded(&answer);
    let target = measured_target(size, shape, backward);
    let expected = if size > rev::pairs::MAX_BYTES {
        GroupOutcome::Limited(Limit::Bytes)
    } else if !backward && shape == "malformed" {
        GroupOutcome::NoGroup
    } else {
        GroupOutcome::Target(Scalar(target))
    };
    assert_eq!(answer.outcome, expected, "{size} {shape} {backward}");
    eprintln!(
        "expressions bytes={size} shape={shape} backward={backward} total_us={} max_poll_us={} stats={:?}",
        started.elapsed().as_micros(),
        maximum.as_micros(),
        answer.stats
    );
}

fn measured_target(size: usize, shape: &str, backward: bool) -> usize {
    if backward && shape == "malformed" {
        1
    } else if backward {
        0
    } else {
        size
    }
}
