use super::{Harness, complete, request, view};
use rev::coordinates::Scalar;
use rev::pairs::{GroupRequest, Outcome, PresentationOutcome, Query, SLICE_TIME, Status};
use std::time::Instant;

// A visual request cannot consume a foreground poll or retire its ticket. A
// completed answer remains borrowable when its scanner serves presentation.
#[test]
fn foreground_priority() {
    for groups in [false, true] {
        let mut h = Harness::new("plain", false);
        let id = h.source(&format!("({})", "word ".repeat(1000)));
        let b = h.buffers.get_mut(id).unwrap();
        let foreground = if groups {
            b.start_group_request(GroupRequest::new(Scalar(0), false, 1).unwrap())
                .unwrap()
        } else {
            b.start_pair_request(Query::At(Scalar(0))).unwrap()
        };
        let ticket = request(b, view(b));
        let before = b.pair_stats(foreground).unwrap();
        for _ in 0..4 {
            assert_eq!(
                b.poll_pair_presentation(ticket, Instant::now() + SLICE_TIME),
                Status::Pending
            );
        }
        assert_eq!(
            b.pair_stats(foreground).unwrap().inspected_bytes,
            before.inspected_bytes
        );
        while b.poll_pair_request(foreground, Instant::now() + SLICE_TIME) == Status::Pending {}
        let pair = b.pair_answer(foreground).cloned();
        let group = b.group_answer(foreground).cloned();
        complete(b, ticket);
        assert_eq!(
            b.poll_pair_request(foreground, Instant::now()),
            Status::Ready
        );
        assert_eq!(b.pair_answer(foreground), pair.as_ref());
        assert_eq!(b.group_answer(foreground), group.as_ref());
        assert_eq!(b.pair_presentation(ticket).unwrap().spans.len(), 2);
        if groups {
            assert!(b.cancel_pair_presentation(ticket));
            assert_eq!(b.group_answer(foreground), group.as_ref());
        }
        assert!(b.cancel_pair_request(foreground));
        if !groups {
            assert!(b.pair_presentation(ticket).is_some());
        }
    }
}

// A loose group can cross wrong delimiter types. Its result must never be
// promoted to typed endpoints, and a later edit request preempts visual work.
#[test]
fn group_is_not_typed_proof() {
    let mut h = Harness::new("plain", false);
    let id = h.source("([)]");
    let b = h.buffers.get_mut(id).unwrap();
    let group = b
        .start_group_request(GroupRequest::new(Scalar(0), false, 1).unwrap())
        .unwrap();
    while b.poll_pair_request(group, Instant::now() + SLICE_TIME) == Status::Pending {}
    let saved = b.group_answer(group).unwrap().clone();
    let ticket = request(b, view(b));
    complete(b, ticket);
    let visual = b.pair_presentation(ticket).unwrap();
    assert!(visual.spans.is_empty());
    assert!(matches!(
        visual.outcome,
        PresentationOutcome::Mismatched { .. }
    ));
    assert_eq!(b.group_answer(group), Some(&saved));
    let foreground = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    while b.poll_pair_request(foreground, Instant::now() + SLICE_TIME) == Status::Pending {}
    assert!(matches!(
        b.pair_answer(foreground).unwrap().outcome,
        Outcome::Mismatched { .. }
    ));
}

// Per-view generation replacement and cancellation cannot resurrect old
// tickets. Language/mode/settings invalidation keeps generations monotonic.
#[test]
fn view_and_source_generations() {
    let mut h = Harness::new("plain", false);
    let id = h.source("([x])");
    let b = h.buffers.get_mut(id).unwrap();
    let first_view = view(b);
    let second_view = view(b);
    let old = request(b, first_view);
    let current = request(b, first_view);
    let sibling = request(b, second_view);
    assert_eq!(
        b.poll_pair_presentation(old, Instant::now()),
        Status::Retired
    );
    complete(b, current);
    assert!(b.pair_presentation(sibling).is_some());
    assert!(!b.cancel_pair_presentation(old));
    assert!(b.cancel_pair_presentation(current));
    assert_eq!(
        b.poll_pair_presentation(current, Instant::now()),
        Status::Retired
    );
    assert!(b.pair_presentation(sibling).is_some());
    for mode in [rev::buffer::Mode::Org, rev::buffer::Mode::Text] {
        b.select_mode(mode).unwrap();
        let next = request(b, second_view);
        assert_eq!(
            b.poll_pair_presentation(sibling, Instant::now()),
            Status::Retired
        );
        complete(b, next);
    }
    let old = request(b, first_view);
    complete(b, old);
    b.apply_settings(&rev::config::Snapshot::default());
    let next = request(b, first_view);
    assert_ne!(old, next);
    assert_eq!(
        b.poll_pair_presentation(old, Instant::now()),
        Status::Retired
    );
    complete(b, next);
    b.toggle_auto_pairs();
    assert!(b.pair_presentation(next).is_some());
}

// Expired deadlines do no lexical/export work; cancellation releases the active
// background scanner without disturbing another foreground operation.
#[test]
fn cancellation_and_deadlines() {
    let mut h = Harness::new("plain", false);
    let id = h.source(&"()".repeat(5000));
    let b = h.buffers.get_mut(id).unwrap();
    let ticket = request(b, view(b));
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now()),
        Status::Pending
    );
    assert!(b.pair_presentation(ticket).is_none());
    let retained = b.retained_pair_bytes();
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now()),
        Status::Pending
    );
    assert_eq!(b.retained_pair_bytes(), retained);
    assert!(b.cancel_pair_presentation(ticket));
    assert!(b.retained_pair_bytes() < retained);
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now() + SLICE_TIME),
        Status::Retired
    );
    let foreground = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    let ticket = request(b, view(b));
    assert!(b.cancel_pair_presentation(ticket));
    assert_eq!(
        b.poll_pair_request(foreground, Instant::now() + SLICE_TIME),
        Status::Ready
    );
}

// A canceled visual consumer must give the scanner back to a completed pair
// ticket so counted continuation retains its original inspection budget.
#[test]
fn foreground_continuation() {
    let mut h = Harness::new("plain", false);
    let id = h.source("(one) [two] {three}");
    let b = h.buffers.get_mut(id).unwrap();
    let foreground = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    while b.poll_pair_request(foreground, Instant::now() + SLICE_TIME) == Status::Pending {}
    let inspected = b.pair_stats(foreground).unwrap().inspected_bytes;
    let ticket = request(b, view(b));
    complete(b, ticket);
    assert!(b.cancel_pair_presentation(ticket));
    assert!(b.pair_answer(foreground).is_some());
    assert!(
        b.continue_pair_request(foreground, Query::At(Scalar(6)))
            .unwrap()
    );
    while b.poll_pair_request(foreground, Instant::now() + SLICE_TIME) == Status::Pending {}
    assert!(matches!(
        b.pair_answer(foreground).unwrap().outcome,
        Outcome::Confirmed(_)
    ));
    assert_eq!(b.pair_stats(foreground).unwrap().inspected_bytes, inspected);
}

// An edit may retain a lexical prefix for the next foreground request without
// creating an unissued foreground ticket that blocks visual qualification.
#[test]
fn retained_prefix_is_idle() {
    let mut h = Harness::new("plain", false);
    let source = format!("{}(alpha)", "word ".repeat(100));
    let id = h.source(&source);
    let b = h.buffers.get_mut(id).unwrap();
    let foreground = b.start_pair_request(Query::At(Scalar(500))).unwrap();
    while b.poll_pair_request(foreground, Instant::now() + SLICE_TIME) == Status::Pending {}
    let edit = rev::buffer::edit::Replacement {
        source: b.span(504..504).unwrap(),
        text: "(".into(),
    };
    let prepared = b.prepare(&[edit], &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    let ticket = request(b, view(b));
    complete(b, ticket);
    assert_eq!(
        b.pair_presentation(ticket).unwrap().outcome,
        PresentationOutcome::Unmatched
    );
}

// Extending another viewport must preserve existing qualified spans during every
// poll, including when the larger request exhausts the shared metadata budget.
#[test]
fn publication_extension() {
    for (name, source, end) in [
        ("sparse", format!("(one) {}[two]", "word ".repeat(1000)), 5),
        ("dense", "()".repeat(50000), 8192),
    ] {
        let mut h = Harness::new("plain", false);
        let id = h.source(&source);
        let b = h.buffers.get_mut(id).unwrap();
        let first = b
            .start_pair_presentation(view(b), b.span(0..end).unwrap())
            .unwrap();
        complete(b, first);
        let retained = b.pair_presentation(first).unwrap().spans.to_vec();
        let next = request(b, view(b));
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        loop {
            assert!(Instant::now() < deadline);
            let status = b.poll_pair_presentation(next, Instant::now() + SLICE_TIME);
            let original = b
                .pair_presentation(first)
                .expect("current view retains its qualified publication");
            assert_eq!(original.spans, retained, "{name}");
            assert!(b.retained_pair_bytes() <= rev::pairs::MAX_METADATA);
            if status == Status::Ready {
                break;
            }
            assert_eq!(status, Status::Pending);
        }
    }
}

// Concurrent view extensions belong to one shared in-flight request; each view
// must not reset its byte budget or disguise earlier inspection as prefix reuse.
#[test]
fn overlapping_view_budget() {
    let mut h = Harness::new("plain", false);
    let id = h.source(&"word ".repeat(rev::pairs::MAX_BYTES / 5 + 10000));
    let b = h.buffers.get_mut(id).unwrap();
    let mut last = None;
    for extra in 0..20 {
        let ticket = b
            .start_pair_presentation(view(b), b.span(0..rev::pairs::MAX_BYTES + extra).unwrap())
            .unwrap();
        assert_eq!(
            b.poll_pair_presentation(ticket, Instant::now() + SLICE_TIME),
            Status::Pending
        );
        last = Some(ticket);
    }
    let last = last.unwrap();
    complete(b, last);
    let answer = b.pair_presentation(last).unwrap();
    assert_eq!(
        answer.outcome,
        PresentationOutcome::Limited(rev::pairs::Limit::Bytes)
    );
    assert_eq!(answer.stats.inspected_bytes, rev::pairs::MAX_BYTES);
    assert_eq!(answer.stats.reused_chars, 0);
}
