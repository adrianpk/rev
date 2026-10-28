use super::*;
impl Tags {
    pub(super) fn edit(
        &self,
        buffer: &Buffer,
        index: usize,
        draft: &str,
        remove: bool,
    ) -> Result<Vec<Replacement>, String> {
        let header = &self.headers[index];
        let mut tags = names(draft)?;
        if remove {
            if tags.len() != 1 {
                return Err("Enter exactly one local Org tag to remove".into());
            }
            let removed = tags.remove(0);
            tags = header
                .local
                .iter()
                .filter(|t| t.name != removed)
                .map(|t| t.name.clone())
                .collect();
        }
        let old: Vec<_> = header.local.iter().map(|t| &t.name).collect();
        if old.iter().copied().eq(tags.iter()) {
            return Ok(Vec::new());
        }
        let rendered = if tags.is_empty() {
            String::new()
        } else {
            format!(":{}:", tags.join(":"))
        };
        let space = if header.span.is_empty() && !rendered.is_empty() {
            "  "
        } else {
            ""
        };
        let source = buffer
            .byte_span(header.span.clone())
            .map_err(|e| e.to_string())?;
        Ok(vec![Replacement {
            source,
            text: format!("{space}{rendered}"),
        }])
    }
}
