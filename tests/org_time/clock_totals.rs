use rev::org::time::{RecordedClock, Zone, ZoneEnvironment, ZoneSelection};
fn zone(name: &str) -> Zone {
    Zone::observe(
        &ZoneSelection::Local,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: Some(name.into()),
        },
    )
    .unwrap()
}
// Clock reports derive elapsed minutes from checked endpoints, never a stale
// display duration. DST gaps/folds require source resolution, not a guessed zone.
#[test]
fn recorded_intervals() {
    for (name, source, tz, expected) in [
        (
            "minute",
            "CLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 10:17] => 9:00",
            "UTC0",
            Some(17),
        ),
        (
            "spring",
            "CLOCK: [2024-03-31 Sun 01:30]--[2024-03-31 Sun 03:30] => 1:00",
            "CET-1CEST,M3.5.0,M10.5.0/3",
            Some(60),
        ),
        (
            "zero",
            "CLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 10:00] => 0:00",
            "UTC0",
            Some(0),
        ),
        ("open", "CLOCK: [2024-01-01 Mon 10:00]", "UTC0", None),
    ] {
        let actual = RecordedClock::observe(source, &zone(tz))
            .unwrap()
            .map(|v| v.minutes());
        assert_eq!(actual, expected, "{name}");
    }
}
// Invalid syntax and ambiguous civil endpoints cannot silently contribute time.
#[test]
fn unresolved_intervals() {
    let zone = zone("CET-1CEST,M3.5.0,M10.5.0/3");
    for (name, source) in [
        (
            "fold",
            "CLOCK: [2024-10-27 Sun 02:30]--[2024-10-27 Sun 03:30] => 1:00",
        ),
        (
            "gap",
            "CLOCK: [2024-03-31 Sun 02:30]--[2024-03-31 Sun 03:30] => 1:00",
        ),
        (
            "backward",
            "CLOCK: [2024-01-01 Mon 11:00]--[2024-01-01 Mon 10:00] => 1:00",
        ),
        (
            "duration",
            "CLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 11:00] => 0:99",
        ),
        ("active", "CLOCK: <2024-01-01 Mon 10:00>"),
        ("repeat", "CLOCK: [2024-01-01 Mon 10:00 +1d]"),
        (
            "trailing",
            "CLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 11:00] => 1:00 junk",
        ),
    ] {
        assert!(RecordedClock::observe(source, &zone).is_err(), "{name}");
    }
}
