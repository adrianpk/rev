use crate::buffer::Buffers;

use super::{Action, Pending, Session};

impl Session {
    pub(in crate::session) fn complete_buffer(&mut self) -> Action {
        if !matches!(self.pending, Some(Pending::Buffer { .. })) {
            return self.message("No buffer completion is pending");
        }
        let Some(prompt) = self.input.prompt() else {
            return self.message("No buffer argument is pending");
        };
        let Some((completed, count)) = completion(&self.buffers, prompt.text()) else {
            return self.message("No matching buffer");
        };
        if let Err(error) = self.input.complete(&completed) {
            return self.error(&error.to_string());
        }
        self.message(&format!(
            "{count} matching buffer(s); Enter selects an exact name"
        ))
    }
}

fn completion(buffers: &Buffers, prefix: &str) -> Option<(String, usize)> {
    let mut matches = buffers
        .iter()
        .map(|buffer| buffer.name())
        .filter(|name| name.starts_with(prefix));
    let mut completed = matches.next()?.to_owned();
    let mut count = 1;
    for name in matches {
        count += 1;
        completed = common_prefix(&completed, name);
    }
    Some((completed, count))
}

fn common_prefix(left: &str, right: &str) -> String {
    left.chars()
        .zip(right.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect()
}
