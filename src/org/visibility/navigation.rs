use crate::command::OrgCommand;

use super::Outline;

impl Outline {
    /// Ordered canonical candidates. Only next/previous-visible navigation may
    /// skip a candidate because of this view's folds; it never invents a row.
    pub(crate) fn targets(
        &self,
        command: OrgCommand,
        byte: usize,
    ) -> Result<Vec<usize>, &'static str> {
        self.complete()?;
        match command {
            OrgCommand::NextHeading => Ok(self
                .headings
                .iter()
                .filter(|h| h.start > byte)
                .map(|h| h.start)
                .collect()),
            OrgCommand::PreviousHeading => Ok(self
                .headings
                .iter()
                .rev()
                .filter(|h| h.start < byte)
                .map(|h| h.start)
                .collect()),
            _ => self.related(command, byte).map(|target| vec![target]),
        }
    }

    fn related(&self, command: OrgCommand, byte: usize) -> Result<usize, &'static str> {
        let index = self.heading(byte).ok_or("No Org heading here")?;
        if command == OrgCommand::ParentHeading {
            return self.headings[index]
                .parent
                .map(|parent| self.headings[parent].start)
                .ok_or("No parent Org heading");
        }
        let candidate = if command == OrgCommand::ForwardHeading {
            (index + 1..self.headings.len())
                .find(|i| self.headings[*i].level <= self.headings[index].level)
        } else {
            (0..index)
                .rev()
                .find(|i| self.headings[*i].level <= self.headings[index].level)
        };
        candidate
            .filter(|i| self.headings[*i].level == self.headings[index].level)
            .map(|i| self.headings[i].start)
            .ok_or("No same-level Org heading")
    }
}
