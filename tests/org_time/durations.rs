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
