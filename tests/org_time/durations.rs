use rev::config::DurationUnits;
use rev::org::time::Duration;

// Work-duration forms use the configured minute scales, independently of dates.
#[test]
fn duration_forms() {
    let units = DurationUnits::default();
    for (text, minutes) in [
        ("90", 90.0),
        ("1:30", 90.0),
        ("0:00:30", 0.5),
        ("1 h 2min", 62.0),
        ("1d", 1440.0),
        ("1w", 10080.0),
        ("1m", 43200.0),
        ("1y", 525960.0),
    ] {
        assert_eq!(
            Duration::parse(text, &units).unwrap().minutes(),
            minutes,
            "{text}"
        );
    }
}

// Clock display is elapsed work with unbounded hours, not a date or a calendar
// month. Fractional minutes truncate at the requested precision, as Org does.
#[test]
fn duration_format() {
    for (minutes, clock, seconds) in [
        (0.0, "0:00", "0:00:00"),
        (0.5, "0:00", "0:00:30"),
        (1.999, "0:01", "0:01:59"),
        (90.0, "1:30", "1:30:00"),
        (1440.0, "24:00", "24:00:00"),
    ] {
        let duration = Duration::from_minutes(minutes).unwrap();
        assert_eq!(duration.format_clock(false).unwrap(), clock);
        assert_eq!(duration.format_clock(true).unwrap(), seconds);
    }
}

// Positive custom work scales affect effort totals but cannot change Gregorian
// shifts. The original duration spelling is only read, never normalized in place.
#[test]
fn independent_scales() {
    let units = DurationUnits {
        d: 480.0,
        m: 9600.0,
        ..DurationUnits::default()
    };
    let text = String::from("1m 1d 1:00");
    assert!(Duration::parse(&text, &units).is_err());
    assert_eq!(text, "1m 1d 1:00");
    let total = Duration::parse("1m 1d", &units)
        .unwrap()
        .checked_add(Duration::parse("1:00", &units).unwrap())
        .unwrap();
    assert_eq!(total.minutes(), 10140.0);
    assert_eq!(total.format_clock(false).unwrap(), "169:00");
    let date = rev::org::time::Date::new(2024, 1, 31).unwrap();
    assert_eq!(date.add_months(1).unwrap().to_string(), "2024-03-02");
}
