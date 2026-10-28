use rev::buffer::Buffers;
use rev::org::agenda::{HabitGraph, HabitRecord, HabitState as S};
use rev::org::time::{Date, HabitSchedule, Timestamp};

pub fn record(schedule: &str, dates: &[&str]) -> HabitRecord {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("").unwrap();
    HabitRecord {
        source: buffers.get(id).unwrap().span(0..0).unwrap(),
        schedule: HabitSchedule::new(Timestamp::parse(schedule).unwrap()).unwrap(),
        last_repeat: None,
        completions: dates
            .iter()
            .map(|d| Timestamp::parse(&format!("[{d}]")).unwrap())
            .collect(),
        unresolved: 0,
    }
}
fn at(graph: &HabitGraph, date: Date) -> &rev::org::agenda::HabitCell {
    graph.cells.iter().find(|c| c.date == date).unwrap()
}
// Due colors never manufacture completion observations. Late completions retain
// overdue state by default, with green available only through the explicit flag.
#[test]
fn habit_graph_states() {
    let today = Date::new(2024, 3, 1).unwrap();
    let policy = rev::config::Habit::default();
    let r = record("<2024-02-28 .+2d/3d>", &[]);
    let graph = r.graph(today, &policy).unwrap();
    for (name, date, state) in [
        ("before", Date::new(2024, 2, 27).unwrap(), S::Clear),
        ("available", Date::new(2024, 2, 28).unwrap(), S::Ready),
        ("imminent", Date::new(2024, 2, 29).unwrap(), S::Alert),
        ("overdue", today, S::Overdue),
    ] {
        let cell = at(&graph, date);
        assert_eq!(cell.state, state, "{name}");
        assert!(!cell.completed, "{name}");
    }
    assert!(at(&graph, Date::new(2024, 2, 28).unwrap()).subdued());
    assert!(!at(&graph, Date::new(2024, 2, 28).unwrap()).future);
    assert_eq!(graph.cells.len(), 29);
    assert_eq!(graph.column, 40);
    assert_eq!(at(&graph, today).glyph(), '!');
    let r = record("<2024-03-03 .+2d/3d>", &["2024-02-25", "2024-02-29"]);
    for (name, green, expected) in [
        ("retain late", false, S::Overdue),
        ("explicit green", true, S::Ready),
    ] {
        let mut policy = policy.clone();
        policy.done_always_green = green;
        let graph = r.graph(today, &policy).unwrap();
        let cell = at(&graph, Date::new(2024, 2, 29).unwrap());
        assert_eq!(cell.state, expected, "{name}");
        assert_eq!(cell.glyph(), '*');
    }
}
// Reconstructing historical due dates follows each repeater type and actual
// recorded events, including duplicate days, instead of inventing daily success.
#[test]
fn habit_graph_recurrence() {
    let today = Date::new(2024, 3, 7).unwrap();
    let dates = ["2024-02-25", "2024-02-26", "2024-03-03"];
    for (name, cookie, expected) in [
        ("restart", ".+2d/3d", S::Ready),
        ("catchup", "++2d/3d", S::Clear),
        ("backlog", "+2d/3d", S::Clear),
    ] {
        let r = record(&format!("<2024-03-06 {cookie}>"), &dates);
        let graph = r.graph(today, &rev::config::Habit::default()).unwrap();
        assert_eq!(
            at(&graph, Date::new(2024, 2, 28).unwrap()).state,
            expected,
            "{name}"
        );
        assert_eq!(graph.cells.iter().filter(|c| c.completed).count(), 3);
    }
}
// Both maximum visible sides are bounded; zero selects just today. A source
// date beyond the calendar range is diagnosed, never wrapped or silently clipped.
#[test]
fn habit_graph_bounds() {
    let r = record("<2024-03-01 .+1d>", &["2024-03-01", "2024-03-01"]);
    let today = Date::new(2024, 3, 1).unwrap();
    for (name, side, count) in [("zero", 0, 1), ("maximum", 366, 733)] {
        let policy = rev::config::Habit {
            preceding_days: side,
            following_days: side,
            ..Default::default()
        };
        let graph = r.graph(today, &policy).unwrap();
        assert_eq!(graph.cells.len(), count, "{name}");
        let cell = at(&graph, today);
        assert_eq!(cell.glyph(), '*');
        assert!(cell.today);
    }
    let policy = rev::config::Habit {
        preceding_days: 367,
        ..Default::default()
    };
    assert!(r.graph(today, &policy).is_err());
    assert!(
        r.graph(Date::new(1, 1, 1).unwrap(), &Default::default())
            .is_err()
    );
}
