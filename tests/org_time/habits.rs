use rev::org::time::{CivilDateTime, CivilTime, Date, HabitSchedule, OrgDay, Timestamp};

// Habit ranges preserve calendar units rather than averaging months or years;
// only real active daily-or-longer repeats qualify as a habit schedule.
#[test]
fn habit_schedules() {
    let completed = CivilDateTime {
        date: Date::new(2024, 1, 31).unwrap(),
        time: CivilTime::new(2, 30, 0).unwrap(),
    };
    for (name, source, expected) in [
        (
            "restart",
            "<2024-01-31 .+2d/3d>",
            Some(("2024-02-02", "2024-02-03")),
        ),
        (
            "backlog",
            "<2024-01-31 +1w>",
            Some(("2024-02-07", "2024-02-07")),
        ),
        (
            "catchup",
            "<2024-01-31 ++1m/2m>",
            Some(("2024-03-02", "2024-03-31")),
        ),
        (
            "leap",
            "<2024-01-31 .+1y/2y>",
            Some(("2025-01-31", "2026-01-31")),
        ),
        ("inactive", "[2024-01-31 .+1d]", None),
        ("no repeat", "<2024-01-31>", None),
        ("hour", "<2024-01-31 02:30 .+24h>", None),
        ("same bound", "<2024-01-31 .+2d/2d>", None),
        ("inverted", "<2024-01-31 .+3d/2d>", None),
    ] {
        let result = HabitSchedule::new(Timestamp::parse(source).unwrap());
        if let Some((first, last)) = expected {
            let (a, b) = result.unwrap().bounds_after(completed).unwrap();
            assert_eq!(
                (a.to_string(), b.to_string()),
                (first.into(), last.into()),
                "{name}"
            );
        } else {
            assert!(result.is_err(), "{name}");
        }
    }
}

// Midnight remains the default. A selected rollover changes the Org date only;
// callers retain the real timestamp for completion logging and clock intervals.
#[test]
fn habit_day_boundary() {
    let date = Date::new(2024, 3, 1).unwrap();
    for (name, hour, minute, expected) in [
        ("before", 3, 59, "2024-02-29"),
        ("at", 4, 0, "2024-03-01"),
        ("after", 23, 59, "2024-03-01"),
    ] {
        let now = CivilDateTime {
            date,
            time: CivilTime::new(hour, minute, 0).unwrap(),
        };
        assert_eq!(
            OrgDay::new(4).unwrap().date_at(now).unwrap().to_string(),
            expected,
            "{name}"
        );
        assert_eq!(OrgDay::default().date_at(now).unwrap(), date);
    }
    assert!(OrgDay::new(24).is_err());
    let first = CivilDateTime {
        date: Date::new(1, 1, 1).unwrap(),
        time: CivilTime::new(0, 0, 0).unwrap(),
    };
    assert!(OrgDay::new(1).unwrap().date_at(first).is_err());
}
