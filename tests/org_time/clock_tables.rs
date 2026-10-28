use rev::org::{
    agenda::{ClockScope, ClockTable},
    time::{CivilDateTime, CivilTime, Date},
};
fn now() -> CivilDateTime {
    CivilDateTime {
        date: Date::new(2024, 3, 31).unwrap(),
        time: CivilTime::new(13, 0, 0).unwrap(),
    }
}
// Compatible options remain a finite data language; month/week/year boundaries
// use the captured date, while conflicting/hooks/overflow options never run code.
#[test]
fn clock_table_options() {
    for (name, source, start, end) in [
        ("default", "", "2024-01-01", "2025-01-01"),
        ("month", ":block lastmonth", "2024-02-01", "2024-03-01"),
        ("week", ":block thisweek", "2024-03-25", "2024-04-01"),
        ("day", ":block today", "2024-03-31", "2024-04-01"),
        (
            "explicit",
            ":scope subtree :tstart \"<2024-02-28 Wed 12:00>\" :tend \"<2024-03-01 Fri 11:00>\"",
            "2024-02-28",
            "2024-03-01",
        ),
    ] {
        let table = ClockTable::parse(source, now()).unwrap();
        let window = table.query.window.unwrap();
        assert_eq!(window.start().date.to_string(), start, "{name}");
        assert_eq!(window.end().date.to_string(), end, "{name}");
        if name == "explicit" {
            assert_eq!(table.query.scope, ClockScope::Subtree);
            assert_eq!(window.start().time.hour(), 12);
        }
    }
    for (name, source) in [
        ("hook", ":formatter shell"),
        ("duplicate", ":maxlevel 2 :maxlevel 1"),
        ("missing", ":scope"),
        ("unclosed", ":tstart \"<2024-01-01>"),
        ("scope", ":scope agenda"),
        ("bound", ":maxlevel 257"),
        (
            "conflict",
            ":block today :tstart 2024-01-01 :tend 2024-01-02",
        ),
        ("unpaired", ":tstart 2024-01-01"),
    ] {
        assert!(ClockTable::parse(source, now()).is_err(), "{name}");
    }
}
