//! Revalidate one retained destination and prepare one bounded source insertion.
mod entry;
mod placement;
use super::*;
use crate::{
    buffer::{Buffer, Buffers},
    config::{Newline, Settings},
    org::{Cancellation, Source, io::DestinationSource, time::Date},
};
use ropey::Rope;

pub struct Finalize {
    discovery: Discovery,
    expected: DestinationSource,
    draft: Rope,
    newline: Newline,
    seconds: u64,
}
/// Source evidence and an insertion in its exact byte coordinates. This is not
/// edit authority; publication must still check the live canonical owner/file.
pub struct Insertion {
    pub source: DestinationSource,
    pub edits: Vec<CaptureEdit>,
}
pub struct CaptureEdit {
    pub bytes: std::ops::Range<usize>,
    pub text: String,
}
impl Resolved {
    pub fn finalize(
        &self,
        buffers: &Buffers,
        draft: &Buffer,
        settings: &Settings,
        date: Date,
    ) -> Result<Finalize, String> {
        if !draft.is_transient() || draft.len_bytes() == 0 || draft.len_bytes() > EXPANSION_BYTES {
            return Err("Capture requires a nonempty transient draft of at most 1 MiB".into());
        }
        let choice = Choice {
            template: self.template.clone(),
            file: Some(self.source.path.clone()),
        };
        Ok(Finalize {
            discovery: choice.discover(buffers, &settings.org, date)?,
            expected: self.source.clone(),
            draft: Source::new(draft).snapshot().map_err(|e| e.to_string())?,
            newline: settings.files.newline,
            seconds: settings.org.limits.job_seconds as u64,
        })
    }
}
impl Finalize {
    pub fn run(self, cancel: &Cancellation) -> Result<Insertion, String> {
        let bounded = cancel
            .bounded(std::time::Instant::now() + std::time::Duration::from_secs(self.seconds));
        let cancel = &bounded;
        let current = self.discovery.run(cancel)?;
        same_source(&self.expected, &current.source)?;
        if current.source.readonly {
            return Err("Capture destination buffer is read-only; draft retained".into());
        }
        let newline = newline(&current.source, self.newline);
        let edits = placement::insert(&current, &self.draft, newline, cancel)?;
        if cancel.is_cancelled() {
            return Err("Capture finalization cancelled".into());
        }
        Ok(Insertion {
            source: current.source,
            edits,
        })
    }
}
fn same_source(expected: &DestinationSource, current: &DestinationSource) -> Result<(), String> {
    if expected.path != current.path
        || expected.version != current.version
        || expected.bom != current.bom
        || expected.text() != current.text()
    {
        return Err(
            "Capture destination changed; draft retained; restart capture against current source"
                .into(),
        );
    }
    Ok(())
}
fn newline(source: &DestinationSource, preferred: Newline) -> &'static str {
    let text = source.text();
    let lf = text.len_lines() - 1;
    let crlf = text
        .chars()
        .zip(text.chars().skip(1))
        .filter(|(a, b)| *a == '\r' && *b == '\n')
        .count();
    if crlf > lf - crlf || lf == 0 && matches!(preferred, Newline::Crlf) {
        "\r\n"
    } else {
        "\n"
    }
}

impl Insertion {
    /// Check semantic source bytes and format as well as actual buffer identity
    /// before qualifying offsets for the ordinary edit owner.
    pub fn replacements(
        &self,
        buffer: &Buffer,
    ) -> Result<Vec<crate::buffer::edit::Replacement>, String> {
        let whole = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        let text = buffer.slice(&whole).map_err(|e| e.to_string())?;
        let same_stamp = match self.source.version {
            crate::org::io::DestinationVersion::Buffer(stamp) => buffer.stamp() == stamp,
            _ => true,
        };
        let bom = matches!(
            self.source.version,
            crate::org::io::DestinationVersion::Missing
        ) || buffer.has_bom() == self.source.bom;
        if !same_stamp
            || !bom
            || text != self.source.text()
            || buffer.file().map(|f| f.path()) != Some(self.source.path.as_path())
        {
            return Err("Capture canonical destination differs from validated source".into());
        }
        self.edits
            .iter()
            .map(|edit| {
                Ok(crate::buffer::edit::Replacement {
                    source: buffer
                        .byte_span(edit.bytes.clone())
                        .map_err(|e| e.to_string())?,
                    text: edit.text.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "finalize/tests.rs"]
mod tests;
