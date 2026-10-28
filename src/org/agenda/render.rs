use super::ClockSummary;
use crate::org::Duration;
impl ClockSummary {
    /// Produce compatible Org table text. The caller owns explicit publication;
    /// preparing or viewing a result never writes source.
    pub fn table(&self, maxlevel: usize, newline: &str) -> Result<String, String> {
        self.table_with(maxlevel, newline, &crate::org::Cancellation::default())
    }
    pub(crate) fn table_with(
        &self,
        maxlevel: usize,
        newline: &str,
        cancel: &crate::org::Cancellation,
    ) -> Result<String, String> {
        if !(1..=256).contains(&maxlevel) {
            return Err("Clocktable maxlevel requires 1..256".into());
        }
        self.table_bounds(newline)?;
        let mut work = super::scan::Progress {
            cancel,
            yielded: std::time::Instant::now(),
        };
        let totals = self.subtree_minutes()?;
        let mut text = self.table_header(newline)?;
        for (index, row) in self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.level <= maxlevel)
        {
            work.tick()?;
            append_row(&mut text, row, totals[index], newline)?;
        }
        text.push_str(&format!(
            "{newline}Open: {}; unresolved: {}; overlapping intervals: {}.{newline}",
            self.open, self.unresolved, self.overlaps
        ));
        output_bound(&text)?;
        work.tick()?;
        Ok(text)
    }
    fn table_header(&self, newline: &str) -> Result<String, String> {
        Ok(format!(
            "| Headline | Time | EFFORT | Open | Unresolved |{newline}|-|-|-|-|-|{newline}| *Total time* | *{}* | {} | {} | {} |{newline}",
            clock(self.minutes)?,
            effort(self.effort)?,
            self.open,
            self.unresolved
        ))
    }
    fn table_bounds(&self, newline: &str) -> Result<(), String> {
        if self.rows.len() > 4096 || !["\n", "\r\n"].contains(&newline) {
            return Err("Clocktable exceeds row/newline bounds".into());
        }
        if self
            .rows
            .iter()
            .any(|r| r.level == 0 || r.level > 256 || r.title.len() > 262144)
        {
            return Err("Clocktable exceeds row bounds".into());
        }
        Ok(())
    }
    fn subtree_minutes(&self) -> Result<Vec<u64>, String> {
        let mut totals: Vec<_> = self.rows.iter().map(|r| r.minutes).collect();
        for index in (0..self.rows.len()).rev() {
            if let Some(parent) = self.rows[index].parent {
                if parent >= index {
                    return Err("Invalid clock row parent".into());
                }
                totals[parent] = totals[parent]
                    .checked_add(totals[index])
                    .ok_or("Clock total overflow")?;
            }
        }
        Ok(totals)
    }
}
fn clock(minutes: u64) -> Result<String, String> {
    Duration::from_minutes(minutes as f64)?.format_clock(false)
}
fn effort(value: Option<Duration>) -> Result<String, String> {
    Ok(value
        .map(|v| v.format_clock(false))
        .transpose()?
        .unwrap_or_default())
}

fn append_row(
    text: &mut String,
    row: &super::ClockRow,
    minutes: u64,
    newline: &str,
) -> Result<(), String> {
    let title = row.title.replace('|', "\\vert{}");
    text.push_str(&format!(
        "| {}{} | {} | {} | {} | {} |{newline}",
        "\\_ ".repeat(row.level.saturating_sub(1)),
        title,
        clock(minutes)?,
        effort(row.effort)?,
        row.open,
        row.unresolved
    ));
    output_bound(text)
}
fn output_bound(text: &str) -> Result<(), String> {
    if text.len() > 1048576 {
        return Err("Clocktable output exceeds 1048576 UTF-8 bytes".into());
    }
    Ok(())
}
