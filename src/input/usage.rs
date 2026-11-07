use super::Input;
use crate::command::usage::History;

impl Input {
    /// Application histories survive context replacement; transient focus and
    /// source/navigation identities belong to the fresh input owner.
    pub(crate) fn inherit_histories(&mut self, previous: &mut Self) {
        std::mem::swap(&mut self.history, &mut previous.history);
        std::mem::swap(&mut self.command_usage, &mut previous.command_usage);
        std::mem::swap(&mut self.shell_history, &mut previous.shell_history);
        self.suppress_repeat();
    }
    pub(crate) fn set_command_usage(&mut self, history: History) {
        self.command_usage = history;
    }
    pub(crate) fn publish_command_usage(&mut self) {
        self.command_usage.publish();
    }
    pub(crate) fn poll_command_usage(&mut self) -> Option<String> {
        let notice = self.command_usage.poll();
        self.command_usage.publish_new();
        notice
    }
    pub(crate) fn command_usage_remaining(&self) -> Option<std::time::Duration> {
        self.command_usage.remaining()
    }
    pub(crate) fn flush_command_usage(&mut self) -> Result<(), String> {
        self.command_usage.flush()
    }
}
