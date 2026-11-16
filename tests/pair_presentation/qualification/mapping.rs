use super::{Harness, complete, request, view};
use rev::buffer::{
    Buffer,
    edit::{History, Replacement},
};
use rev::pairs::{Delimiter, Provenance, Status};
use std::time::Instant;

fn replace(b: &mut Buffer, range: std::ops::Range<usize>, text: &str) {
    let prepared = b
        .prepare(
            &[Replacement {
                source: b.span(range).unwrap(),
                text: text.into(),
            }],
            &[],
            None,
        )
        .unwrap();
    b.publish(prepared, &mut []).unwrap();
}

fn oracle(source: &str, initial: &[Delimiter]) -> Vec<Delimiter> {
    let positions: Vec<_> = source
        .chars()
        .enumerate()
        .filter(|(_, c)| "()[]{}".contains(*c))
        .map(|(i, _)| i)
        .collect();
    initial
        .iter()
        .enumerate()
        .map(|(i, s)| Delimiter {
            at: rev::coordinates::Scalar(positions[i]),
            mate: rev::coordinates::Scalar(positions[positions.len() - 1 - i]),
            ..*s
        })
        .collect()
}

// Each native insertion/deletion publishes correctly mapped endpoints BEFORE
// any lexical poll. A rebuild gap would fail on every one of these revisions.
#[test]
fn neutral_bursts() {
    for (language, source) in [
        ("plain", "Body target (outer [middle {leaf}]) tail\n"),
        (
            "org",
            "* Heading\nBody target (outer [middle {leaf}]) tail\n",
        ),
        (
            "markdown",
            "# Heading\nBody target (outer [middle {leaf}]) tail\n",
        ),
        ("rust", "// 界🙂\r\n(outer [middle {leaf}])\r\n"),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        let b = h.buffers.get_mut(id).unwrap();
        let view = view(b);
        let ticket = request(b, view);
        complete(b, ticket);
        let initial = b.pair_presentation(ticket).unwrap().spans.to_vec();
        assert_eq!(initial.len(), 6);
        let mut expected = source.to_owned();
        let at = source.chars().position(|c| c == 'f').unwrap() - 1;
        for step in 0..64 {
            let insertion = step < 32;
            let end = if insertion { at } else { at + 1 };
            replace(b, at..end, if insertion { "x" } else { "" });
            let byte_start = expected.char_indices().nth(at).unwrap().0;
            let byte_end = expected.char_indices().nth(end).unwrap().0;
            expected.replace_range(byte_start..byte_end, if insertion { "x" } else { "" });
            assert_eq!(
                b.poll_pair_presentation(ticket, Instant::now()),
                Status::Retired
            );
            let next = request(b, view);
            let mapped = b
                .pair_presentation(next)
                .expect("neutral transaction retained proof before polling");
            assert_eq!(mapped.stamp, b.stamp());
            assert_eq!(mapped.provenance, Provenance::MappedLexical);
            assert_eq!(mapped.spans, oracle(&expected, &initial));
            assert_eq!(
                b.slice(&b.span(0..b.len_chars()).unwrap())
                    .unwrap()
                    .to_string(),
                expected
            );
        }
        assert_eq!(expected, source);
    }
}

// Exact multi-range replacement and native undo/redo use base/final coordinates,
// not a guessed global offset. Dropped prepared edits do not change the cache.
#[test]
fn transactions_and_history() {
    let mut h = Harness::new("plain", false);
    let id = h.source("(alpha [bravo {charlie}])");
    let b = h.buffers.get_mut(id).unwrap();
    let view = view(b);
    let ticket = request(b, view);
    complete(b, ticket);
    let initial = b.pair_presentation(ticket).unwrap().spans.to_vec();
    let edits = [
        Replacement {
            source: b.span(3..3).unwrap(),
            text: "XY".into(),
        },
        Replacement {
            source: b.span(10..11).unwrap(),
            text: "z".into(),
        },
    ];
    drop(b.prepare(&edits, &[], None).unwrap());
    assert_eq!(b.pair_presentation(ticket).unwrap().spans, initial);
    let prepared = b.prepare(&edits, &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    for direction in [None, Some(History::Undo), Some(History::Redo)] {
        if let Some(direction) = direction {
            b.navigate(direction, &mut [], None).unwrap();
        }
        let next = request(b, view);
        let mapped = b.pair_presentation(next).unwrap();
        assert_eq!(
            mapped.spans,
            oracle(
                &b.slice(&b.span(0..b.len_chars()).unwrap())
                    .unwrap()
                    .to_string(),
                &initial
            )
        );
        assert_eq!(mapped.provenance, Provenance::MappedLexical);
    }
}

// Changing lexical/nesting boundaries never maps old proof, even where scalar
// offsets happen to stay unchanged. Requalification is explicitly pending.
#[test]
fn invalidating_edits() {
    for (name, range, text) in [
        ("delimiter", 4..4, "("),
        ("quote", 4..4, "\""),
        ("escape", 4..4, "\\"),
        ("newline", 4..4, "\n"),
        ("word boundary", 1..1, "x"),
        ("replacement", 0..6, "(beta)"),
    ] {
        let mut h = Harness::new("plain", false);
        let id = h.source("(alpha)");
        let b = h.buffers.get_mut(id).unwrap();
        let view = view(b);
        let old = request(b, view);
        complete(b, old);
        replace(b, range, text);
        let next = request(b, view);
        assert!(b.pair_presentation(next).is_none(), "{name}");
        assert_eq!(
            b.poll_pair_presentation(old, Instant::now()),
            Status::Retired
        );
        complete(b, next);
    }
}

// Source-identical preparation does not authorize mapping a later publication.
// A different completed viewport between prepare/publish invalidates that proof.
#[test]
fn changed_proof_publication() {
    let mut h = Harness::new("plain", false);
    let id = h.source("(alpha) [bravo]");
    let b = h.buffers.get_mut(id).unwrap();
    let view = view(b);
    let ticket = request(b, view);
    complete(b, ticket);
    let prepared = b
        .prepare(
            &[Replacement {
                source: b.span(3..5).unwrap(),
                text: "z".into(),
            }],
            &[],
            None,
        )
        .unwrap();
    // This completed range ends inside the replacement, so blindly mapping its
    // target with the earlier range's proof would be invalid.
    let later = b
        .start_pair_presentation(view, b.span(0..4).unwrap())
        .unwrap();
    assert!(b.pair_presentation(later).is_some());
    b.select_mode(rev::buffer::Mode::Org).unwrap();
    let later = b
        .start_pair_presentation(view, b.span(0..4).unwrap())
        .unwrap();
    complete(b, later);
    b.publish(prepared, &mut []).unwrap();
    let next = request(b, view);
    assert!(b.pair_presentation(next).is_none());
    complete(b, next);
}

// Reload uses its disk/native transaction even for an otherwise neutral edit;
// read-only qualification itself preserves source, saved state and history.
#[test]
fn reload_and_read_only() {
    let mut h = Harness::new("plain", false);
    let id = h.source("(alpha)");
    let b = h.buffers.get_mut(id).unwrap();
    b.set_readonly(true);
    let view = view(b);
    let ticket = request(b, view);
    complete(b, ticket);
    let state = b.state();
    assert!(!b.is_modified() && b.history().is_empty());
    std::fs::write(b.file().unwrap().path(), "(alXpha)").unwrap();
    let checked = b
        .request_reload(1024)
        .unwrap()
        .read()
        .unwrap()
        .verify(false)
        .unwrap();
    let publication = b.prepare_reload(checked, &[]).unwrap();
    b.publish_reload(publication, &mut []).unwrap();
    let next = request(b, view);
    assert!(b.pair_presentation(next).is_none());
    complete(b, next);
    assert!(b.is_readonly() && !b.is_modified());
    assert_ne!(b.state(), state);
}

// A malformed suffix cannot revoke an earlier confirmed pair's neutral edit
// proof. Advancement preserves the conservative outcome and only proven spans.
#[test]
fn neutral_proved_prefix() {
    for source in ["(alpha) {]", "(alpha) ("] {
        let mut h = Harness::new("plain", false);
        let id = h.source(source);
        let b = h.buffers.get_mut(id).unwrap();
        let view = view(b);
        let old = request(b, view);
        complete(b, old);
        let result = b.pair_presentation(old).unwrap();
        let outcome = result.outcome;
        assert_eq!(result.spans.len(), 2);
        replace(b, 3..3, "x");
        let next = request(b, view);
        let result = b
            .pair_presentation(next)
            .expect("known prefix advances before a lexical poll");
        assert_eq!(result.outcome, outcome);
        assert_eq!(result.spans.len(), 2);
        assert_eq!((result.spans[0].at.0, result.spans[0].mate.0), (0, 7));
        assert_eq!((result.spans[1].at.0, result.spans[1].mate.0), (7, 0));
        assert_eq!(result.provenance, Provenance::MappedLexical);
    }
}

// An Org word edit can introduce inline source syntax without changing any
// punctuation. Such control words require fresh lexical qualification.
#[test]
fn org_control_word() {
    let mut h = Harness::new("org", false);
    let source = "Body (sxc_rust{leaf})\n";
    let id = h.source(source);
    let b = h.buffers.get_mut(id).unwrap();
    let view = view(b);
    let old = request(b, view);
    complete(b, old);
    assert_eq!(b.pair_presentation(old).unwrap().spans.len(), 4);
    let at = source.find('x').unwrap();
    replace(b, at..at + 1, "r");
    let next = request(b, view);
    assert!(b.pair_presentation(next).is_none());
    complete(b, next);
    let result = b.pair_presentation(next).unwrap();
    assert_eq!(
        result.outcome,
        rev::pairs::PresentationOutcome::Uncertain(rev::pairs::Reason::LiteralContext)
    );
    assert!(result.spans.is_empty());
}
