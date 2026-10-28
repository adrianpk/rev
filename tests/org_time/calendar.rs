use rev::org::time::{CalendarInterval as Shift, CivilDateTime, CivilTime, Date};

fn at(date: (u16, u8, u8), time: (u8, u8, u8)) -> CivilDateTime {
    CivilDateTime {
        date: Date::new(date.0, date.1, date.2).unwrap(),
        time: CivilTime::new(time.0, time.1, time.2).unwrap(),
    }
}

// Expected month/year normalization was observed in an owned Org reference
// process. It is deliberately different from a fixed 30-day/365.25-day duration.
#[test]
fn calendar_shifts() {
    for (name, start, shift, expected) in [
        ("leap month", (2024, 1, 31), Shift::Months(1), (2024, 3, 2)),
        (
            "ordinary month",
            (2023, 1, 31),
            Shift::Months(1),
            (2023, 3, 3),
        ),
        ("back month", (2023, 3, 31), Shift::Months(-1), (2023, 3, 3)),
        ("leap year", (2024, 2, 29), Shift::Years(1), (2025, 3, 1)),
        ("December", (2024, 12, 1), Shift::Months(1), (2025, 1, 1)),
        ("day", (2024, 2, 28), Shift::Days(1), (2024, 2, 29)),
        ("week", (2024, 2, 28), Shift::Weeks(1), (2024, 3, 6)),
    ] {
        let actual = at(start, (17, 5, 12)).shift(shift).unwrap();
        assert_eq!(actual, at(expected, (17, 5, 12)), "{name}");
    }
}

// Civil minutes/hours cross dates without consulting duration preferences or TZ.
#[test]
fn subday_shifts() {
    for (name, start, shift, end) in [
        (
            "minute rollover",
            at((2024, 2, 29), (23, 59, 30)),
            Shift::Minutes(1),
            at((2024, 3, 1), (0, 0, 30)),
        ),
        (
            "negative hour",
            at((1970, 1, 1), (0, 0, 0)),
            Shift::Hours(-1),
            at((1969, 12, 31), (23, 0, 0)),
        ),
        (
            "zero",
            at((1, 1, 1), (0, 0, 0)),
            Shift::Days(0),
            at((1, 1, 1), (0, 0, 0)),
        ),
    ] {
        assert_eq!(start.shift(shift).unwrap(), end, "{name}");
    }
}

// Timestamp formatting derives English weekdays from checked dates and keeps
// active/inactive delimiters explicit. No existing buffer source is normalized.
#[test]
fn timestamp_format() {
    for (value, expected) in [
        (at((1970, 1, 1), (0, 0, 0)), "[1970-01-01 Thu 00:00]"),
        (at((1969, 12, 31), (23, 59, 59)), "[1969-12-31 Wed 23:59]"),
        (at((2000, 2, 29), (0, 0, 0)), "[2000-02-29 Tue 00:00]"),
        (at((2026, 10, 28), (17, 5, 0)), "[2026-10-28 Wed 17:05]"),
    ] {
        assert_eq!(value.format_timestamp(false), expected);
    }
    assert_eq!(
        at((1, 1, 1), (0, 0, 0)).format_timestamp(true),
        "<0001-01-01 Mon 00:00>"
    );
}
