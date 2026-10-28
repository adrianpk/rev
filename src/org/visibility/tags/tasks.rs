mod blocking;
mod edit;
mod logging;
mod model;
pub(super) mod repeat;

use super::*;
use crate::org::task::{Log, Workflow};
pub(super) use model::Metadata;

pub(crate) struct TaskChange {
    pub index: usize,
    pub old: String,
    pub next: String,
    pub workflow: Workflow,
    pub logging: Logging,
    pub repeat: Option<repeat::RepeatChange>,
}
pub(crate) struct Logging {
    pub closed: bool,
    pub state: Log,
    pub note: bool,
    pub drawer: String,
}
impl Outline {
    pub(crate) fn task_workflow(&self, policy: &crate::config::Todo) -> Result<Workflow, String> {
        self.tags.workflow.effective(policy)
    }
    pub(crate) fn task_word(&self, index: usize) -> &str {
        &self.tags.headers[index].word
    }
    pub(crate) fn task_change(
        &self,
        buffer: &Buffer,
        index: usize,
        next: &str,
        policy: &crate::config::Todo,
    ) -> Result<TaskChange, String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        let workflow = self.task_workflow(policy)?;
        if !next.is_empty() && workflow.state(next).is_none() {
            return Err("Unknown Org task state".into());
        }
        let old = workflow
            .state(self.task_word(index))
            .map_or("", |s| s.name.as_str())
            .to_owned();
        self.task_blocked(buffer, index, next, (&workflow, policy))?;
        let logging = self.task_logging(buffer, index, (&old, next), (&workflow, policy))?;
        let mut change = TaskChange {
            index,
            old,
            next: next.into(),
            workflow,
            logging,
            repeat: None,
        };
        self.task_repeat(buffer, &mut change, policy)?;
        Ok(change)
    }
    fn task_property(&self, buffer: &Buffer, index: usize, name: &str) -> Result<String, String> {
        let point = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .from_byte(crate::coordinates::Byte(self.headings[index].start))
            .map_err(|e| e.to_string())?;
        let entry = self.property_target(buffer, point)?;
        self.property_value(buffer, entry, name, &[])
            .map(|p| p.text)
    }
}
