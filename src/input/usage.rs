use super::Input;
use crate::command::usage::History;

impl Input {
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
