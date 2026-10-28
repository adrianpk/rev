use rev::org::time::{
    CivilDateTime, CivilTime, Date, RepeatMode as Mode, RepeatUnit as Unit, Timestamp,
    TimestampRange,
};

fn time(date: (u16, u8, u8), clock: (u8, u8)) -> CivilDateTime {
    CivilDateTime {
        date: Date::new(date.0, date.1, date.2).unwrap(),
        time: CivilTime::new(clock.0, clock.1, 0).unwrap(),
    }
}
fn next(source: &str, now: CivilDateTime) -> String {
    Timestamp::parse(source)
        .unwrap()
        .advance_repeat(now)
        .unwrap()
        .unwrap()
        .format()
}

// Typed suffixes expose distinct recurrence and delay semantics while source
// spelling remains exact; interval maxima retain their own calendar unit.
#[test]
fn repeater_source_values() {
    for (name, source, mode, count, unit) in [
        ("once", "<2024-02-29 木 +1m>", Mode::Once, 1, Unit::Month),
        (
            "catch up",
            "<2024-02-29 Thu ++2w -3d>",
            Mode::CatchUp,
            2,
            Unit::Week,
        ),
        (
            "restart",
            "<2024-02-29  Thu  .+2d/3d --1d>",
            Mode::Restart,
            2,
            Unit::Day,
        ),
    ] {
        let stamp = Timestamp::parse(source).unwrap();
        let repeat = stamp.repeater().unwrap().unwrap();
        assert_eq!(
            (repeat.mode, repeat.interval.count(), repeat.interval.unit()),
            (mode, count, unit),
            "{name}"
        );
        assert_eq!(stamp.format(), source, "{name}");
    }
    let stamp = Timestamp::parse("<2024-02-29 Thu .+2d/3d --1d>").unwrap();
    assert_eq!(
        stamp.repeater().unwrap().unwrap().maximum.unwrap().count(),
        3
    );
    assert!(stamp.warning_delay().unwrap().unwrap().first_only);
}

// Invalid counts, repeated fields, unsupported delays and non-ASCII units fail
// before any date arithmetic, rather than becoming another valid suffix.
#[test]
fn repeater_source_errors() {
    for (name, suffix) in [
        ("zero", "+0d"),
        ("overflow", "+4294967296d"),
        ("Unicode", "+1界"),
        ("duplicate", "+1d ++2d"),
        ("warnings", "-1d --2d"),
        ("extra", "+1d -2d -3d"),
        ("missing maximum", ".+2d/"),
        ("warning range", "-1d/2d"),
        ("unknown", ".+2s"),
    ] {
        assert!(
            Timestamp::parse(&format!("<2024-02-29 Thu {suffix}>")).is_err(),
            "{name}"
        );
    }
}

// Once preserves backlog, catch-up skips whole occurrences, and restart uses
// completion. Timed catch-up may retain a later clock on today's date.
#[test]
fn repeater_modes() {
    let now = time((2024, 2, 10), (10, 0));
    for (name, source, expected) in [
        ("backlog", "<2024-02-01 Thu +1d>", "<2024-02-02 Fri +1d>"),
        ("catch up", "<2024-02-01 Thu ++1d>", "<2024-02-11 Sun ++1d>"),
        ("restart", "<2024-02-01 Thu .+1d>", "<2024-02-11 Sun .+1d>"),
        (
            "future once",
            "<2024-03-01 Fri +1d>",
            "<2024-03-02 Sat +1d>",
        ),
        (
            "future catch up",
            "<2024-03-01 Fri ++1d>",
            "<2024-03-02 Sat ++1d>",
        ),
        (
            "future restart",
            "<2024-03-01 Fri .+1d>",
            "<2024-02-11 Sun .+1d>",
        ),
        (
            "today clock",
            "<2024-02-01 Thu 20:00 ++1d>",
            "<2024-02-10 Sat 20:00 ++1d>",
        ),
        (
            "equal clock",
            "<2024-02-01 Thu 10:00 ++1d>",
            "<2024-02-11 Sun 10:00 ++1d>",
        ),
    ] {
        assert_eq!(next(source, now), expected, "{name}");
    }
}

// Calendar repeat intervals retain Org excess-day normalization. Catch-up
// carries a normalized anchor forward instead of silently restoring month-end.
#[test]
fn repeater_calendar() {
    for (name, source, now, expected) in [
        (
            "leap month",
            "<2024-01-31 Wed +1m>",
            time((2024, 2, 29), (0, 0)),
            "<2024-03-02 Sat +1m>",
        ),
        (
            "normalized catch up",
            "<2024-01-31 Wed ++1m>",
            time((2024, 4, 1), (0, 0)),
            "<2024-04-02 Tue ++1m>",
        ),
        (
            "restart month",
            "<2024-01-01 Mon 9:05 .+1m>",
            time((2024, 1, 31), (14, 30)),
            "<2024-03-02 Sat 9:05 .+1m>",
        ),
        (
            "leap year",
            "<2024-02-29 Thu +1y>",
            time((2024, 3, 1), (0, 0)),
            "<2025-03-01 Sat +1y>",
        ),
        (
            "year anchor",
            "<2024-02-29 Thu ++1y>",
            time((2028, 2, 28), (0, 0)),
            "<2028-03-01 Wed ++1y>",
        ),
        (
            "week rollover",
            "<2023-12-28 Thu +1w>",
            time((2023, 12, 29), (0, 0)),
            "<2024-01-04 Thu +1w>",
        ),
    ] {
        assert_eq!(next(source, now), expected, "{name}");
    }
}

// Hourly recurrence requires a clock and preserves range length across midnight.
// Restart uses the captured completion minute, while larger units retain time.
#[test]
fn repeater_clocks() {
    let now = time((2024, 2, 29), (14, 30));
    for (name, source, expected) in [
        ("overnight", "[2024-02-29 Thu 23:45-0:15 +1h]", None),
        (
            "active overnight",
            "<2024-02-29 Thu 23:45-0:15 +1h>",
            Some("<2024-03-01 Fri 00:45-01:15 +1h>"),
        ),
        (
            "hour restart",
            "<2024-02-28 Wed 9:45-10:15 .+1h>",
            Some("<2024-02-29 Thu 15:30-16:00 .+1h>"),
        ),
        (
            "hour catch up",
            "<2024-02-28 Wed 9:45 ++2h>",
            Some("<2024-02-29 Thu 15:45 ++2h>"),
        ),
    ] {
        let stamp = Timestamp::parse(source).unwrap();
        let result = stamp.advance_repeat(now).unwrap().map(|s| s.format());
        assert_eq!(result.as_deref(), expected, "{name}");
        assert_eq!(stamp.format(), source);
    }
}

// An ordinary warning/delay applies to every occurrence. The '--' scheduled
// first-only delay is removed when repeating; remaining source spacing survives.
#[test]
fn repeater_warning_delays() {
    let now = time((2024, 3, 1), (12, 0));
    for (name, source, expected) in [
        (
            "retained",
            "<2024-03-01 Fri +1d -2d>",
            "<2024-03-02 Sat +1d -2d>",
        ),
        (
            "first only",
            "<2024-03-01 Fri +1d --2d>",
            "<2024-03-02 Sat +1d>",
        ),
        (
            "first suffix",
            "<2024-03-01 Fri --2d +1d>",
            "<2024-03-02 Sat +1d>",
        ),
    ] {
        assert_eq!(next(source, now), expected, "{name}");
    }
    let stamp = Timestamp::parse("<2024-03-01 Fri -2d>").unwrap();
    assert_eq!(
        stamp.deadline_warning().unwrap(),
        Some(time((2024, 2, 28), (0, 0)))
    );
    assert_eq!(
        stamp.scheduled_display().unwrap(),
        Some(time((2024, 3, 3), (0, 0)))
    );
    assert_eq!(stamp.advance_repeat(now).unwrap(), None);
}

// Habit bounds use calendar arithmetic rather than work-duration approximations;
// an inverted maximum is rejected without normalizing or deleting source text.
#[test]
fn repeater_habit_ranges() {
    for (name, source, now, first, last) in [
        (
            "days",
            "<2024-02-28 Wed .+2d/3d>",
            time((2024, 2, 28), (0, 0)),
            (2024, 3, 1),
            (2024, 3, 2),
        ),
        (
            "months",
            "<2024-01-31 Wed .+1m/2m>",
            time((2024, 1, 31), (0, 0)),
            (2024, 3, 2),
            (2024, 3, 31),
        ),
    ] {
        let value = Timestamp::parse(source)
            .unwrap()
            .repeater()
            .unwrap()
            .unwrap();
        assert_eq!(
            value.habit_window(now).unwrap(),
            (time(first, (0, 0)), time(last, (0, 0))),
            "{name}"
        );
    }
    let source = "<2024-02-28 Wed .+2d/1d>";
    let stamp = Timestamp::parse(source).unwrap();
    assert!(stamp.advance_repeat(time((2024, 2, 28), (0, 0))).is_err());
    assert_eq!(stamp.format(), source);
}

// Connected range endpoints repeat independently and retain unmarked bounds.
// An inverted result rejects the complete edit instead of moving an unmarked end.
#[test]
fn repeater_date_ranges() {
    let source = "<2024-02-28 Wed +1d>--<2024-03-01 Fri>";
    let value = TimestampRange::parse(source).unwrap();
    assert_eq!(
        value
            .advance_repeat(time((2024, 3, 1), (0, 0)))
            .unwrap()
            .unwrap()
            .format(),
        "<2024-02-29 Thu +1d>--<2024-03-01 Fri>"
    );
    let source = "<2024-02-28 Wed +1w>--<2024-03-01 Fri +1d>";
    let value = TimestampRange::parse(source).unwrap();
    assert!(value.advance_repeat(time((2024, 3, 1), (0, 0))).is_err());
    assert_eq!(value.format(), source);
}

// Very old hourly/day/month backlogs remain bounded and do not change source.
// Missing clocks and source-year overflow reject the complete candidate.
#[test]
fn repeater_bounds() {
    let now = time((9998, 1, 1), (0, 0));
    assert_eq!(
        next("<0001-01-01 Mon 00:00 ++1h>", now),
        "<9998-01-01 Thu 01:00 ++1h>"
    );
    assert_eq!(next("<0001-01-31 Wed ++1m>", now), "<9998-01-03 Sat ++1m>");
    for (name, source) in [
        ("no clock", "<2024-02-29 Thu +1h>"),
        ("year limit", "<9999-12-31 Fri +1d>"),
        ("large count", "<2024-02-29 Thu ++4294967295y>"),
    ] {
        let stamp = Timestamp::parse(source).unwrap();
        assert!(stamp.advance_repeat(now).is_err(), "{name}");
        assert_eq!(stamp.format(), source, "{name}");
    }
}
