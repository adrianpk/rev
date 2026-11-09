use super::*;

/// State belongs to one immutable unified patch, including malformed evidence.
struct Parser<'a> {
    operation: OperationId,
    lines: &'a mut Vec<DiffLine>,
    hunks: &'a mut Vec<Hunk>,
    cancel: &'a crate::process::Cancellation,
    offset: usize,
    remaining: (usize, usize),
    active: Option<HunkId>,
    failure: Option<&'static str>,
}

pub(super) fn parse(
    operation: OperationId,
    bytes: &[u8],
    lines: &mut Vec<DiffLine>,
    hunks: &mut Vec<Hunk>,
    cancel: &crate::process::Cancellation,
) -> Result<(), String> {
    Parser {
        operation,
        lines,
        hunks,
        cancel,
        offset: 0,
        remaining: (0, 0),
        active: None,
        failure: None,
    }
    .parse(bytes)
}
impl Parser<'_> {
    fn parse(&mut self, bytes: &[u8]) -> Result<(), String> {
        for line in bytes.split_inclusive(|b| *b == b'\n') {
            if self.cancel.is_cancelled() {
                return Err("Git diff parsing cancelled or timed out".into());
            }
            self.line(line);
        }
        if self.remaining != (0, 0) {
            self.failure = Some("Incomplete Git unified hunk body");
        }
        if let Some(message) = self.failure {
            // Malformed evidence must never retain a changing hunk target.
            self.hunks.clear();
            for line in &mut *self.lines {
                line.hunk = None;
            }
            return Err(message.into());
        }
        Ok(())
    }
    fn line(&mut self, line: &[u8]) {
        let content = line.strip_suffix(b"\n").unwrap_or(line);
        let kind = if content.starts_with(b"@@ ") {
            self.header(content, line.len());
            DiffLineKind::Hunk
        } else if self.active.is_some()
            && (self.remaining != (0, 0) || content == b"\\ No newline at end of file")
        {
            self.body(content, line.len())
        } else {
            self.metadata(content)
        };
        for start in (0..line.len()).step_by(2048) {
            self.lines.push(DiffLine {
                bytes: self.offset + start..self.offset + (start + 2048).min(line.len()),
                hunk: self.active,
                kind,
            });
        }
        self.offset += line.len();
    }
    fn header(&mut self, content: &[u8], length: usize) {
        if self.remaining != (0, 0) {
            self.failure = Some("Git hunk body does not match its declared ranges");
        }
        match ranges(content) {
            Ok((old, new)) => {
                let id = HunkId(self.operation, self.hunks.len());
                self.remaining = (old.count, new.count);
                self.hunks.push(Hunk {
                    id,
                    old,
                    new,
                    patch: self.offset..self.offset + length,
                });
                self.active = Some(id);
            }
            Err(_) => {
                self.failure = Some("Malformed Git unified hunk ranges");
                self.active = None;
            }
        }
    }
    fn body(&mut self, content: &[u8], length: usize) -> DiffLineKind {
        let kind = match content.first() {
            Some(b'+') => DiffLineKind::Added,
            Some(b'-') => DiffLineKind::Removed,
            Some(b' ') => DiffLineKind::Context,
            _ => DiffLineKind::Metadata,
        };
        let used = self.body_usage(content);
        if used.0 > self.remaining.0 || used.1 > self.remaining.1 {
            self.failure = Some("Git hunk exceeds its declared ranges");
        }
        self.remaining = (
            self.remaining.0.saturating_sub(used.0),
            self.remaining.1.saturating_sub(used.1),
        );
        if let Some(hunk) = self.hunks.last_mut() {
            hunk.patch.end = self.offset + length;
        }
        kind
    }
    fn body_usage(&mut self, content: &[u8]) -> (usize, usize) {
        match content.first() {
            Some(b' ') => (1, 1),
            Some(b'-') => (1, 0),
            Some(b'+') => (0, 1),
            Some(b'\\') if content == b"\\ No newline at end of file" => (0, 0),
            _ => {
                self.failure = Some("Malformed Git hunk body");
                (0, 0)
            }
        }
    }
    fn metadata(&mut self, content: &[u8]) -> DiffLineKind {
        self.active = None;
        let mut kind = DiffLineKind::Metadata;
        if [b"diff ".as_slice(), b"--- ", b"+++ "]
            .iter()
            .any(|prefix| content.starts_with(prefix))
        {
            kind = DiffLineKind::Header;
        }
        // Every unified comparison comes from one native Git request;
        // unexpected extra payload must not acquire semantic text targets.
        if !content.is_empty()
            && ![
                b"diff ".as_slice(),
                b"index ",
                b"--- ",
                b"+++ ",
                b"old mode ",
                b"new mode ",
                b"new file mode ",
                b"deleted file mode ",
                b"similarity index ",
                b"dissimilarity index ",
                b"rename from ",
                b"rename to ",
                b"copy from ",
                b"copy to ",
                b"Binary files ",
                b"Submodule ",
                b"GIT binary patch",
                b"@@@ ",
            ]
            .iter()
            .any(|prefix| content.starts_with(prefix))
        {
            self.failure = Some("Unsupported Git diff payload");
        }
        kind
    }
}
