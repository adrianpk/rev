//! Explicit delimited-data work uses the existing table/source and file owners.
mod data;
mod publication;
use super::*;
use crate::{buffer::Save, file::Location, org::io::CaptureRead};
use std::path::Path;
pub(crate) enum Work {
    Import(Snapshot, CaptureRead, char),
    Export(Snapshot, Location, char),
    Publish(Save),
}
impl Work {
    pub fn import(snapshot: Snapshot, read: CaptureRead, path: &Path) -> Result<Self, String> {
        Ok(Self::Import(snapshot, read, data::Format::path(path)?.0))
    }
    pub fn export(snapshot: Snapshot, location: Location) -> Result<Self, String> {
        let delimiter = data::Format::path(location.path())?.0;
        Ok(Self::Export(snapshot, location, delimiter))
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Output, String> {
        check(cancel)?;
        match self {
            Self::Import(snapshot, read, delimiter) => {
                let source = read.run(cancel)?;
                let rows =
                    data::decode(&source.text().to_string(), data::Format(delimiter), cancel)?;
                let plan = snapshot.import(rows, cancel)?;
                Ok(Output::Imported(plan, Box::new(source)))
            }
            Self::Export(snapshot, location, delimiter) => {
                let rows = snapshot.export(cancel)?;
                let text = data::encode(&rows, data::Format(delimiter), cancel)?;
                Ok(Output::File(text, location))
            }
            Self::Publish(save) => Ok(Output::Exported(publication::publish(&save, cancel))),
        }
    }
}
impl Snapshot {
    fn import(&self, values: Vec<Vec<String>>, cancel: &Cancellation) -> Result<Plan, String> {
        let nodes = self.nodes(cancel)?;
        let (start, source) = self.import_location(&nodes)?;
        let indent = source.trim_end_matches(['\r', '\n']);
        let table = imported_table(start, indent, values, self.newline)?;
        let mut layout = self.layout(&table, cancel)?;
        layout.separate(
            if action::adjacent_row(&nodes, start, false) {
                self.newline
            } else {
                ""
            },
            if action::adjacent_row(&nodes, start + source.len(), true) {
                self.newline
            } else {
                ""
            },
        )?;
        let mut edits = Vec::new();
        self.gap(&mut edits, table.source, &layout.text)?;
        Ok(Plan {
            edits,
            cursor: None,
            retained: Vec::new(),
        })
    }
    fn import_location(&self, nodes: &[structure::Raw]) -> Result<(usize, String), String> {
        if self.region.is_some() {
            return Err("Import an Org table without an active region".into());
        }
        let line = self.text.byte_to_line(self.point);
        let start = self.text.line_to_byte(line);
        let source = self.text.line(line).to_string();
        let indent = source.trim_end_matches(['\r', '\n']);
        if !indent.bytes().all(|b| b" \t".contains(&b))
            || !action::creation_context(nodes, self.point, &self.text)
        {
            return Err("Import an Org table on an admitted blank line".into());
        }
        Ok((start, source))
    }
    fn export(&self, cancel: &Cancellation) -> Result<Vec<Vec<String>>, String> {
        let nodes = self.nodes(cancel)?;
        let table = Table::observe(&self.text, &nodes, self.point, cancel)?
            .ok_or("No native Org table at point")?;
        let mut values = Vec::new();
        for row in table.rows.into_iter().filter(|r| !r.separator) {
            check(cancel)?;
            if row.cells.len() != table.columns {
                return Err("Ragged table export would invent fields; source unchanged".into());
            }
            values.push(row.cells.into_iter().map(|c| c.value).collect());
        }
        if values.is_empty() {
            return Err("No table data rows to export".into());
        }
        Ok(values)
    }
}
fn imported_table(
    start: usize,
    indent: &str,
    values: Vec<Vec<String>>,
    newline: &str,
) -> Result<Table, String> {
    let count = values.len();
    let columns = values[0].len();
    let mut rows = Vec::with_capacity(count);
    for (index, values) in values.into_iter().enumerate() {
        let ending = if index + 1 < count { newline } else { "" };
        let mut row = Row::empty(start, indent, ending, false, columns);
        row.cells = values
            .into_iter()
            .map(|value| Cell {
                source: start..start,
                value,
            })
            .collect();
        rows.push(row);
    }
    let mut table = Table {
        source: start..start + indent.len(),
        rows,
        columns,
        formulas: Vec::new(),
        column_order: Vec::new(),
    };
    table.finish()?;
    Ok(table)
}
