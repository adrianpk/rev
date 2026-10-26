use super::{Context, Input, Outcome};

impl Input {
    /// A paste cancels pending chord modifiers but retains its focused owner.
    pub(super) fn paste(&mut self, text: String) -> Outcome {
        self.prefix = None;
        self.meta_deadline = None;
        self.suppress_repeat = true;
        match &mut self.context {
            Context::Editing => Outcome::Paste(text),
            Context::Command(palette) => palette.paste(&text),
            Context::Argument(prompt) => prompt.paste(&text),
            _ => Outcome::Message("Paste is unavailable in this input context"),
        }
    }
}
