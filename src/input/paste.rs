use super::{Context, Input, Outcome};

impl Input {
    /// A paste cancels pending chord modifiers but retains its focused owner.
    pub(super) fn paste(&mut self, text: String) -> Outcome {
        self.clear_prefix();
        self.suppress_repeat = true;
        match &mut self.context {
            Context::Editing => Outcome::Paste(text),
            Context::Search(prompt) => prompt.literal_paste(&text),
            Context::Literal(prompt) => prompt.literal_paste(&text),
            Context::Command(palette) => palette.paste(&text),
            Context::Argument(prompt) | Context::Path(prompt) => prompt.paste(&text),
            _ => Outcome::Message("Paste is unavailable in this input context"),
        }
    }
}
