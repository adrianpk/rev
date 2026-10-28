use super::ClockScan;
use crate::{
    buffer::{Span, edit::Replacement},
    org::{Cancellation, time::Zone},
};
pub(crate) struct ClockUpdate {
    pub scan: ClockScan,
    pub source: Span,
    pub prefix: String,
    pub suffix: String,
    pub maxlevel: usize,
    pub newline: String,
}
impl ClockUpdate {
    pub(crate) fn run(self, zone: &Zone, cancel: &Cancellation) -> Result<Replacement, String> {
        let summary = self.scan.run(zone, cancel)?;
        let body = summary.table_with(self.maxlevel, &self.newline, cancel)?;
        let text = format!("{}{body}{}", self.prefix, self.suffix);
        if text.len() > 1048576 {
            return Err("Clocktable output exceeds 1048576 UTF-8 bytes".into());
        }
        Ok(Replacement {
            source: self.source,
            text,
        })
    }
}
