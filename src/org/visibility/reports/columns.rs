use super::*;
use crate::org::columns::{Format, Options, Update};
pub(crate) struct ColumnPlan {
    pub format: Format,
    pub clocks: ClockScan,
    pub update: Option<Update>,
    pub maxlevel: usize,
}
impl Outline {
    pub(crate) fn column_plan(
        &self,
        buffer: &Buffer,
        point: Scalar,
        policy: &crate::config::Org,
        insert: bool,
    ) -> Result<ColumnPlan, String> {
        self.report_current(buffer)?;
        let (options, update) = self.column_options(buffer, point, insert)?;
        let declaration = match options.format {
            Some(value) => value,
            None => self.column_declaration(buffer, point)?,
        };
        let scope = self.column_scope(buffer, point, options.global)?;
        Ok(ColumnPlan {
            format: Format::parse(&declaration)?,
            clocks: self.clock_scan(
                buffer,
                point,
                ClockQuery {
                    scope,
                    window: None,
                },
                policy,
            )?,
            update,
            maxlevel: options.maxlevel,
        })
    }
    fn column_scope(
        &self,
        buffer: &Buffer,
        point: Scalar,
        global: bool,
    ) -> Result<ClockScope, String> {
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = c.to_byte(point).map_err(|e| e.to_string())?.0;
        Ok(if global || self.heading(byte).is_none() {
            ClockScope::File
        } else {
            ClockScope::Subtree
        })
    }
    fn column_options(
        &self,
        buffer: &Buffer,
        point: Scalar,
        update: bool,
    ) -> Result<(Options, Option<Update>), String> {
        if !update {
            return Ok((Options::parse("")?, None));
        }
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = c.to_byte(point).map_err(|e| e.to_string())?.0;
        if let Some(block) = self
            .dynamic
            .iter()
            .find(|b| b.start <= byte && byte < b.end)
        {
            return column_update(buffer, block).map(|(options, update)| (options, Some(update)));
        }
        let mut options = Options::parse("")?;
        options.global = true;
        let update = Update {
            source: self.report_insertion(buffer, point)?,
            prefix: format!("#+BEGIN: columnview :id global{}", buffer.newline()),
            suffix: "#+END:".into(),
            newline: buffer.newline().into(),
        };
        Ok((options, Some(update)))
    }
    pub(crate) fn column_block_at(&self, buffer: &Buffer, point: Scalar) -> bool {
        let Ok(c) = Coordinates::new(buffer) else {
            return false;
        };
        let Ok(byte) = c.to_byte(point) else {
            return false;
        };
        self.dynamic.iter().any(|b| {
            b.start <= byte.0
                && byte.0 < b.end
                && source(buffer, b.name.clone())
                    .is_ok_and(|n| n.eq_ignore_ascii_case("columnview"))
        })
    }
    fn column_declaration(&self, buffer: &Buffer, point: Scalar) -> Result<String, String> {
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = c.to_byte(point).map_err(|e| e.to_string())?.0;
        if let Some(index) = self.heading(byte) {
            let entry = self.heading_property_entry(buffer, index)?;
            let value = self.property_value(buffer, entry, "COLUMNS", &["COLUMNS".into()])?;
            if !value.records.is_empty() {
                return Ok(value.text);
            }
        }
        Ok(self.column_format.clone()?.unwrap_or_default())
    }
}
pub(in crate::org::visibility) fn declaration(
    structure: &crate::org::Structure,
    text: &ropey::Rope,
    cancel: &crate::org::Cancellation,
) -> Result<Option<String>, String> {
    use crate::org::StructureKind as K;
    let nodes: Vec<_> = structure.raw_nodes().collect();
    let mut value = None;
    for node in nodes.iter().filter(|n| n.kind == K::Keyword) {
        if cancel.is_cancelled() {
            return Err("Column declaration observation cancelled".into());
        }
        let top = node
            .parent
            .is_none_or(|p| nodes[p].kind == K::Section && nodes[p].parent.is_none());
        if !top {
            continue;
        }
        if node.name.as_ref().is_some_and(|r| {
            text.byte_slice(r.clone())
                .to_string()
                .eq_ignore_ascii_case("COLUMNS")
        }) {
            if value.is_some() {
                return Err("Multiple file COLUMNS declarations are ambiguous".into());
            }
            if node.content.len() > 4096 {
                return Err("Column format exceeds 4096 bytes".into());
            }
            value = Some(
                text.byte_slice(node.content.clone())
                    .to_string()
                    .trim()
                    .into(),
            );
        }
    }
    Ok(value)
}

fn column_update(buffer: &Buffer, block: &Dynamic) -> Result<(Options, Update), String> {
    if !source(buffer, block.name.clone())?.eq_ignore_ascii_case("columnview") {
        return Err("Selected dynamic block is not a columnview".into());
    }
    let options = Options::parse(source(buffer, block.name.end..block.header.end)?.trim())?;
    let update = Update {
        source: buffer
            .byte_span(block.body.clone())
            .map_err(|e| e.to_string())?,
        prefix: String::new(),
        suffix: String::new(),
        newline: buffer.newline().into(),
    };
    Ok((options, update))
}
