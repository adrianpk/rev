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
    Update(ClockUpdate),
}
pub(in crate::session) enum Output {
    Display(String),
    Update(Replacement),
}
impl Work {
    pub(in crate::session) fn run(
        self,
        zone: &Zone,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        match self {
            Self::Display(scan) => scan.run(zone, cancel)?.feedback().map(Output::Display),
            Self::Update(update) => update.run(zone, cancel).map(Output::Update),
        }
    }
}
