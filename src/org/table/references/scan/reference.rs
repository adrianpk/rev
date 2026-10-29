use super::*;
impl Scanner<'_> {
    pub(super) fn reference(&mut self) -> Result<Reference, String> {
        let row = if self.take("@") {
            Some(self.coordinate()?)
        } else {
            None
        };
        let column = if self.take("$") {
            Some(self.coordinate()?)
        } else {
            None
        };
        if row.is_none() && column.is_none() {
            return Err("Expected an absolute table reference".into());
        }
        Ok(Reference { row, column })
    }
    fn coordinate(&mut self) -> Result<Number, String> {
        let start = self.at;
        while self.byte().is_some_and(|b| b.is_ascii_digit()) {
            self.progress()?;
        }
        let value = self.text[start..self.at]
            .parse::<usize>()
            .map_err(|_| "Only bounded unsigned absolute coordinates are supported")?;
        if value == 0 {
            return Err(
                "Current/relative coordinates are unsupported for structural rewriting".into(),
            );
        }
        Ok(Number {
            value,
            source: start..self.at,
        })
    }
    pub(super) fn field(&mut self, axes: &Axes) -> Result<(), String> {
        let start = self.reference()?;
        axes.validate(&start)?;
        if self.take("..") {
            let end = self.reference()?;
            axes.validate(&end)?;
            axes.range(&start, &end)?;
            self.emit(&start, axes)?;
            self.emit(&end, axes)?;
        } else {
            self.emit(&start, axes)?;
        }
        Ok(())
    }
    pub(super) fn emit(&mut self, reference: &Reference, axes: &Axes) -> Result<(), String> {
        if let Some(row) = &reference.row {
            self.number(row, &axes.rows)?;
        }
        if let Some(column) = &reference.column {
            self.number(column, &axes.columns)?;
        }
        Ok(())
    }
    fn number(&mut self, number: &Number, axis: &Axis) -> Result<(), String> {
        let target = axis.target(number.value)?;
        if target != number.value {
            self.edits.push((number.source.clone(), target.to_string()));
        }
        Ok(())
    }
}
