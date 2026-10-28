use rev::org::time::OperationTime;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn instant(seconds: i64) -> SystemTime {
    let duration = Duration::from_secs(seconds.unsigned_abs());
    if seconds < 0 {
        UNIX_EPOCH - duration
    } else {
        UNIX_EPOCH + duration
    }
}

// One clock call captures the entire operation, even when subsequent projections
// cross a date boundary. Clock failure is returned without retry or fabrication.
#[test]
fn captured_clock() {
    let mut calls = 0;
    let captured = OperationTime::capture(|| {
        calls += 1;
        Ok(instant(1793207100))
    })
    .unwrap();
    assert_eq!(captured.unix_seconds(), 1793207100);
    assert_eq!(
        captured.civil_at_offset(0).unwrap().format_timestamp(false),
        "[2026-10-28 Wed 17:05]"
    );
    assert_eq!(
        captured
            .civil_at_offset(28800)
            .unwrap()
            .format_timestamp(false),
        "[2026-10-29 Thu 01:05]"
    );
    assert_eq!(calls, 1);
    let failure = OperationTime::capture(|| {
        calls += 1;
        Err("owned clock unavailable".into())
    });
    assert_eq!(failure.unwrap_err(), "owned clock unavailable");
    assert_eq!(calls, 2);
}

// Negative fractions floor toward the preceding Unix second; neither truncation
// toward zero nor unsigned epoch arithmetic can silently move the civil date.
#[test]
fn epoch_precision() {
    for (name, time, seconds, stamp) in [
        ("epoch", UNIX_EPOCH, 0, "[1970-01-01 Thu 00:00]"),
        (
            "after fraction",
            UNIX_EPOCH + Duration::from_millis(999),
            0,
            "[1970-01-01 Thu 00:00]",
        ),
        (
            "before fraction",
            UNIX_EPOCH - Duration::from_nanos(1),
            -1,
            "[1969-12-31 Wed 23:59]",
        ),
        (
            "negative exact",
            instant(-60),
            -60,
            "[1969-12-31 Wed 23:59]",
        ),
        (
            "negative fraction",
            instant(-60) - Duration::from_nanos(1),
            -61,
            "[1969-12-31 Wed 23:58]",
        ),
    ] {
        let captured = OperationTime::capture(|| Ok(time)).unwrap();
        assert_eq!(captured.unix_seconds(), seconds, "{name}");
        assert_eq!(
            captured.civil_at_offset(0).unwrap().format_timestamp(false),
            stamp,
            "{name}"
        );
    }
}

// These are explicit known offsets, not a zone/DST resolver. Elapsed subtraction
// uses captured instants despite a two-hour civil jump, with zero retained.
#[test]
fn elapsed_instants() {
    let start = OperationTime::capture(|| Ok(instant(1711845000))).unwrap();
    let end = OperationTime::capture(|| Ok(instant(1711848600))).unwrap();
    assert_eq!(
        start.civil_at_offset(3600).unwrap().format_timestamp(false),
        "[2024-03-31 Sun 01:30]"
    );
    assert_eq!(
        end.civil_at_offset(7200).unwrap().format_timestamp(false),
        "[2024-03-31 Sun 03:30]"
    );
    assert_eq!(end.elapsed_since(start).unwrap().minutes(), 60.0);
    assert_eq!(start.elapsed_since(start).unwrap().minutes(), 0.0);
    assert!(start.elapsed_since(end).is_err());
    let seconds = OperationTime::capture(|| Ok(instant(1711845030))).unwrap();
    assert_eq!(
        seconds
            .elapsed_since(start)
            .unwrap()
            .format_clock(true)
            .unwrap(),
        "0:00:30"
    );
}

// Civil profile limits and known-offset limits are enforced before formatting;
// an out-of-range projection cannot silently wrap into an admitted year.
#[test]
fn operation_bounds() {
    for (name, seconds, offset, valid) in [
        ("first", -62135596800, 0, true),
        ("last", 253402300799, 0, true),
        ("before", -62135596801, 0, false),
        ("after", 253402300800, 0, false),
        ("offset before", -62135596800, -1, false),
        ("offset after", 253402300799, 1, false),
        ("largest offset", 0, 86400, true),
        ("least offset", 0, -86400, true),
        ("large offset", 0, 86401, false),
        ("small offset", 0, -86401, false),
        ("integer offset", 0, i32::MAX, false),
    ] {
        let captured = OperationTime::capture(|| Ok(instant(seconds))).unwrap();
        assert_eq!(captured.civil_at_offset(offset).is_ok(), valid, "{name}");
    }
}
