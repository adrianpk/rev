use super::*;
mod syntax;
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct Definitions(BTreeMap<String, String>);
impl Definitions {
    pub fn observe(text: &Rope, nodes: &[structure::Raw]) -> Result<Self, String> {
        let mut defs = Self::default();
        defs.collect(text, nodes)?;
        Ok(defs)
    }
    fn collect(&mut self, text: &Rope, nodes: &[structure::Raw]) -> Result<(), String> {
        for node in nodes {
            if let Some((key, value)) = document::keyword(text, node)
                && key == "MACRO"
            {
                self.declare(&value)?;
            }
        }
        Ok(())
    }
    fn declare(&mut self, value: &str) -> Result<(), String> {
        if value.len() > 4096 {
            return Err("Text macro definition exceeds 4096 bytes".into());
        }
        let (name, body) = value
            .trim()
            .split_once(char::is_whitespace)
            .ok_or("Text macro requires a name and replacement")?;
        syntax::name(name)?;
        let body = body.trim_start();
        if body.to_ascii_lowercase().starts_with("(eval") {
            return Err("Executable Org macro definitions are unsupported".into());
        }
        if self.0.len() == 128 && !self.0.contains_key(name) {
            return Err("Org macro definition limit reached".into());
        }
        self.0.insert(name.into(), body.into());
        Ok(())
    }
}
pub(super) struct Expanded {
    pub text: Rope,
    pub diagnostics: Vec<String>,
}
pub(super) fn expand(
    text: Rope,
    mut definitions: Definitions,
    metadata: &mut Metadata,
    limits: &crate::config::OrgLimits,
    cancel: &Cancellation,
) -> Result<Expanded, String> {
    let nodes = document::nodes(&text, cancel)?;
    definitions.collect(&text, &nodes)?;
    let mut work = Expansion {
        definitions,
        active: Vec::new(),
        steps: 0,
        limit: limits.expansion_bytes,
        cancel,
        diagnostics: Vec::new(),
        prepared_bytes: 0,
    };
    let text = work.document(&text, &nodes)?;
    work.metadata(metadata)?;
    Ok(Expanded {
        text: Rope::from_str(&text),
        diagnostics: work.diagnostics,
    })
}
struct Expansion<'a> {
    definitions: Definitions,
    active: Vec<String>,
    steps: usize,
    limit: usize,
    cancel: &'a Cancellation,
    diagnostics: Vec<String>,
    prepared_bytes: usize,
}
impl Expansion<'_> {
    fn document(&mut self, text: &Rope, nodes: &[structure::Raw]) -> Result<String, String> {
        let mut changes = Vec::new();
        for n in nodes {
            check(self.cancel)?;
            if document::keyword(text, n).is_some_and(|(k, _)| k == "MACRO") {
                changes.push((n.source.clone(), String::new()));
            } else if n.context.is_some_and(|c| c != InlineContext::Literal) {
                self.content(text, n, &mut changes)?;
            }
        }
        self.compose(text, changes)
    }
    fn compose(
        &self,
        text: &Rope,
        mut changes: Vec<(Range<usize>, String)>,
    ) -> Result<String, String> {
        changes.sort_by_key(|(r, _)| r.start);
        let mut result = String::new();
        let mut cursor = 0;
        for (range, value) in changes {
            if range.start < cursor {
                continue;
            }
            self.append(
                &mut result,
                &text.byte_slice(cursor..range.start).to_string(),
            )?;
            self.append(&mut result, &value)?;
            cursor = range.end;
        }
        self.append(&mut result, &text.byte_slice(cursor..).to_string())?;
        Ok(result)
    }
    fn content(
        &mut self,
        text: &Rope,
        node: &structure::Raw,
        changes: &mut Vec<(Range<usize>, String)>,
    ) -> Result<(), String> {
        let objects = super::super::inline::objects(
            text,
            node.content.clone(),
            node.context.expect("inline context"),
            self.cancel,
        )?;
        let mut cursor = node.content.start;
        for object in objects.iter().filter(|o| {
            matches!(
                o.kind,
                super::super::InlineKind::Code
                    | super::super::InlineKind::Verbatim
                    | super::super::InlineKind::Link(_)
            )
        }) {
            if object.source.start < cursor {
                continue;
            }
            self.part(text, cursor..object.source.start, changes)?;
            cursor = object.source.end;
        }
        self.part(text, cursor..node.content.end, changes)
    }
    fn part(
        &mut self,
        text: &Rope,
        range: Range<usize>,
        changes: &mut Vec<(Range<usize>, String)>,
    ) -> Result<(), String> {
        let value = text.byte_slice(range.clone()).to_string();
        if value.contains("{{{") {
            let mut result = String::new();
            self.expand(&value, &mut result)?;
            self.prepared_bytes = self
                .prepared_bytes
                .checked_add(result.len())
                .ok_or("Org macro output overflows")?;
            if self.prepared_bytes > self.limit {
                return Err("Org aggregate macro expansion byte limit reached".into());
            }
            changes.push((range, result));
        }
        Ok(())
    }
    fn expand(&mut self, value: &str, out: &mut String) -> Result<(), String> {
        let mut cursor = 0;
        while let Some(start) = value[cursor..].find("{{{").map(|p| cursor + p) {
            check(self.cancel)?;
            self.append(out, &value[cursor..start])?;
            let end = value[start + 3..]
                .find("}}}")
                .map(|p| start + 3 + p)
                .ok_or("Unclosed Org macro invocation")?;
            self.invoke(&value[start + 3..end], out)?;
            cursor = end + 3;
        }
        self.append(out, &value[cursor..])
    }
    fn invoke(&mut self, value: &str, out: &mut String) -> Result<(), String> {
        self.steps += 1;
        if self.steps > 100000 {
            return Err("Org macro invocation limit reached".into());
        }
        let (name, args) = syntax::invocation(value)?;
        let Some(body) = self.definitions.0.get(name).cloned() else {
            return self.unknown(value, name, out);
        };
        if self.active.len() == 16 || self.active.iter().any(|n| n == name) {
            return Err("Org text macro cycle or depth limit reached".into());
        }
        let value = syntax::substitute(&body, &args, self.limit)?;
        self.active.push(name.into());
        self.expand(&value, out)?;
        self.active.pop();
        Ok(())
    }
    fn unknown(&mut self, value: &str, name: &str, out: &mut String) -> Result<(), String> {
        let diagnostic = format!("Unsupported Org macro retained as source: {name}");
        if !self.diagnostics.contains(&diagnostic) {
            if self.diagnostics.len() == 128 {
                return Err("Org macro diagnostic limit reached".into());
            }
            self.diagnostics.push(diagnostic);
        }
        self.append(out, "{{{")?;
        self.append(out, value)?;
        self.append(out, "}}}")
    }
    fn append(&self, out: &mut String, value: &str) -> Result<(), String> {
        if value.len() > self.limit.saturating_sub(out.len()) {
            return Err("Org expansion byte limit reached".into());
        }
        let mut start = 0;
        while start < value.len() {
            check(self.cancel)?;
            let mut end = (start + 8192).min(value.len());
            while !value.is_char_boundary(end) {
                end -= 1;
            }
            out.push_str(&value[start..end]);
            start = end;
            std::thread::yield_now();
        }
        Ok(())
    }
    fn metadata(&mut self, metadata: &mut Metadata) -> Result<(), String> {
        for value in [
            &mut metadata.title,
            &mut metadata.author,
            &mut metadata.email,
            &mut metadata.date,
            &mut metadata.language,
        ]
        .into_iter()
        .flatten()
        {
            let mut result = String::new();
            self.expand(value, &mut result)?;
            *value = result;
        }
        Ok(())
    }
}
fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Org export preparation cancelled or expired".into())
    } else {
        Ok(())
    }
}
