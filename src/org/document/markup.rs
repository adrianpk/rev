use super::*;
impl Document {
    pub(super) fn emphasize(
        &self,
        point: usize,
        region: Option<Range<usize>>,
        argument: Argument,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        let range = region.unwrap_or(point..point);
        self.standard(&range)?;
        let Argument::Markup(marker) = argument else {
            return Ok(Output::Prompt);
        };
        if marker.len() != 1 || !"*/_+=~ ".contains(&marker) {
            return Err("Choose one emphasis marker: * / _ + = ~ or a space to remove".into());
        }
        let content = self.markup_content(&range);
        let selected = self.slice(content);
        let text = if marker == " " {
            selected
        } else {
            format!("{marker}{selected}{marker}")
        };
        if !range.is_empty() && marker != " " {
            self.validate_markup(&range, &text, cancel)?;
        }
        let cursor = if range.is_empty() {
            Some((range.start, 1))
        } else {
            None
        };
        self.change(vec![Edit { bytes: range, text }], cursor)
    }
    fn markup_content(&self, range: &Range<usize>) -> Range<usize> {
        self.objects
            .iter()
            .find(|o| o.source == *range && emphasis(o.kind))
            .map_or_else(|| range.clone(), |o| o.content.clone())
    }
    fn validate_markup(
        &self,
        range: &Range<usize>,
        replacement: &str,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let mut text = self.text.clone();
        text.remove(text.byte_to_char(range.start)..text.byte_to_char(range.end));
        text.insert(text.byte_to_char(range.start), replacement);
        let nodes = inline::objects(&text, 0..text.len_bytes(), InlineContext::Standard, cancel)?;
        if !nodes
            .iter()
            .any(|o| emphasis(o.kind) && o.source == (range.start..range.start + replacement.len()))
        {
            return Err("Selection does not form valid Org emphasis at these boundaries".into());
        }
        Ok(())
    }
    pub(super) fn entity(&self, at: usize, argument: Argument) -> Result<Output, String> {
        self.standard(&(at..at))?;
        let Argument::Entity(name) = argument else {
            return Ok(Output::Prompt);
        };
        if name.len() > 64 || !inline::entity::known(&name) {
            return Err("Unknown native Org entity name; source unchanged".into());
        }
        let text = format!("\\{name}{{}}");
        let offset = text.chars().count();
        self.change(
            vec![Edit {
                bytes: at..at,
                text,
            }],
            Some((at, offset)),
        )
    }
}
fn emphasis(kind: InlineKind) -> bool {
    matches!(
        kind,
        InlineKind::Bold
            | InlineKind::Italic
            | InlineKind::Underline
            | InlineKind::Strike
            | InlineKind::Code
            | InlineKind::Verbatim
    )
}
