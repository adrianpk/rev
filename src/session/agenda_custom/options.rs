use super::*;
use crate::config::{AgendaSort, AgendaSpan};
pub(super) fn apply(
    view: &mut AgendaView,
    baseline: &AgendaView,
    command: C,
    draft: &str,
) -> Result<(), String> {
    for (block, base) in view.blocks.iter_mut().zip(&baseline.blocks) {
        match command {
            C::AgendaFilter | C::AgendaFilterReset => {
                block.filter = filter(draft, &base.filter);
            }
            C::AgendaSort => {
                block.sort = list(draft)
                    .into_iter()
                    .map(sort)
                    .collect::<Result<_, _>>()?
            }
            C::AgendaColumns => {
                block.columns = list(draft)
                    .into_iter()
                    .map(str::to_ascii_uppercase)
                    .collect()
            }
            C::AgendaSpan => block.span = Some(span(draft)?),
            _ => return Err("Unavailable agenda presentation control".into()),
        }
        block.validate()?;
    }
    Ok(())
}
fn list(source: &str) -> Vec<&str> {
    source
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}
pub(super) fn span(source: &str) -> Result<AgendaSpan, String> {
    match source.trim() {
        "day" => Ok(AgendaSpan::Day),
        "week" => Ok(AgendaSpan::Week),
        "month" => Ok(AgendaSpan::Month),
        "year" => Ok(AgendaSpan::Year),
        value => value
            .parse::<u16>()
            .ok()
            .filter(|n| (1..=366).contains(n))
            .map(AgendaSpan::Days)
            .ok_or("Agenda span requires day/week/month/year or 1..366".into()),
    }
}
fn sort(value: &str) -> Result<AgendaSort, String> {
    match value {
        "title-up" => Ok(AgendaSort::TitleUp),
        "title-down" => Ok(AgendaSort::TitleDown),
        "todo-up" => Ok(AgendaSort::TodoUp),
        "priority-down" => Ok(AgendaSort::PriorityDown),
        "effort-up" => Ok(AgendaSort::EffortUp),
        "effort-down" => Ok(AgendaSort::EffortDown),
        "date-up" => Ok(AgendaSort::DateUp),
        "clock-down" => Ok(AgendaSort::ClockDown),
        "source-up" => Ok(AgendaSort::SourceUp),
        _ => Err("Unknown native agenda sort key".into()),
    }
}

pub(super) fn context(
    old: CalendarQuery,
    command: C,
    draft: &str,
) -> Result<CalendarQuery, String> {
    match command {
        C::AgendaSpan => old.at(old.date, span(draft)?),
        C::AgendaNext => old.shifted(1),
        C::AgendaPrevious => old.shifted(-1),
        C::AgendaToday => old.at(old.today, old.span),
        C::AgendaDate => old.at(
            if draft.trim() == "today" {
                old.today
            } else {
                CalendarQuery::parse_date(draft)?
            },
            old.span,
        ),
        _ => Ok(old),
    }
}

fn filter(draft: &str, base: &str) -> String {
    if draft.trim().is_empty() {
        base.into()
    } else if base.is_empty() {
        draft.into()
    } else {
        format!("({base})&({draft})")
    }
}
