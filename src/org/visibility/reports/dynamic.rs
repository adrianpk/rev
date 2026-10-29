use super::*;
use crate::org::agenda::ClockTable;
use crate::org::agenda::ClockUpdate;
use crate::org::time::ZonedOperation;
impl Outline {
    pub(crate) fn clock_table_update(
        &self,
        buffer: &Buffer,
        point: Scalar,
        operation: &ZonedOperation,
        policy: &crate::config::Org,
    ) -> Result<ClockUpdate, String> {
        self.report_current(buffer)?;
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = c.to_byte(point).map_err(|e| e.to_string())?.0;
        if let Some(block) = self
            .dynamic
            .iter()
            .find(|b| b.start <= byte && byte < b.end)
        {
            return self.update_clock_table(buffer, point, block, (operation, policy));
        }
        self.insert_clock_table(buffer, point, (operation, policy))
    }
    pub(crate) fn clock_table_at(&self, buffer: &Buffer, point: Scalar) -> bool {
        let Ok(c) = Coordinates::new(buffer) else {
            return false;
        };
        let Ok(byte) = c.to_byte(point) else {
            return false;
        };
        self.dynamic
            .iter()
            .any(|b| b.start <= byte.0 && byte.0 < b.end)
    }
    fn update_clock_table(
        &self,
        buffer: &Buffer,
        point: Scalar,
        block: &Dynamic,
        data: (&ZonedOperation, &crate::config::Org),
    ) -> Result<ClockUpdate, String> {
        let name = source(buffer, block.name.clone())?;
        if !name.eq_ignore_ascii_case("clocktable") {
            return Err("Selected dynamic block is not a clocktable".into());
        }
        let header = source(buffer, block.name.end..block.header.end)?;
        let table = ClockTable::parse(header.trim(), data.0.now())?;
        let scan = self.clock_scan(buffer, point, table.query, data.1)?;
        Ok(ClockUpdate {
            scan,
            source: buffer
                .byte_span(block.body.clone())
                .map_err(|e| e.to_string())?,
            prefix: String::new(),
            suffix: String::new(),
            maxlevel: table.maxlevel,
            newline: buffer.newline().into(),
        })
    }

    fn insert_clock_table(
        &self,
        buffer: &Buffer,
        point: Scalar,
        data: (&ZonedOperation, &crate::config::Org),
    ) -> Result<ClockUpdate, String> {
        let span = self.report_insertion(buffer, point)?;
        let table = ClockTable::parse("", data.0.now())?;
        let scan = self.clock_scan(buffer, point, table.query, data.1)?;
        let nl = buffer.newline();
        Ok(ClockUpdate {
            scan,
            source: span,
            prefix: format!("#+BEGIN: clocktable :maxlevel 2 :scope file{nl}"),
            suffix: "#+END:".into(),
            maxlevel: 2,
            newline: nl.into(),
        })
    }

    pub(super) fn report_insertion(
        &self,
        buffer: &Buffer,
        point: Scalar,
    ) -> Result<crate::buffer::Span, String> {
        if !self.timestamp_context(buffer, point)? {
            return Err("Clocktable insertion requires ordinary Org source".into());
        }
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let start = c.line_start(point).map_err(|e| e.to_string())?;
        let end = c.line_end(point).map_err(|e| e.to_string())?;
        let span = buffer.span(start.0..end.0).map_err(|e| e.to_string())?;
        if !buffer
            .slice(&span)
            .map_err(|e| e.to_string())?
            .to_string()
            .trim()
            .is_empty()
        {
            return Err("Insert a clocktable on an empty line or at empty EOF".into());
        }
        Ok(span)
    }
}
