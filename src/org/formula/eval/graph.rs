use super::*;
use std::collections::VecDeque;
impl Engine<'_> {
    pub(super) fn order(&mut self) -> Result<Vec<usize>, Error> {
        let mut outgoing = vec![Vec::new(); self.grid.cells.len()];
        let mut degree = vec![0; self.grid.cells.len()];
        let mut active = vec![false; self.grid.cells.len()];
        let mut pending: Vec<_> = self
            .owners
            .iter()
            .enumerate()
            .filter(|(c, owner)| owner.is_some() && self.selection.includes(self.grid.location(*c)))
            .map(|(c, _)| c)
            .collect();
        while let Some(cell) = pending.pop() {
            self.work.charge(1)?;
            if active[cell] {
                continue;
            }
            active[cell] = true;
            let index = self.owners[cell].expect("assigned dependency");
            let assignment = &self.program.assignments[index];
            let dependencies = self
                .dependencies(cell, assignment.expression)
                .map_err(|e| e.located(self.grid.location(cell), assignment.source.clone()))?;
            degree[cell] = dependencies.len();
            for dependency in dependencies {
                self.work.charge(1)?;
                outgoing[dependency].push(cell);
                pending.push(dependency);
            }
        }
        self.sorted(outgoing, degree, active)
    }
    fn sorted(
        &mut self,
        outgoing: Vec<Vec<usize>>,
        mut degree: Vec<usize>,
        active: Vec<bool>,
    ) -> Result<Vec<usize>, Error> {
        let mut ready: VecDeque<_> = degree
            .iter()
            .enumerate()
            .filter(|(cell, n)| **n == 0 && active[*cell])
            .map(|(cell, _)| cell)
            .collect();
        let mut order = Vec::new();
        while let Some(cell) = ready.pop_front() {
            self.work.charge(1)?;
            order.push(cell);
            for next in &outgoing[cell] {
                self.work.charge(1)?;
                degree[*next] -= 1;
                if degree[*next] == 0 {
                    ready.push_back(*next);
                }
            }
        }
        if let Some(cell) = degree.iter().position(|n| *n > 0) {
            let assignment = &self.program.assignments[self.owners[cell].expect("cyclic formula")];
            return Err(Error::new(ErrorKind::Cycle, "cyclic formula dependency")
                .located(self.grid.location(cell), assignment.source.clone()));
        }
        Ok(order)
    }
}
