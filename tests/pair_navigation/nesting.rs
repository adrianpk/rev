use std::time::Instant;

use rev::buffer::Buffers;
use rev::command::{Command, RegionCommand};
use rev::coordinates::Scalar;
use rev::pairs::{GroupOutcome, GroupRequest, Limit};
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
    mode: String,
    initial: String,
    point_before: usize,
    keys: String,
    point: usize,
    error: Option<String>,
}

// All twelve frozen up/down observations apply through the declared native
// Plain-compatible or C profile. Reference inputs and errors remain unchanged.
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
        .filter(|c| c.keys.ends_with("C-M-u") || c.keys.ends_with("C-M-d"))
    {
        count += 1;
        let language = if case.mode == "c-mode" { "c" } else { "plain" };
        let mut e = Editor::new("", language, &marked(&case.initial, case.point_before));
        let stamp = e.session.buffer().unwrap().stamp();
        keys(&mut e, &case.keys);
        e.expect(&marked(&case.initial, case.point));
        assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
        assert!(e.session.buffer().unwrap().history().is_empty());
        if case.error.is_some() {
            assert!(echo(&mut e).contains("No qualified group"));
        }
    }
    assert_eq!(count, 12);
}

// Nesting cannot be replaced by the nearest delimiter: entry changes level,
// and counted backward entry must stay within the group just entered.
#[test]
fn endpoints() {
    for (source, point, down, count, target) in [
        ("(a [b] c)", 0, true, 1, 1),
        ("(a [b] c)", 1, true, 1, 4),
        ("(a [b] c)", 4, false, 1, 3),
        ("(a [b] c)", 3, false, 1, 0),
        ("(a [b] c)", 5, false, 1, 3),
        ("(a [b] c)", 6, false, 1, 0),
        ("(a [b] c)", 9, false, 1, 9),
        ("(a [b] c)", 4, false, -1, 6),
        ("(a [b] c)", 4, false, -2, 9),
        ("(a [b] c)", 3, false, -1, 9),
        ("(a [b] c)", 9, true, -2, 5),
        ("((x)) (y)", 9, true, -2, 9),
        ("(x) ((y))", 9, true, -2, 7),
        ("(a [b] c)", 4, true, 1, 4),
        ("[a)", 3, true, -1, 2),
        ("[a)", 1, false, -1, 3),
        ("a b", 0, true, 1, 0),
        ("(a)", 0, false, 1, 0),
        ("(a)", 3, true, 1, 3),
    ] {
        let mut e = Editor::build("pairs.enabled=false", "plain", &marked(source, point), true);
        e.session.execute(Command::Region(RegionCommand::Set));
        let stamp = e.session.buffer().unwrap().stamp();
        let chord = if down { "C-M-d" } else { "C-M-u" };
        keys(&mut e, &format!("C-u {count} {chord}"));
        e.expect(&marked(source, target));
        let buffer = e.session.buffer().unwrap();
        assert_eq!(buffer.mark(), Some(Scalar(point)));
        assert!(buffer.mark_active());
        assert_eq!(buffer.stamp(), stamp);
        assert!(buffer.history().is_empty());
    }
}

// Large/zero counts and a missing later level are atomic, including a nonempty
// active linear mark. No private intermediate destination becomes the point.
#[test]
fn counts() {
    for (input, target) in [
        ("C-u 2 C-M-d", 4),
        ("C-u 3 C-M-d", 0),
        ("C-u 1000000 C-M-d", 0),
        ("C-u 0 C-M-d", 0),
        ("C-u -1000000 C-M-u", 0),
    ] {
        let mut e = Editor::new("", "plain", "|(a [b])");
        e.session.execute(Command::Region(RegionCommand::Set));
        keys(&mut e, input);
        e.expect(&marked("(a [b])", target));
        assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(0)));
        assert!(e.session.buffer().unwrap().mark_active());
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
    let mut e = Editor::new("", "plain", "(a [|b])");
    e.session.execute(Command::Region(RegionCommand::Set));
    keys(&mut e, "C-u 2 C-M-u");
    e.expect("|(a [b])");
    assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(4)));
    assert!(e.session.buffer().unwrap().mark_active());
    keys(&mut e, "C-u 1000000 C-M-u");
    e.expect("|(a [b])");
    assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(4)));
    assert!(e.session.buffer().unwrap().mark_active());
}

// Each mode reuses its existing admitted strings/comments. Unknown raw,
// template and embedded contexts cannot become structural opening authority.
#[test]
fn profiles() {
    for (language, source) in [
        ("c", "/* (fake) */ (real)"),
        ("rust", "'x' (real)"),
        ("go", "`(fake)` (real)"),
        ("javascript", "'(fake)' (real)"),
        ("json", "\"(fake)\" (real)"),
        ("toml", "'(fake)' (real)"),
        ("css", "/* (fake) */ (real)"),
        ("html", "<b title=\"(fake)\"> (real)"),
        ("org", "=fake()= (real)"),
        ("markdown", "`fake()` (real)"),
    ] {
        let mut e = Editor::new(
            "pairs.enabled=false\npairs.quotes=false",
            language,
            &marked(source, 0),
        );
        named(&mut e, "down-pair");
        e.expect(&marked(source, source.find("(real)").unwrap() + 1));
        named(&mut e, "up-pair");
        e.expect(&marked(source, source.find("(real)").unwrap()));
    }
    for (language, source) in [
        ("rust", "r#\"(fake)\"# (real)"),
        ("javascript", "`(fake)` (real)"),
        ("html", "<script>(fake)</script> (real)"),
    ] {
        let mut e = Editor::new("", language, &marked(source, 0));
        named(&mut e, "down-pair");
        e.expect(&marked(source, 0));
    }
}

// Native source coordinates normalize only when a delimiter endpoint falls
// inside a grapheme. Forward/backward normalization preserves whole CRLFs.
#[test]
fn graphemes() {
    for (source, point, input, target) in [
        ("(\u{301}x)", 0, "C-M-d", 2),
        ("\u{600}(x)", 3, "C-M-u", 0),
        ("(a)\u{301}", 1, "C-u -1 C-M-u", 4),
        ("(\u{600})", 3, "C-u -1 C-M-d", 1),
        ("(a\r\n[b])", 0, "C-u 2 C-M-d", 5),
    ] {
        let mut e = Editor::new("", "plain", &marked(source, point));
        keys(&mut e, input);
        e.expect(&marked(source, target));
        assert!(
            rev::coordinates::Coordinates::new(e.session.buffer().unwrap())
                .unwrap()
                .is_boundary(Scalar(target))
                .unwrap()
        );
    }
}

// Each nesting direction measures a fresh complete/malformed request through
// one budget. Nesting can succeed at a proved prefix without reading its suffix.
#[test]
fn resources() {
    let mut buffers = Buffers::new().unwrap();
    for size in [1024, 65_536, 262_144, 1_048_576] {
        for operation in ["up", "up-reverse", "down", "down-reverse"] {
            for malformed in [false, true] {
                measure(&mut buffers, size, operation, malformed);
            }
        }
    }
    for depth in [256, 257] {
        let id = buffers
            .create(&format!("{}{}", "(".repeat(depth), ")".repeat(depth)))
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let count = i32::try_from(depth).unwrap();
        let ticket = buffer
            .start_group_request(GroupRequest::nesting(Scalar(0), true, count).unwrap())
            .unwrap();
        let (answer, _) = complete(buffer, ticket);
        bounded(&answer);
        let expected = if depth == 257 {
            GroupOutcome::Limited(Limit::Depth)
        } else {
            GroupOutcome::Target(Scalar(depth))
        };
        assert_eq!(answer.outcome, expected);
    }
    let source = "()".repeat(rev::pairs::MAX_BYTES / 2);
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let ticket = buffer
        .start_group_request(GroupRequest::nesting(Scalar(source.len()), true, -1).unwrap())
        .unwrap();
    let (answer, _) = complete(buffer, ticket);
    bounded(&answer);
    assert_eq!(answer.outcome, GroupOutcome::Limited(Limit::Retention));
}

fn measure(buffers: &mut Buffers, size: usize, operation: &str, malformed: bool) {
    let closing = if malformed { 'a' } else { '"' };
    let (source, point, down, count, target) = match operation {
        "down" => (
            format!("\"{}{closing} (x)", "a".repeat(size - 6)),
            0,
            true,
            1,
            size - 2,
        ),
        "up" => (
            format!("(\"{}{closing})", "a".repeat(size - 4)),
            size - 1,
            false,
            1,
            0,
        ),
        "up-reverse" => (
            format!("(\"{}{closing})", "a".repeat(size - 4)),
            1,
            false,
            -1,
            size,
        ),
        _ => (
            format!("(\"{}{closing})", "a".repeat(size - 4)),
            size,
            true,
            -1,
            size - 1,
        ),
    };
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let started = Instant::now();
    let ticket = buffer
        .start_group_request(GroupRequest::nesting(Scalar(point), down, count).unwrap())
        .unwrap();
    let (answer, maximum) = complete(buffer, ticket);
    bounded(&answer);
    let expected = if size > rev::pairs::MAX_BYTES {
        GroupOutcome::Limited(Limit::Bytes)
    } else if malformed {
        GroupOutcome::NoGroup
    } else {
        GroupOutcome::Target(Scalar(target))
    };
    assert_eq!(answer.outcome, expected, "{operation} {size} {malformed}");
    eprintln!(
        "nesting bytes={size} operation={operation} malformed={malformed} total_us={} max_poll_us={} stats={:?}",
        started.elapsed().as_micros(),
        maximum.as_micros(),
        answer.stats
    );
}
