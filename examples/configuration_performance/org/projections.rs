use super::{Result, observe, require};
use rev::{
    buffer::Buffers,
    config::Org,
    coordinates::Scalar,
    org::{
        Cancellation, Coverage, Limits, StructureScan,
        agenda::{
            AgendaQuery, CalendarQuery, HabitRecord, QueryKind, QueryResult, SourceScan,
            SourceSelection,
        },
        time::Date,
    },
};
use std::path::Path;

pub(super) fn agenda(root: &Path) -> Result<()> {
    let buffers = Buffers::new()?;
    let policy = Org::default();
    observe(
        "org_agenda",
        || {
            let selection = SourceSelection {
                paths: vec![root.join("agenda")],
                buffers: Vec::new(),
            };
            let sources =
                SourceScan::new(&buffers, selection, &policy.limits)?.run(&Cancellation::default());
            require(
                sources.complete() && sources.documents.len() == 256,
                "agenda source set incomplete",
            )?;
            let query = AgendaQuery::parse(QueryKind::Todo, "", &policy.limits)?;
            let context = CalendarQuery::new(super::documents::clock()?, &policy)?;
            let result =
                QueryResult::observe(&sources, query, context, &policy, &Cancellation::default())?;
            require(
                result.complete() && result.rows.len() == 256,
                "agenda rows incomplete",
            )?;
            require(
                result.rows.iter().all(|r| r.title.starts_with("Task ")),
                "agenda title lost",
            )?;
            Ok(result)
        },
        || "\"sources\":256,\"rows\":256".into(),
    )
}

pub(super) fn habits(root: &Path) -> Result<()> {
    let mut buffers = Buffers::new()?;
    let id = buffers.visit(&root.join("work/habit.org"))?;
    let buffer = buffers.get(id)?;
    let policy = Org::default();
    let structure = StructureScan::new(buffer, Limits::default())?.run(&Cancellation::default());
    require(
        structure.coverage() == Coverage::Complete,
        "habit structure incomplete",
    )?;
    let record = HabitRecord::observe(
        buffer,
        &structure,
        Scalar(0),
        &policy.todo,
        &Cancellation::default(),
    )?
    .ok_or("habit missing")?;
    let window = rev::config::Habit {
        preceding_days: 366,
        following_days: 366,
        ..policy.habit
    };
    let today = Date::new(2024, 3, 1)?;
    observe(
        "org_habits",
        || {
            let graph = record.graph(today, &window)?;
            require(graph.cells.len() == 733, "habit window lost days")?;
            require(
                graph.cells.iter().filter(|c| c.completed).count() == 1,
                "habit fabricated completion",
            )?;
            require(
                graph.cells.iter().filter(|c| c.today).count() == 1,
                "habit today lost",
            )?;
            Ok(graph)
        },
        || "\"days\":733,\"completions\":1".into(),
    )
}
