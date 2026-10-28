use super::*;
use rev::org::agenda::{HabitCell, HabitGraph, HabitState as S};
use rev::org::time::Date;
use rev::window::Area;
fn graph() -> HabitGraph {
    let date = Date::new(2024, 3, 1).unwrap();
    HabitGraph {
        column: 40,
        unresolved: 0,
        cells: [
            S::Clear,
            S::Ready,
            S::Alert,
            S::Overdue,
            S::Clear,
            S::Ready,
            S::Alert,
            S::Overdue,
        ]
        .into_iter()
        .enumerate()
        .map(|(i, state)| HabitCell {
            date: date.add_days(i as i64).unwrap(),
            state,
            completed: i < 2,
            today: i == 2,
            future: i >= 4,
        })
        .collect(),
    }
}
// A graph owns only its pane row: clipping keeps whole Unicode title glyphs,
// complete ASCII markers and the selected day, with peers and source unchanged.
#[test]
fn habit_graph_clipping() {
    let buffers = Buffers::new().unwrap();
    let b = buffers.get(buffers.initial()).unwrap();
    let view = View::new(b);
    for width in [1, 2, 4, 8, 12, 40, 70] {
        let mut frame = Frame::project(
            b,
            &view,
            Size {
                width: 100,
                height: 8,
            },
            "echo",
        )
        .unwrap();
        let before = frame.rows()[1].text().to_owned();
        frame
            .habit_row(
                &graph(),
                "Habit 界e\u{301} title",
                Area::new(10, 1, width, 1).unwrap(),
            )
            .unwrap();
        let after = frame.rows()[1].text();
        assert_eq!(after.width(), 100);
        assert_eq!(
            after.chars().take(10).collect::<String>(),
            before.chars().take(10).collect::<String>()
        );
        assert!(after.ends_with(&" ".repeat(90 - usize::from(width))));
        assert!(after.contains('!'), "width {width}");
        assert!(frame.cursor().is_none());
        assert!(b.history().is_empty());
    }
}
// Each of the eight semantic graph backgrounds reaches the native terminal
// writer. Completion and today glyphs remain ordinary printable source-free UI.
#[test]
fn habit_graph_palette() {
    let buffers = Buffers::new().unwrap();
    let b = buffers.get(buffers.initial()).unwrap();
    let mut frame = Frame::project(
        b,
        &View::new(b),
        Size {
            width: 80,
            height: 8,
        },
        "",
    )
    .unwrap();
    frame
        .habit_row(&graph(), "Habit", Area::new(10, 1, 60, 1).unwrap())
        .unwrap();
    let cells = surface::draw(&frame);
    for (i, rgb) in [
        rev::config::Rgb(0, 0, 255),
        rev::config::Rgb(34, 139, 34),
        rev::config::Rgb(255, 215, 0),
        rev::config::Rgb(178, 34, 34),
        rev::config::Rgb(25, 25, 112),
        rev::config::Rgb(0, 100, 0),
        rev::config::Rgb(184, 134, 11),
        rev::config::Rgb(139, 0, 0),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(cells[&(1, 50 + i)].background, Some(rgb), "role {i}");
    }
    assert!(frame.rows()[1].text().contains("**!-"));
}
