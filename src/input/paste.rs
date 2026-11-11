use super::{Context, Input, Outcome};

impl Input {
    /// A paste cancels pending chord modifiers but retains its focused owner.
    pub(super) fn paste(&mut self, text: String) -> Outcome {
        if matches!(
            self.context,
            Context::Editing | Context::Command(_) | Context::ArgumentDigit(_)
        ) {
            self.clear_sequence();
        } else {
            self.clear_prefix();
        }
        self.suppress_repeat = true;
        let accepts_text = self.accepts_text();
        match &mut self.context {
            Context::Editing if accepts_text => Outcome::Paste(text),
            Context::Search(prompt) | Context::Literal(prompt) => prompt.literal_paste(&text),
            Context::Command(palette) => palette.paste(&text),
            Context::Names(names) => names.paste(&text),
            _ => self.paste_argument(&text),
        }
    }

    fn paste_argument(&mut self, text: &str) -> Outcome {
        match &mut self.context {
            Context::Calendar(prompt)
            | Context::Argument(prompt)
            | Context::ArgumentDigit(prompt)
            | Context::Shell(prompt)
            | Context::Path(prompt) => prompt.paste(text),
            _ => Outcome::Message("Paste is unavailable in this input context"),
        }
    }
}
