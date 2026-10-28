use super::*;

impl Outline {
    pub(super) fn task_blocked(
        &self,
        buffer: &Buffer,
        index: usize,
        next: &str,
        settings: (&Workflow, &crate::config::Todo),
    ) -> Result<(), String> {
        let (workflow, policy) = settings;
        if !workflow.state(next).is_some_and(|s| s.done)
            || self.task_property(buffer, index, "NOBLOCKING")? == "t"
        {
            return Ok(());
        }
        if policy.block_children {
            self.task_children(index, workflow)?;
        }
        if let Some(parent) = self.headings[index].parent
            && self.task_property(buffer, parent, "ORDERED")? == "t"
        {
            self.task_predecessors(index, parent, workflow)?;
        }
        Ok(())
    }
    fn task_children(&self, index: usize, workflow: &Workflow) -> Result<(), String> {
        for child in index + 1..self.headings.len() {
            if self.headings[child].start >= self.headings[index].end {
                break;
            }
            if workflow
                .state(self.task_word(child))
                .is_some_and(|s| !s.done)
            {
                return Err(format!(
                    "Org completion blocked: unfinished child {}",
                    self.task_word(child)
                ));
            }
        }
        Ok(())
    }
    fn task_predecessors(
        &self,
        index: usize,
        parent: usize,
        workflow: &Workflow,
    ) -> Result<(), String> {
        for sibling in parent + 1..index {
            if self.headings[sibling].parent == Some(parent)
                && workflow
                    .state(self.task_word(sibling))
                    .is_some_and(|s| !s.done)
            {
                return Err("Org completion blocked: unfinished earlier sibling (ORDERED)".into());
            }
        }
        Ok(())
    }
}
