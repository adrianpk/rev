use super::*;
use crate::org::agenda::{Calendar, CalendarLine};
pub struct AgendaDisplay<'a> {
    pub calendar: &'a Calendar,
    pub top: usize,
    pub selected: usize,
    pub active: bool,
    pub stale: bool,
    pub marked: &'a [crate::org::agenda::ActionTarget],
}
impl Frame {
    /// A calendar replaces only its owned pane content. It has no document,
    /// history or cursor authority over a selected peer pane.
    pub fn org_agenda(&mut self, display: AgendaDisplay<'_>, area: Area) -> Result<(), Error> {
        let view = display.calendar;
        let header = format!(
            "Org agenda [{}] {}..{} [{}] [read-only]{}",
            if view.complete() {
                "complete"
            } else {
                "incomplete"
            },
            view.query.start,
            view.query
                .end()
                .map_err(|_| crate::coordinates::Error::Overflow)?,
            view.query.zone,
            if display.stale {
                " [stale: g refresh]"
            } else {
                ""
            }
        );
        for row in 0..area.height() {
            let line = Area::new(area.column(), area.row() + row, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let text = if row == 0 {
                header.clone()
            } else {
                let index = display.top + usize::from(row - 1);
                view.lines.get(index).map_or_else(String::new, |kind| {
                    format!(
                        "{}{}{}",
                        if index == display.selected {
                            "> "
                        } else {
                            "  "
                        },
                        calendar_mark(&display, kind),
                        calendar_text(view, kind)
                    )
                })
            };
            self.projection_text(&text, line)?;
        }
        if display.active {
            self.cursor = None;
        }
        Ok(())
    }
}
fn calendar_text(view: &Calendar, line: &CalendarLine) -> String {
    match line {
        CalendarLine::Date(date) => format!(
            "{date} {}",
            ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][usize::from(date.weekday() - 1)]
        ),
        CalendarLine::Section(kind) => format!("{}:", kind.label()),
        CalendarLine::Grid(time) => format!("{} ................", clock(*time)),
        CalendarLine::Issue(index) => {
            let issue = &view.issues[*index];
            format!("Incomplete {:?}: {}", issue.path, issue.message)
        }
        CalendarLine::Entry(index) => {
            let row = &view.rows[*index];
            let source = &view.sources[row.source];
            let name = source
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy())
                .unwrap_or(std::borrow::Cow::Borrowed("buffer"));
            let time = row
                .time
                .map_or_else(String::new, |t| format!("{} ", clock(t)));
            let end = row
                .end_time
                .map_or_else(String::new, |t| format!("until {} ", clock(t)));
            format!(
                "{time}{end}{}: {} ({name}){}{}",
                row.kind.label(),
                row.title,
                if row.unresolved {
                    " [unresolved local time]"
                } else {
                    ""
                },
                if row.stale { " [stale]" } else { "" }
            )
        }
    }
}

fn clock(time: crate::org::time::CivilTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

fn calendar_mark(display: &AgendaDisplay<'_>, line: &CalendarLine) -> &'static str {
    let CalendarLine::Entry(index) = line else {
        return "";
    };
    let row = &display.calendar.rows[*index];
    let source = &display.calendar.sources[row.source];
    if display.marked.iter().any(|t| {
        t.byte == row.byte && t.source.version == source.version && t.source.path == source.path
    }) {
        "* "
    } else {
        ""
    }
}
