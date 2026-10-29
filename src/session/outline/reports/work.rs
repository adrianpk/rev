use crate::{
    buffer::edit::Replacement,
    org::{
        Cancellation,
        agenda::{ClockScan, ClockUpdate},
        time::Zone,
    },
};
pub(in crate::session) enum Work {
    Display(ClockScan),
    Columns(Box<crate::org::columns::Scan>),
    Update(ClockUpdate),
    Habits(crate::org::agenda::HabitScan),
}
pub(in crate::session) enum Output {
    Display(String),
    Columns(crate::org::columns::Output),
    Update(Replacement),
    Habits(crate::org::agenda::HabitAgenda),
}
impl Work {
    pub(in crate::session) fn run(
        self,
        zone: &Zone,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        match self {
            Self::Columns(scan) => scan.run(zone, cancel).map(Output::Columns),
            Self::Display(scan) => scan.run(zone, cancel)?.feedback().map(Output::Display),
            Self::Update(update) => update.run(zone, cancel).map(Output::Update),
            Self::Habits(scan) => scan.run(cancel).map(Output::Habits),
        }
    }
}
