use super::*;
impl Document {
    pub(super) fn context(
        &self,
        at: usize,
        command: C,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        self.context_allowed(at)?;
        if let Some(result) = self.footnote_context(at, cancel)? {
            return Ok(result);
        }
        if let Some(target) = self
            .objects
            .iter()
            .find(|o| o.kind == InlineKind::RadioTarget && o.source.contains(&at))
        {
            self.radio_unique(&self.slice(target.content.clone()))?;
            return Ok(Output::Message(
                "Radio targets checked against current source".into(),
            ));
        }
        if command == C::OpenLink
            && let Some(byte) = self.radio_at(at, cancel)?
        {
            return Ok(Output::Move(byte));
        }
        Ok(Output::Continue)
    }
    pub(super) fn context_allowed(&self, at: usize) -> Result<(), String> {
        if super::observe::protected_range(&self.protected, &(at..at)) {
            return Err("Literal block, macro or unsupported context; source unchanged".into());
        }
        if self.objects.iter().any(|o| {
            matches!(o.kind, InlineKind::Code | InlineKind::Verbatim) && o.source.contains(&at)
        }) {
            return Err("Literal code/verbatim context; edit source explicitly".into());
        }
        Ok(())
    }
}
