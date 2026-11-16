use std::time::Instant;

use crate::qualification::{Harness, complete, request, view};
use rev::pairs::{Limit, PresentationOutcome as Outcome, Status};

// The frozen stress forms retain the same shared scanner/export budgets when
// consumed by presentation. Measurements use the existing bounded poll helper.
#[test]
fn frozen_resources() {
    let workloads = [
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
            "dense5000",
            "()".repeat(5000),
            Outcome::Limited(Limit::Retention),
        ),
        (
            "token262145",
            "x".repeat(262145),
            Outcome::Limited(Limit::Bytes),
        ),
        (
            "offscreen",
            format!("({})", "word ".repeat(1000)),
            Outcome::Complete,
        ),
    ];
    for (name, source, expected) in workloads {
        measure(name, &source, expected);
    }
}

fn measure(name: &str, source: &str, expected: Outcome) {
    let mut h = Harness::new("plain", false);
    let id = h.source(source);
    let b = h.buffers.get_mut(id).unwrap();
    let owner = view(b);
    let ticket = request(b, owner);
    let maximum = complete(b, ticket);
    let result = b.pair_presentation(ticket).unwrap();
    assert_eq!(result.outcome, expected, "{name}");
    assert!(result.spans.len() <= 8192);
    assert!(result.stats.inspected_bytes <= 262144);
    assert!(result.stats.maximum_depth <= 256);
    assert!(result.stats.maximum_slice_tokens <= 256);
    assert!(result.stats.retained_bytes <= 2097152);
    assert!(
        result
            .spans
            .iter()
            .all(|s| s.depth < 256 && s.at.0 < b.len_chars() && s.mate.0 < b.len_chars())
    );
    eprintln!(
        "CONTINUITY_RESOURCE {name} bytes={} spans={} inspected={} retained={} slices={} max_tokens={} max_poll_us={}",
        source.len(),
        result.spans.len(),
        result.stats.inspected_bytes,
        result.stats.retained_bytes,
        result.stats.slices,
        result.stats.maximum_slice_tokens,
        maximum.as_micros()
    );
    assert!(b.cancel_pair_presentation(ticket));
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now()),
        Status::Retired
    );
    assert!(b.retained_pair_bytes() < 4096);
    assert!(b.history().is_empty() && !b.is_modified());
}

// Expired polls do no work, and canceling a pending dense export cannot let a
// later poll publish it. This is the same native ticket used by source views.
#[test]
fn canceled_work() {
    let mut h = Harness::new("plain", false);
    let id = h.source(&"()".repeat(5000));
    let b = h.buffers.get_mut(id).unwrap();
    let ticket = request(b, view(b));
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now()),
        Status::Pending
    );
    let bytes = b.retained_pair_bytes();
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now()),
        Status::Pending
    );
    assert_eq!(b.retained_pair_bytes(), bytes);
    assert!(b.cancel_pair_presentation(ticket));
    assert_eq!(
        b.poll_pair_presentation(ticket, Instant::now() + rev::pairs::SLICE_TIME),
        Status::Retired
    );
    assert!(b.pair_presentation(ticket).is_none());
    assert!(b.retained_pair_bytes() < 4096);
}
