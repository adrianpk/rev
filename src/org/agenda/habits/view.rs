use super::*;
use crate::org::Cancellation;

#[derive(Clone, Debug)]
pub struct HabitRow {
    pub title: String,
    pub record: HabitRecord,
    pub graph: HabitGraph,
}
#[derive(Clone, Debug)]
pub struct HabitAgenda {
    pub source: Span,
    pub today: Date,
    pub rows: Vec<HabitRow>,
}
pub(crate) struct HabitScan {
    pub(crate) source: Span,
    pub(crate) today: Date,
    pub(crate) policy: crate::config::Habit,
    pub(crate) rows: Vec<(String, HabitRecord)>,
}
impl HabitScan {
    pub(crate) fn run(self, cancel: &Cancellation) -> Result<HabitAgenda, String> {
        let mut rows = Vec::with_capacity(self.rows.len());
        for (title, record) in self.rows {
            if cancel.is_cancelled() {
                return Err("Org habit graph cancelled".into());
            }
            let graph = record.graph(self.today, &self.policy)?;
            rows.push(HabitRow {
                title,
                record,
                graph,
            });
        }
        Ok(HabitAgenda {
            source: self.source,
            today: self.today,
            rows,
        })
    }
}
impl HabitAgenda {
    pub fn visible<'a>(
        &'a self,
        date: Date,
        policy: &'a crate::config::Habit,
        shown: bool,
    ) -> impl Iterator<Item = &'a HabitRow> {
        let admitted = shown && (!policy.today_only || date == self.today);
        self.rows.iter().filter(move |row| {
            admitted
                && (date == self.today && policy.show_all_today
                    || row.record.schedule.scheduled.date() <= date)
        })
    }
    pub(crate) fn redraw(&mut self, policy: &crate::config::Habit) -> Result<(), String> {
        let graphs = self
            .rows
            .iter()
            .map(|row| row.record.graph(self.today, policy))
            .collect::<Result<Vec<_>, _>>()?;
        for (row, graph) in self.rows.iter_mut().zip(graphs) {
            row.graph = graph
        }
        Ok(())
    }
}
