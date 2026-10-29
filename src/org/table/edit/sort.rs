use super::*;
use std::{cmp::Ordering, collections::BTreeMap};
enum Key {
    Text(String),
    Number(f64),
}
impl Key {
    fn new(value: &str, numeric: bool) -> Result<Self, String> {
        if !numeric {
            return Ok(Self::Text(value.to_lowercase()));
        }
        let number = if value.is_empty() {
            0.0
        } else {
            value
                .parse::<f64>()
                .map_err(|_| "Numeric table sort requires finite numbers")?
        };
        if !number.is_finite() {
            return Err("Numeric table sort requires finite numbers".into());
        }
        Ok(Self::Number(number))
    }
    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            (Self::Number(a), Self::Number(b)) => a.total_cmp(b),
            _ => unreachable!("one selected sort mode"),
        }
    }
}
impl Table {
    pub(super) fn sort(
        &mut self,
        at: &mut Position,
        argument: &str,
        scope: (usize, Range<usize>),
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if self.rows[at.row].separator {
            return Err("Select a data row for sorting".into());
        }
        let (numeric, reverse) = sort_mode(argument)?;
        let (column, range) = scope;
        let (start, end) = (range.start, range.end);
        let mut keys = BTreeMap::new();
        for row in &self.rows[start..end] {
            check(cancel)?;
            let value = row.cells.get(column).map_or("", |c| c.value.as_str());
            keys.insert(row.source.start, Key::new(value, numeric)?);
        }
        let selected = self.rows[at.row].source.start;
        self.rows[start..end].sort_by(|a, b| {
            let order = keys[&a.source.start].compare(&keys[&b.source.start]);
            if reverse { order.reverse() } else { order }
        });
        at.row = self
            .rows
            .iter()
            .position(|r| r.source.start == selected)
            .expect("retained selected row");
        Ok(())
    }
}
fn sort_mode(argument: &str) -> Result<(bool, bool), String> {
    match argument.trim() {
        "a" => Ok((false, false)),
        "A" => Ok((false, true)),
        "n" => Ok((true, false)),
        "N" => Ok((true, true)),
        _ => Err("Sort type must be a/A (alphabetical) or n/N (numeric); source unchanged".into()),
    }
}

impl Snapshot {
    pub(super) fn sort_scope(
        &self,
        table: &Table,
        at: Position,
    ) -> Result<(usize, Range<usize>), String> {
        if at.prefix || table.rows[at.row].separator {
            return Err("Select a data field for sorting".into());
        }
        let Some(region) = &self.region else {
            let start = table.rows[..at.row]
                .iter()
                .rposition(|r| r.separator)
                .map_or(0, |r| r + 1);
            let end = table.rows[at.row + 1..]
                .iter()
                .position(|r| r.separator)
                .map_or(table.rows.len(), |r| at.row + 1 + r);
            return Ok((at.column, start..end));
        };
        if region.start < table.source.start || region.end > table.source.end {
            return Err("Sort region must stay inside one native table".into());
        }
        let mark = table.position(&self.text, self.mark.ok_or("Sort region has no mark")?);
        let range = mark.row.min(at.row)..mark.row.max(at.row) + 1;
        if mark.prefix || table.rows[range.clone()].iter().any(|r| r.separator) {
            return Err("Sort region must select data fields within one separator group".into());
        }
        Ok((mark.column, range))
    }
}
