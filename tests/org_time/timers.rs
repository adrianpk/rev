use rev::org::time::{Timer, TimerDuration, TimerMode};
use std::time::{Duration, Instant};
// Relative time is monotonic and on demand. Pause excludes fractional elapsed
// time correctly, reset discards old state, and no relative deadline is armed.
#[test]
fn relative_timer() {
    let base = Instant::now();
    let mut timer = Timer::default();
    assert!(timer.reading(base).unwrap().is_none());
    assert_eq!(timer.deadline(), None);
    timer.start(base).unwrap();
    assert_eq!(timer.deadline(), None);
    assert_eq!(
        timer
            .reading(base + Duration::from_secs(3661))
            .unwrap()
            .unwrap()
            .format(),
        "1:01:01"
    );
    let paused = timer
        .pause_or_continue(base + Duration::from_millis(1500))
        .unwrap();
    assert!(paused.paused);
    assert_eq!(paused.seconds, 1);
    assert_eq!(
        timer
            .reading(base + Duration::from_secs(10))
            .unwrap()
            .unwrap()
            .seconds,
        1
    );
    timer
        .pause_or_continue(base + Duration::from_secs(10))
        .unwrap();
    assert_eq!(
        timer
            .reading(base + Duration::from_millis(10750))
            .unwrap()
            .unwrap()
            .seconds,
        2
    );
    timer.start(base + Duration::from_secs(20)).unwrap();
    assert_eq!(
        timer
            .reading(base + Duration::from_secs(20))
            .unwrap()
            .unwrap()
            .seconds,
        0
    );
    timer.stop().unwrap();
    assert!(timer.pause_or_continue(base).is_err());
    assert_eq!(timer.deadline(), None);
}
// A countdown owns one exact deadline. Pause disarms it, resume retains precise
// remaining time, and expiry clears state once even after a delayed wake.
#[test]
fn countdown_lifecycle() {
    let base = Instant::now();
    let mut timer = Timer::default();
    timer
        .countdown(base, TimerDuration::parse("0:00:03").unwrap())
        .unwrap();
    assert_eq!(timer.deadline(), Some(base + Duration::from_secs(3)));
    assert!(timer.start(base).is_err());
    let reading = timer
        .pause_or_continue(base + Duration::from_millis(1250))
        .unwrap();
    assert_eq!(reading.seconds, 2);
    assert!(reading.paused);
    assert_eq!(timer.deadline(), None);
    assert!(!timer.expire(base + Duration::from_secs(100)).unwrap());
    timer
        .pause_or_continue(base + Duration::from_secs(100))
        .unwrap();
    let due = base + Duration::from_millis(101750);
    assert_eq!(timer.deadline(), Some(due));
    assert!(!timer.expire(due - Duration::from_nanos(1)).unwrap());
    assert!(timer.expire(due + Duration::from_secs(100)).unwrap());
    assert!(!timer.expire(due + Duration::from_secs(200)).unwrap());
    assert_eq!(timer.mode(), None);
    timer.start(due).unwrap();
    assert_eq!(timer.mode(), Some(TimerMode::Relative));
    assert_eq!(timer.deadline(), None);
}
// Only explicitly admitted input can arm a timer. Numeric overflow, resource
// bounds and clock-field mistakes reject without changing existing state.
#[test]
fn timer_duration_bounds() {
    for (name, source, seconds) in [
        ("default", "0", 0),
        ("minutes", "2", 120),
        ("seconds", "0:00:01", 1),
        ("day limit", "8760:00:00", 31536000),
    ] {
        assert_eq!(
            TimerDuration::parse(source).unwrap().seconds(),
            seconds,
            "{name}"
        );
    }
    for (name, source) in [
        ("negative", "-1"),
        ("fraction", "0.5"),
        ("field", "0:00:60"),
        ("short", "0:1:00"),
        ("extra", "0:00:01:00"),
        ("overflow", "18446744073709551615"),
        ("limit", "8760:00:01"),
        ("empty", ""),
    ] {
        assert!(TimerDuration::parse(source).is_err(), "{name}");
    }
    let mut timer = Timer::default();
    let now = Instant::now();
    assert!(
        timer
            .countdown(now, TimerDuration::parse("0").unwrap())
            .is_err()
    );
    assert_eq!(timer.mode(), None);
    timer.start(now).unwrap();
    let generation = timer.generation();
    assert!(
        timer
            .pause_or_continue(now - Duration::from_secs(1))
            .is_err()
    );
    assert_eq!(timer.generation(), generation);
    assert_eq!(timer.mode(), Some(TimerMode::Relative));
}
