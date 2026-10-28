use rev::buffer::{Buffers, Mode};
use rev::coordinates::Scalar;
use rev::org::agenda::{HabitObservation as O, HabitRecord};
use rev::org::time::Date;
use rev::org::{Cancellation, Limits, StructureScan};

fn observe(source: &str) -> Result<Option<HabitRecord>, String> {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    let structure = StructureScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    HabitRecord::observe(
        buffer,
        &structure,
        Scalar(source[..source.find("* TODO").unwrap_or(0)].chars().count()),
        &rev::config::Todo::default(),
        &Cancellation::default(),
    )
}
const HEADER: &str = "* TODO Habit\nSCHEDULED: <2024-03-01 Fri .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:LAST_REPEAT: [2024-02-29 Thu 23:50]\n:END:\n";

// Plain and drawer logs retain source order and repeated real events. Metadata
// alone is not history; child/source examples and nonfinished states contribute
// nothing. Malformed actual DONE records remain explicitly unresolved.
#[test]
fn habit_history_sources() {
    let source = format!(
        "{HEADER}- State \"DONE\" from \"TODO\" [2024-02-27 Tue]\n:LOGBOOK:\n- State \"DONE\"   from \"TODO\"   [2024-02-28 Wed 00:05] \\\\\n  note\n- State \"DONE\" from \"TODO\" [2024-02-28 Wed 13:00]\n- State \"WAIT\" from \"TODO\" [2024-02-29 Thu]\n- State \"DONE\" from \"TODO\" [2024-02-30 Fri]\n:END:\n#+begin_src text\n- State \"DONE\" from \"TODO\" [2024-02-26 Mon]\n#+end_src\n** TODO Child\n- State \"DONE\" from \"TODO\" [2024-02-25 Sun]\n"
    );
    let record = observe(&source).unwrap().unwrap();
    assert_eq!(record.completions.len(), 3);
    assert_eq!(record.unresolved, 1);
    assert_eq!(
        record.last_repeat.as_ref().unwrap().date().to_string(),
        "2024-02-29"
    );
    assert_eq!(
        record.observation(Date::new(2024, 2, 28).unwrap()),
        O::Completed(2)
    );
    for day in [25, 26, 29] {
        assert_eq!(
            record.observation(Date::new(2024, 2, day).unwrap()),
            O::Unrecorded
        );
    }
    let empty = observe(HEADER).unwrap().unwrap();
    assert!(empty.completions.is_empty());
    assert_eq!(
        empty.observation(Date::new(2024, 2, 29).unwrap()),
        O::Unrecorded
    );
}

// File-local workflows determine finished history keywords. An explicit closing
// note is completion evidence too, while malformed metadata never manufactures it.
#[test]
fn habit_local_history() {
    let source = HEADER
        .replacen(
            "* TODO Habit",
            "#+SEQ_TODO: TODO | FINISHED\n* TODO Habit",
            1,
        )
        .replace(
            ":LAST_REPEAT: [2024-02-29 Thu 23:50]",
            ":LAST_REPEAT: invalid",
        );
    let source = format!(
        "{source}- State \"FINISHED\" from \"TODO\" [2024-02-28 Wed]\n- State \"DONE\" from \"TODO\" [2024-02-27 Tue]\n- CLOSING NOTE [2024-02-29 Thu 01:00]\n"
    );
    let r = observe(&source).unwrap().unwrap();
    assert_eq!(r.completions.len(), 2);
    assert_eq!(r.unresolved, 1);
    assert!(r.last_repeat.is_none());
}

// The completion count is bounded independently of the selected entry bytes;
// one extra actual record rejects the complete derived observation.
#[test]
fn habit_history_bounds() {
    for (name, count, admit) in [("limit", 4096, true), ("over", 4097, false)] {
        let source = format!(
            "{HEADER}{}",
            "- State \"DONE\" from \"TODO\" [2024-02-28 Wed]\n".repeat(count)
        );
        let r = observe(&source);
        assert_eq!(r.is_ok(), admit, "{name}: {r:?}");
        if admit {
            assert_eq!(r.unwrap().unwrap().completions.len(), count)
        }
    }
}
