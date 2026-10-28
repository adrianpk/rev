use rev::org::time::{CalendarInterval as Shift, CivilDateTime, CivilTime, Date};

// Integer extremes must fail checked arithmetic in every calendar unit; ordinary
// shifts beyond the source year profile fail before publishing any result.
#[test]
fn shift_overflow() {
    let value = CivilDateTime {
        date: Date::new(2024, 2, 29).unwrap(),
        time: CivilTime::new(12, 0, 0).unwrap(),
    };
    for n in [i64::MIN, i64::MAX] {
        for shift in [
            Shift::Minutes(n),
            Shift::Hours(n),
            Shift::Days(n),
            Shift::Weeks(n),
            Shift::Months(n),
            Shift::Years(n),
        ] {
            assert!(value.shift(shift).is_err(), "{shift:?}");
        }
    }
    assert!(Date::new(1, 1, 1).unwrap().add_days(-1).is_err());
    assert!(Date::new(9999, 12, 31).unwrap().add_days(1).is_err());
    assert!(Date::new(1, 1, 1).unwrap().add_months(-1).is_err());
    assert!(Date::new(9999, 12, 31).unwrap().add_months(1).is_err());
    assert!(Date::new(9999, 1, 1).unwrap().add_years(1).is_err());
    assert_eq!(value.date.to_string(), "2024-02-29");
}

// A complete 400-year Gregorian cycle checks the independent civil constructor
// against serial-day conversion, weekday continuity and checked inverse shifts.
// The fixed 146097-day bound completes without wall-clock waits or host state.
#[test]
fn gregorian_cycle() {
    let start = Date::new(2000, 3, 1).unwrap();
    for days in 0..146097 {
        let date = start.add_days(days).unwrap();
        assert_eq!(
            Date::new(date.year(), date.month(), date.day()).unwrap(),
            date
        );
        assert_eq!(date.weekday(), ((2 + days) % 7 + 1) as u8);
        assert_eq!(date.add_days(-days).unwrap(), start);
    }
    assert_eq!(
        start.add_days(146097).unwrap(),
        Date::new(2400, 3, 1).unwrap()
    );
}
