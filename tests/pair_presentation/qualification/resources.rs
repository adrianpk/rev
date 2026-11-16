use super::{Harness, complete, request, view};
use rev::pairs::{
    Limit, MAX_BYTES, MAX_DEPTH, MAX_METADATA, MAX_VISIBLE_SPANS, PresentationOutcome as Outcome,
    SLICE_TIME, Status,
};
use std::time::{Duration, Instant};

// Depth and byte exhaustion are not mismatches. Oversized atomic text cannot
// bypass inspection charging, and offscreen mates stay source-qualified.
#[test]
fn source_bounds() {
    for (name, source, expected) in [
        (
            "depth128",
            format!("{}{}", "(".repeat(128), ")".repeat(128)),
            Outcome::Complete,
        ),
        (
            "depth256",
            format!("{}{}", "(".repeat(256), ")".repeat(256)),
            Outcome::Complete,
        ),
        (
            "depth257",
            format!("{}{}", "(".repeat(257), ")".repeat(257)),
            Outcome::Limited(Limit::Depth),
        ),
        (
            "oversized token",
            "x".repeat(MAX_BYTES + 1),
            Outcome::Limited(Limit::Bytes),
        ),
    ] {
        let mut h = Harness::new("plain", false);
        let id = h.source(&source);
        let b = h.buffers.get_mut(id).unwrap();
        let ticket = request(b, view(b));
        let maximum = complete(b, ticket);
        let result = b.pair_presentation(ticket).unwrap();
        assert_eq!(result.outcome, expected, "{name}");
        assert!(result.stats.maximum_depth <= MAX_DEPTH);
        assert!(result.spans.iter().all(|s| s.depth < MAX_DEPTH));
        eprintln!(
            "qualification {name}: inspected={} retained={} slices={} max poll={maximum:?}",
            result.stats.inspected_bytes, result.stats.retained_bytes, result.stats.slices
        );
    }
    let mut h = Harness::new("plain", false);
    let id = h.source(&format!("({})", "word ".repeat(1000)));
    let b = h.buffers.get_mut(id).unwrap();
    let ticket = b
        .start_pair_presentation(view(b), b.span(0..1).unwrap())
        .unwrap();
    complete(b, ticket);
    let spans = b.pair_presentation(ticket).unwrap().spans;
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].mate.0, b.len_chars() - 1);
}

// Dense output is one shared allocation across views. Each viewport admits at
// most 8192 spans; another viewport can read the remainder without a full copy.
#[test]
fn shared_retention() {
    let mut h = Harness::new("plain", false);
    let id = h.source(&"()".repeat(5000));
    let b = h.buffers.get_mut(id).unwrap();
    let first = request(b, view(b));
    let maximum = complete(b, first);
    let result = b.pair_presentation(first).unwrap();
    assert_eq!(result.outcome, Outcome::Limited(Limit::Retention));
    assert_eq!(result.spans.len(), MAX_VISIBLE_SPANS);
    let address = result.spans.as_ptr();
    let initial = b.retained_pair_bytes();
    for _ in 0..20 {
        let next = request(b, view(b));
        assert_eq!(b.pair_presentation(next).unwrap().spans.as_ptr(), address);
    }
    assert!(b.retained_pair_bytes() < initial + 4096);
    let tail = b
        .start_pair_presentation(view(b), b.span(8192..10000).unwrap())
        .unwrap();
    assert_eq!(b.pair_presentation(tail).unwrap().spans.len(), 1808);
    assert_eq!(
        b.pair_presentation(tail).unwrap().outcome,
        Outcome::Complete
    );
    eprintln!(
        "qualification dense: endpoint_size={} retained={initial}, 22 views retained={}, max poll={maximum:?}",
        std::mem::size_of::<rev::pairs::Delimiter>(),
        b.retained_pair_bytes()
    );
}

// Scanner facts, pending export and output all participate in the same cap.
// Exhaustion terminates conservatively and cancellation releases the allocation.
#[test]
fn aggregate_retention() {
    let mut h = Harness::new("plain", false);
    let id = h.source(&"()".repeat(50000));
    let b = h.buffers.get_mut(id).unwrap();
    let ticket = request(b, view(b));
    let end = Instant::now() + Duration::from_secs(10);
    let mut maximum = Duration::ZERO;
    let mut peak = 0;
    loop {
        assert!(Instant::now() < end);
        let start = Instant::now();
        let status = b.poll_pair_presentation(ticket, start + SLICE_TIME);
        maximum = maximum.max(start.elapsed());
        peak = peak.max(b.retained_pair_bytes());
        assert!(peak <= MAX_METADATA, "retained {peak}");
        if status == Status::Ready {
            break;
        }
        assert_eq!(status, Status::Pending);
    }
    assert_eq!(
        b.pair_presentation(ticket).unwrap().outcome,
        Outcome::Limited(Limit::Retention)
    );
    assert!(b.cancel_pair_presentation(ticket));
    assert!(b.retained_pair_bytes() < 4096);
    eprintln!(
        "qualification aggregate: peak={peak}, after_cancel={}, max poll={maximum:?}",
        b.retained_pair_bytes()
    );
}
