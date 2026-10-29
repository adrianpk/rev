use super::*;
impl Document {
    pub(super) fn affiliated(&self, at: usize, argument: Argument) -> Result<Output, String> {
        let node = self.affiliated_owner(at)?;
        if node.kind == StructureKind::Paragraph {
            self.standard(&(at..at))?;
        }
        if node.kind == StructureKind::Block(super::super::BlockKind::Greater)
            && !node.name.as_ref().is_some_and(|r| {
                matches!(
                    self.slice(r.clone()).to_ascii_uppercase().as_str(),
                    "QUOTE" | "CENTER"
                )
            })
        {
            return Err("Unsupported block form; affiliated source unchanged".into());
        }
        let Argument::Affiliated(key, value) = argument else {
            return Ok(Output::Prompt);
        };
        let key = key.to_ascii_uppercase();
        valid_key(&key)?;
        if value.len() > 4096 || value.chars().any(char::is_control) {
            return Err("Affiliated value requires at most 4096 bytes without controls".into());
        }
        let existing = self.affiliated_value(node, &key)?;
        let nl = self.newline();
        let (bytes, text) = existing.map_or_else(
            || {
                let at = node.source.start;
                (at..at, format!("#+{key}: {value}{nl}"))
            },
            |range| (range, value.clone()),
        );
        self.change(vec![Edit { bytes, text }], None)
    }
    fn affiliated_owner(&self, at: usize) -> Result<&structure::Raw, String> {
        self.nodes
            .iter()
            .filter(|n| element(n.kind) && n.source.start <= at && at <= n.source.end)
            .min_by_key(|n| n.source.len())
            .ok_or_else(|| "Point must be on a paragraph, table or supported block".into())
    }
    fn affiliated_value(
        &self,
        owner: &structure::Raw,
        key: &str,
    ) -> Result<Option<Range<usize>>, String> {
        let mut found = None;
        let Some(affiliated) = &owner.affiliated else {
            return Ok(None);
        };
        for n in self.nodes.iter().filter(|n| {
            n.kind == StructureKind::AffiliatedKeyword && affiliated.contains(&n.source.start)
        }) {
            if n.name
                .as_ref()
                .is_some_and(|r| self.slice(r.clone()).eq_ignore_ascii_case(key))
                && found.replace(n.content.clone()).is_some()
            {
                return Err("Repeated affiliated keyword; edit its source explicitly".into());
            }
        }
        Ok(found)
    }
}
fn element(kind: StructureKind) -> bool {
    matches!(
        kind,
        StructureKind::Paragraph
            | StructureKind::Table
            | StructureKind::Block(
                super::super::BlockKind::Source
                    | super::super::BlockKind::Example
                    | super::super::BlockKind::Verse
                    | super::super::BlockKind::Greater
            )
    )
}
pub(crate) fn valid_key(key: &str) -> Result<(), String> {
    if key == "NAME" || key == "CAPTION" || attribute(key) {
        Ok(())
    } else {
        Err("Use NAME, CAPTION or ATTR_BACKEND; unknown forms remain literal source".into())
    }
}
fn attribute(key: &str) -> bool {
    key.strip_prefix("ATTR_").is_some_and(|s| {
        !s.is_empty() && s.len() <= 32 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    })
}
