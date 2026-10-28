use super::*;
impl Outline {
    pub(super) fn report_records(
        &self,
        buffer: &Buffer,
        window: Option<RecordedClock>,
        zone: &Zone,
        data: (&mut ClockSummary, &mut Vec<RecordedClock>),
    ) -> Result<(), String> {
        let (summary, records) = data;
        let text = buffer
            .slice(&summary.source)
            .map_err(|e| e.to_string())?
            .to_string();
        let mut scalar = summary.source.range.start;
        let mut count = 0;
        for line in text.split_inclusive('\n') {
            if line.trim_start().starts_with("CLOCK:") {
                self.report_line(buffer, (scalar, line), window, (zone, summary, records))?;
                count += 1;
            }
            if count > 4096 {
                return Err("Clock report exceeds 4096 records".into());
            }
            scalar += line.chars().count();
        }
        Ok(())
    }
    fn report_line(
        &self,
        buffer: &Buffer,
        line: (usize, &str),
        window: Option<RecordedClock>,
        data: (&Zone, &mut ClockSummary, &mut Vec<RecordedClock>),
    ) -> Result<(), String> {
        let (zone, summary, records) = data;
        if !self.timestamp_context(buffer, Scalar(line.0))? {
            return Ok(());
        }
        let owner = summary
            .rows
            .partition_point(|r| r.source.range.start <= line.0)
            .checked_sub(1);
        let Some(owner) = owner else {
            return Ok(());
        };
        let row = &mut summary.rows[owner];
        match RecordedClock::observe(line.1, zone) {
            Ok(Some(record)) => add_record(row, records, record.clipped(window))?,
            Ok(None) => row.open += 1,
            Err(_) => row.unresolved += 1,
        }
        Ok(())
    }
}
fn add_record(
    row: &mut ClockRow,
    records: &mut Vec<RecordedClock>,
    record: Option<RecordedClock>,
) -> Result<(), String> {
    if let Some(record) = record {
        row.minutes = row
            .minutes
            .checked_add(record.minutes())
            .ok_or("Clock total overflow")?;
        records.push(record);
    }
    Ok(())
}
