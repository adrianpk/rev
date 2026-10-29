use super::*;
impl Engine<'_> {
    pub(super) fn owners(&mut self) -> Result<Vec<Option<usize>>, Error> {
        let mut columns = vec![None; self.grid.columns];
        let mut fields = vec![None; self.grid.cells.len()];
        for (index, assignment) in self.program.assignments.iter().enumerate() {
            self.work.charge(1)?;
            let cell = self.grid.resolve(&assignment.target, 0)?;
            if assignment.target.row.is_some() {
                fields[cell] = Some(index);
            } else {
                columns[cell] = Some(index);
            }
        }
        for (cell, field) in fields.iter_mut().enumerate() {
            self.work.charge(1)?;
            if field.is_none() && cell / self.grid.columns >= self.grid.header_rows {
                *field = columns[cell % self.grid.columns];
            }
        }
        Ok(fields)
    }
    pub(super) fn dependencies(&mut self, cell: usize, root: usize) -> Result<Vec<usize>, Error> {
        let mut pending = vec![root];
        let mut dependencies = Vec::new();
        while let Some(node) = pending.pop() {
            self.work.charge(1)?;
            let expression = &self.program.nodes[node].expression;
            self.leaf_dependencies(cell, expression, &mut dependencies)?;
            children(expression, &mut pending);
        }
        dependencies.sort_unstable();
        dependencies.dedup();
        Ok(dependencies)
    }
    fn leaf_dependencies(
        &mut self,
        cell: usize,
        expression: &Expression,
        dependencies: &mut Vec<usize>,
    ) -> Result<(), Error> {
        match expression {
            Expression::Field(reference) => {
                let target = self.grid.resolve(reference, cell)?;
                self.dependency(target, dependencies)
            }
            Expression::Range(start, end) => {
                let range = self.grid.rectangle(start, end, cell)?;
                for target in range.cells(self.grid.columns) {
                    self.dependency(target, dependencies)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn dependency(&mut self, cell: usize, dependencies: &mut Vec<usize>) -> Result<(), Error> {
        self.work.charge(1)?;
        if self.owners[cell].is_some() {
            dependencies.push(cell);
        }
        Ok(())
    }
}
fn children(expression: &Expression, pending: &mut Vec<usize>) {
    match expression {
        Expression::Unary(_, child) | Expression::Group(child) => pending.push(*child),
        Expression::Binary(_, left, right) => pending.extend([*left, *right]),
        Expression::Call(_, arguments) => pending.extend(arguments),
        _ => {}
    }
}
