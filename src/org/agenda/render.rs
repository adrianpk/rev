use super::ClockSummary;
use crate::org::Duration;
impl ClockSummary {
    /// Produce compatible Org table text. The caller owns explicit publication;
    /// preparing or viewing a result never writes source.
    pub fn table(&self, maxlevel: usize, newline: &str) -> Result<String, String> {
        if !(1..=256).contains(&maxlevel) {
            return Err("Clocktable maxlevel requires 1..256".into());
        }
        let totals = self.subtree_minutes()?;
        let mut text = format!(
            "| Headline | Time | EFFORT | Open | Unresolved |{newline}|-|-|-|-|-|{newline}"
        );
        text.push_str(&format!(
            "| *Total time* | *{}* | {} | {} | {} |{newline}",
            clock(self.minutes)?,
            effort(self.effort)?,
            self.open,
            self.unresolved
        ));
        for (index, row) in self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.level <= maxlevel)
        {
            let title = row.title.replace('|', "\\vert{}");
            text.push_str(&format!(
                "| {}{} | {} | {} | {} | {} |{newline}",
                "\\_ ".repeat(row.level.saturating_sub(1)),
                title,
                clock(totals[index])?,
                effort(row.effort)?,
                row.open,
                row.unresolved
            ));
            if text.len() > 1048576 {
                return Err("Clocktable output exceeds 1048576 UTF-8 bytes".into());
            }
        }
        text.push_str(&format!(
            "{newline}Open: {}; unresolved: {}; overlapping intervals: {}.{newline}",
            self.open, self.unresolved, self.overlaps
        ));
        Ok(text)
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
