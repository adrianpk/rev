use super::*;
use std::path::Path;
impl Document {
    pub(crate) fn at(&self, point: usize) -> Option<&Link> {
        self.links.iter().find(|l| l.bytes.contains(&point))
    }
    pub(crate) fn edit(&self, point: usize) -> Edit {
        self.at(point).map_or(
            Edit {
                bytes: point..point,
                target: String::new(),
                description: String::new(),
            },
            |l| Edit {
                bytes: l.bytes.clone(),
                target: l.target.clone(),
                description: l.description.clone(),
            },
        )
    }
    pub(crate) fn motion(&self, point: usize, next: bool, wrap: bool) -> Option<usize> {
        let found = if next {
            self.links.iter().find(|l| l.bytes.start > point)
        } else {
            self.links.iter().rev().find(|l| l.bytes.start < point)
        };
        found
            .or_else(|| {
                if !wrap {
                    None
                } else if next {
                    self.links.first()
                } else {
                    self.links.last()
                }
            })
            .map(|l| l.bytes.start)
    }
    pub(crate) fn stored(&self, point: usize, path: Option<&Path>) -> Result<Stored, String> {
        let heading = self
            .headings
            .iter()
            .rev()
            .find(|h| h.byte <= point && point <= h.end);
        if let Some(heading) = heading {
            if let Some(id) = self.value(heading.byte, "ID")? {
                return Ok(Stored {
                    target: format!("id:{}", encode(&id)),
                    description: heading.title.clone(),
                });
            }
            if let Some(id) = self.value(heading.byte, "CUSTOM_ID")? {
                return self.file_link(path, &format!("#{}", encode(&id)), heading.title.clone());
            }
        }
        if let Some(anchor) = self.anchors.iter().find(|a| a.bytes.contains(&point)) {
            return self.file_link(path, &encode(&anchor.name), anchor.name.clone());
        }
        if let Some(heading) = heading {
            return self.file_link(
                path,
                &format!("*{}", encode(&heading.title)),
                heading.title.clone(),
            );
        }
        self.file_link(
            path,
            &(self.text.byte_to_line(point) + 1).to_string(),
            String::new(),
        )
    }
    fn file_link(
        &self,
        path: Option<&Path>,
        search: &str,
        description: String,
    ) -> Result<Stored, String> {
        let path = path
            .ok_or("Associate this source with a file or add an existing ID before storing a link")?
            .to_str()
            .ok_or("Link file path is not UTF-8")?;
        let target = format!("file:{}::{search}", encode(path));
        Target::parse(&target)?;
        Ok(Stored {
            target,
            description,
        })
    }
    pub(crate) fn value(&self, byte: usize, key: &str) -> Result<Option<String>, String> {
        self.properties.query_value(&self.text, byte, key, &[])
    }
    pub(crate) fn resolve(&self, search: &str, scope: Range<usize>) -> Result<usize, String> {
        if search.is_empty() {
            return Ok(scope.start);
        }
        if let Some(id) = search.strip_prefix('#') {
            return self.property_target(id, "CUSTOM_ID", scope);
        }
        if let Some(title) = search.strip_prefix('*') {
            return unique(
                self.headings
                    .iter()
                    .filter(|h| scope.contains(&h.byte) && h.title == title.trim())
                    .map(|h| h.byte),
            );
        }
        if search.bytes().all(|b| b.is_ascii_digit()) {
            return self.line(search, scope);
        }
        let anchors = self
            .anchors
            .iter()
            .filter(|a| a.name == search && scope.contains(&a.bytes.start))
            .map(|a| a.bytes.start)
            .collect::<Vec<_>>();
        if !anchors.is_empty() {
            return unique(anchors);
        }
        unique(
            self.headings
                .iter()
                .filter(|h| scope.contains(&h.byte) && h.title == search)
                .map(|h| h.byte),
        )
    }
    fn line(&self, value: &str, scope: Range<usize>) -> Result<usize, String> {
        let row = value
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .filter(|r| *r < self.text.len_lines())
            .ok_or("Link line is outside the document")?;
        let at = self.text.line_to_byte(row);
        if !scope.contains(&at) && !(at == self.text.len_bytes() && scope.end == at) {
            return Err("Link line is outside its checked scope".into());
        }
        Ok(at)
    }
    pub(crate) fn property_target(
        &self,
        id: &str,
        key: &str,
        scope: Range<usize>,
    ) -> Result<usize, String> {
        let mut matches = Vec::new();
        for heading in self.headings.iter().filter(|h| scope.contains(&h.byte)) {
            if self.value(heading.byte, key)?.as_deref() == Some(id) {
                matches.push(heading.byte);
            }
        }
        unique(matches)
    }
    pub(crate) fn id_scope(&self, id: &str) -> Result<Option<Range<usize>>, String> {
        let mut found = None;
        for heading in &self.headings {
            if self.value(heading.byte, "ID")?.as_deref() != Some(id) {
                continue;
            }
            if found.is_some() {
                return Err("Ambiguous Org ID destination".into());
            }
            found = Some(heading.byte..heading.end);
        }
        Ok(found)
    }
}

fn unique(values: impl IntoIterator<Item = usize>) -> Result<usize, String> {
    let values = values.into_iter().collect::<Vec<_>>();
    match values.as_slice() {
        [at] => Ok(*at),
        [] => Err("Unresolved Org link target".into()),
        _ => Err("Ambiguous Org link target; make its source identity unique".into()),
    }
}
