use rev::org::time::{CivilDateTime, CivilInterval, CivilTime, Date};

// Independent boundary cases reject Gregorian holes and out-of-profile years.
#[test]
fn checked_dates() {
    for (name, y, m, d, valid) in [
        ("first", 1, 1, 1, true),
        ("last", 9999, 12, 31, true),
        ("zero year", 0, 1, 1, false),
        ("large year", 10000, 1, 1, false),
        ("century", 1900, 2, 29, false),
        ("four centuries", 2000, 2, 29, true),
        ("leap", 2024, 2, 29, true),
        ("ordinary", 2023, 2, 29, false),
        ("April", 2024, 4, 31, false),
        ("zero day", 2024, 1, 0, false),
        ("zero month", 2024, 0, 1, false),
        ("large month", 2024, 13, 1, false),
    ] {
        assert_eq!(Date::new(y, m, d).is_ok(), valid, "{name}");
    }
}

// Gregorian weekdays are independent of local zones; ordering is chronological.
#[test]
fn calendar_identity() {
    for (y, m, d, weekday) in [(1, 1, 1, 1), (1970, 1, 1, 4), (2000, 2, 29, 2)] {
        let date = Date::new(y, m, d).unwrap();
        assert_eq!((date.year(), date.month(), date.day()), (y, m, d));
        assert_eq!(date.weekday(), weekday);
    }
    assert!(Date::new(2024, 2, 29).unwrap() < Date::new(2024, 3, 1).unwrap());
}

// Wall-time constructors reject overflow rather than normalizing entered data.
#[test]
fn checked_times() {
    for (name, h, m, s, valid) in [
        ("midnight", 0, 0, 0, true),
        ("last second", 23, 59, 59, true),
        ("24 hours", 24, 0, 0, false),
        ("minute carry", 1, 60, 0, false),
        ("leap second", 1, 0, 60, false),
    ] {
        assert_eq!(CivilTime::new(h, m, s).is_ok(), valid, "{name}");
    }
    let t = CivilTime::new(17, 5, 12).unwrap();
    assert_eq!((t.hour(), t.minute(), t.second()), (17, 5, 12));
}

// Civil interval boundaries remain explicit, including an empty interval.
#[test]
fn interval_bounds() {
    let start = CivilDateTime {
        date: Date::new(2024, 2, 29).unwrap(),
        time: CivilTime::new(0, 0, 0).unwrap(),
    };
    let end = CivilDateTime {
        date: Date::new(2024, 3, 1).unwrap(),
        ..start
    };
    let range = CivilInterval::new(start, end).unwrap();
    assert_eq!((range.start(), range.end()), (start, end));
    assert!(range.contains(start));
    assert!(!range.contains(end));
    assert!(!CivilInterval::new(start, start).unwrap().contains(start));
    assert!(CivilInterval::new(end, start).is_err());
}
