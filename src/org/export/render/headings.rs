use super::*;
impl Render<'_> {
    pub(super) fn heading(
        &mut self,
        node: &structure::Raw,
        level: usize,
    ) -> Result<String, String> {
        let title = self.title(node)?;
        let level = level.saturating_sub(self.base).max(1);
        while self.numbers.len() < level {
            self.numbers.push(0);
        }
        self.numbers.truncate(level);
        self.numbers[level - 1] += 1;
        let number = if self.snapshot.options.numbered {
            format!(
                "{} ",
                self.numbers
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            )
        } else {
            String::new()
        };
        let title = format!("{number}{title}");
        self.headings
            .push(format!("{}- {title}", "  ".repeat(level - 1)));
        let marker = if self.markdown() {
            format!("{} ", "#".repeat(level.min(6)))
        } else {
            String::new()
        };
        Ok(format!("{marker}{title}\n"))
    }
    fn title(&mut self, node: &structure::Raw) -> Result<String, String> {
        let range = title_range(
            &self.snapshot.text,
            node,
            &self.snapshot.options,
            &self.snapshot.todo,
        );
        self.inline.range(range, 0)
    }
    pub(super) fn header(&self) -> Result<String, String> {
        if self.org() || matches!(self.snapshot.request.scope, Scope::Region(_)) {
            return Ok(String::new());
        }
        let mut out = String::new();
        let metadata = &self.snapshot.metadata;
        let title_prefix = if self.markdown() { "# " } else { "" };
        for (value, enabled, prefix) in [
            (&metadata.title, self.snapshot.options.title, title_prefix),
            (&metadata.author, self.snapshot.options.author, ""),
            (&metadata.date, true, ""),
        ] {
            if let Some(value) = value.as_ref().filter(|_| enabled) {
                append(
                    &mut out,
                    &format!("{prefix}{}\n\n", inline::literal(value, self.markdown())),
                    self.snapshot.export_bytes,
                )?;
            }
        }
        self.contents(&mut out)?;
        Ok(out)
    }
    fn contents(&self, out: &mut String) -> Result<(), String> {
        if self.snapshot.options.toc && !self.headings.is_empty() {
            append(out, "Contents\n\n", self.snapshot.export_bytes)?;
            for heading in &self.headings {
                append(out, &format!("{heading}\n"), self.snapshot.export_bytes)?;
            }
            append(out, "\n", self.snapshot.export_bytes)?;
        }
        Ok(())
    }
    pub(super) fn org_node(
        &mut self,
        id: usize,
        depth: usize,
        changes: &mut Vec<Change>,
    ) -> Result<(), String> {
        let n = &self.snapshot.nodes[id];
        if let StructureKind::Heading { level } = n.kind {
            let range = title_range(
                &self.snapshot.text,
                n,
                &self.snapshot.options,
                &self.snapshot.todo,
            );
            self.change(
                changes,
                n.content.clone(),
                self.snapshot.text.byte_slice(range).to_string(),
            )?;
            if self.base > 0 {
                let prefix = n.opening.as_ref().expect("heading").start..n.content.start;
                self.change(
                    changes,
                    prefix,
                    format!("{} ", "*".repeat(level.saturating_sub(self.base).max(1))),
                )?;
            }
        }
        if n.kind == StructureKind::Keyword
            && let Some((key, _)) = document::keyword(&self.snapshot.text, n)
            && suppressed(&key, &self.snapshot.options)
        {
            self.change(changes, n.source.clone(), String::new())?;
        }
        self.descend(id, depth, changes)
    }
    pub(super) fn keyword(&mut self, n: &structure::Raw) -> Result<String, String> {
        let key = n
            .name
            .as_ref()
            .map(|r| {
                self.snapshot
                    .text
                    .byte_slice(r.clone())
                    .to_string()
                    .to_ascii_uppercase()
            })
            .unwrap_or_default();
        if key == "CAPTION" {
            return Ok(format!("{}\n", self.inline.range(n.content.clone(), 0)?));
        }
        if known(&key) {
            return Ok(String::new());
        }
        self.diagnostic(format!(
            "Unsupported export keyword retained literally: {key}"
        ))?;
        Ok(inline::literal(
            &self.snapshot.text.byte_slice(n.source.clone()).to_string(),
            self.markdown(),
        ))
    }
}
fn title_range(
    text: &Rope,
    n: &structure::Raw,
    options: &Options,
    todo: &[String],
) -> Range<usize> {
    let value = text.byte_slice(n.content.clone()).to_string();
    let mut start = 0;
    let mut end = value.len();
    if !options.todo
        && let Some(word) = value.split_ascii_whitespace().next()
        && todo.iter().any(|s| s == word)
    {
        start = word.len();
        start += value[start..].len() - value[start..].trim_start().len();
    }
    if !options.tags
        && let Some(at) = value.rfind(char::is_whitespace)
        && valid_tags(&value[at..])
    {
        end = at;
    }
    n.content.start + start..n.content.start + end.max(start)
}
fn valid_tags(value: &str) -> bool {
    let value = value.trim();
    value.starts_with(':')
        && value.ends_with(':')
        && value.len() > 2
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || ":_@#%".contains(c))
}
fn suppressed(key: &str, options: &Options) -> bool {
    key == "TITLE" && !options.title || matches!(key, "AUTHOR" | "EMAIL") && !options.author
}
fn known(key: &str) -> bool {
    matches!(
        key,
        "TITLE"
            | "AUTHOR"
            | "EMAIL"
            | "DATE"
            | "LANGUAGE"
            | "OPTIONS"
            | "NAME"
            | "TODO"
            | "SEQ_TODO"
            | "TYP_TODO"
            | "FILETAGS"
            | "PRIORITIES"
            | "COLUMNS"
    ) || key.starts_with("ATTR_")
}
