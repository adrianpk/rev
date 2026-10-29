use super::*;
use crate::{
    buffer::edit::Replacement,
    org::agenda::{AgendaSource, SourceVersion},
};

pub struct Plan {
    pub destination: AgendaSource,
    pub subtree: Subtree,
    pub copy: bool,
    at: usize,
    body: String,
}
impl Targets {
    pub fn plan(
        &self,
        index: usize,
        subtree: Subtree,
        copy: bool,
        cancel: &Cancellation,
    ) -> Result<Plan, String> {
        let target = self
            .headings
            .get(index)
            .ok_or("Refile target is unavailable")?;
        let destination = self
            .sources
            .documents
            .get(target.document)
            .ok_or("Refile destination is unavailable")?
            .clone();
        let text = destination.snapshot();
        let same = matches!(destination.version, SourceVersion::Buffer(stamp) if stamp.buffer == subtree.stamp().buffer);
        if same {
            if destination.version != SourceVersion::Buffer(subtree.stamp()) {
                return Err("Refile source observation changed; discover targets again".into());
            }
            let start = text.char_to_byte(subtree.source.range.start);
            let end = text.char_to_byte(subtree.source.range.end);
            if (start..end).contains(&target.bytes.start) {
                return Err("Cannot refile a subtree into itself or its descendants".into());
            }
        }
        let lf = text.len_lines() - 1;
        let crlf = text
            .chars()
            .zip(text.chars().skip(1))
            .filter(|(a, b)| *a == '\r' && *b == '\n')
            .count();
        let newline = if crlf > lf - crlf { "\r\n" } else { "\n" };
        let mut body = crate::org::structure::entry::relevel(
            &subtree.text,
            target.level + 1,
            newline,
            cancel,
        )?;
        if !body.ends_with('\n') {
            body.push_str(newline);
        }
        Ok(Plan {
            destination,
            subtree,
            copy,
            at: target.bytes.end,
            body,
        })
    }
}
impl Plan {
    /// Recheck both actual owners before qualifying either side. The caller
    /// still admits all histories, live points and viewports before publishing.
    pub fn edits(&self, source: &Buffer, destination: &Buffer) -> Result<Vec<Replacement>, String> {
        self.subtree.current(source)?;
        self.check_destination(destination)?;
        if let Some(at) = self.replacement_at(source, destination)? {
            return Ok(vec![Replacement {
                source: self.subtree.source.clone(),
                text: self.payload(destination, at)?,
            }]);
        }
        let at = self.at;
        let text = self.payload(destination, at)?;
        let mut edits = vec![Replacement {
            source: destination.byte_span(at..at).map_err(|e| e.to_string())?,
            text,
        }];
        if !self.copy {
            edits.push(Replacement {
                source: self.subtree.source.clone(),
                text: String::new(),
            });
        }
        edits.sort_by_key(|e| (e.source.stamp.buffer, e.source.range.start));
        Ok(edits)
    }
    fn replacement_at(
        &self,
        source: &Buffer,
        destination: &Buffer,
    ) -> Result<Option<usize>, String> {
        if source.id() != destination.id() || self.copy {
            return Ok(None);
        }
        let start = source
            .slice(
                &source
                    .span(0..self.subtree.source.range.start)
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?
            .len_bytes();
        let end = start + self.subtree.text.len_bytes();
        Ok((self.at == start || self.at == end).then_some(start))
    }
    fn check_destination(&self, buffer: &Buffer) -> Result<(), String> {
        let stamp = match self.destination.version {
            SourceVersion::Buffer(stamp) => stamp == buffer.stamp(),
            _ => true,
        };
        let span = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        if !stamp
            || buffer.file().map(|f| f.path()) != self.destination.path.as_deref()
            || buffer.has_bom() != self.destination.bom
            || buffer.slice(&span).map_err(|e| e.to_string())? != self.destination.text()
        {
            return Err("Refile destination changed; nothing moved".into());
        }
        Ok(())
    }
    fn payload(&self, buffer: &Buffer, at: usize) -> Result<String, String> {
        let before = buffer.byte_span(0..at).map_err(|e| e.to_string())?;
        let before = buffer.slice(&before).map_err(|e| e.to_string())?;
        let mut text = String::new();
        if at > 0 && before.char(before.len_chars() - 1) != '\n' {
            text.push_str(if self.body.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            });
        }
        text.push_str(&self.body);
        Ok(text)
    }
}

impl Plan {
    pub(crate) fn visit(
        &self,
        buffers: &crate::buffer::Buffers,
    ) -> Result<crate::buffer::Visit, String> {
        match &self.destination.version {
            SourceVersion::Buffer(stamp) => {
                let buffer = buffers.get(stamp.buffer).map_err(|e| e.to_string())?;
                self.check_destination(buffer)?;
                Ok(crate::buffer::Visit::Existing(stamp.buffer))
            }
            SourceVersion::Disk(observed) => buffers
                .prepare_observed_visit(
                    self.destination
                        .path
                        .as_deref()
                        .ok_or("Refile disk target has no path")?,
                    crate::file::Loaded::from_snapshot(
                        self.destination.snapshot(),
                        self.destination.bom,
                    ),
                    observed,
                )
                .map_err(|e| e.to_string()),
        }
    }
}
