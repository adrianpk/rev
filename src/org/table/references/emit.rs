use super::*;
use crate::org::formula::Limits;
impl Axes {
    pub(super) fn rewrite(
        &self,
        text: &str,
        cancel: &Cancellation,
    ) -> Result<Vec<(Range<usize>, String)>, String> {
        let program = Program::parse(text, Limits::default(), cancel).map_err(|e| e.to_string())?;
        let mut edits = Vec::new();
        for assignment in program.assignments() {
            check(cancel)?;
            self.emit(&assignment.target, &mut edits)?;
        }
        for node in program.nodes() {
            check(cancel)?;
            self.node(&node.expression, &mut edits)?;
        }
        edits.sort_by_key(|(range, _)| range.start);
        Ok(edits)
    }
    fn node(
        &self,
        expression: &Expression,
        edits: &mut Vec<(Range<usize>, String)>,
    ) -> Result<(), String> {
        match expression {
            Expression::Field(reference) => self.emit(reference, edits),
            Expression::Range(start, end) => {
                self.range(start, end)?;
                self.emit(start, edits)?;
                self.emit(end, edits)
            }
            _ => Ok(()),
        }
    }
    fn emit(
        &self,
        reference: &Reference,
        edits: &mut Vec<(Range<usize>, String)>,
    ) -> Result<(), String> {
        self.validate(reference)?;
        if let Some(row) = &reference.row {
            number(row, &self.rows, edits)?;
        }
        if let Some(column) = &reference.column {
            number(column, &self.columns, edits)?;
        }
        Ok(())
    }
}
fn number(
    component: &Component,
    axis: &Axis,
    edits: &mut Vec<(Range<usize>, String)>,
) -> Result<(), String> {
    let old = absolute(component)?;
    let target = axis.target(old)?;
    if target != old {
        edits.push((component.source.clone(), target.to_string()));
    }
    Ok(())
}
