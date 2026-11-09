use super::*;
use crate::coordinates::Error as CoordinateError;
use std::os::unix::ffi::OsStrExt;

/// OS debug escaping preserves distinguishable labels for invalid UTF-8 and
/// controls. These labels are never reparsed into repository arguments.
pub(crate) fn label(bytes: &[u8]) -> String {
    let text = format!("{:?}", std::ffi::OsStr::from_bytes(bytes));
    text.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(&text)
        .to_owned()
}

/// Diagnostic rows borrow retained capture and allocate only the visible
/// escaped chunk. Sensitive lines are withheld in full, including URL userinfo
/// and separated authorization values; these strings are never command data.
pub(crate) struct DiagnosticLines<'a> {
    remaining: &'a [u8],
    line: &'a [u8],
}

enum DiagnosticPart<'a> {
    Bytes(&'a [u8], bool),
    Redacted,
}

impl<'a> DiagnosticLines<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            remaining: bytes,
            line: &[],
        }
    }
    fn part(&mut self) -> Option<DiagnosticPart<'a>> {
        while self.line.is_empty() {
            if self.remaining.is_empty() {
                return None;
            }
            let end = self
                .remaining
                .iter()
                .position(|b| matches!(b, b'\n' | 0))
                .unwrap_or(self.remaining.len());
            let line = &self.remaining[..end];
            self.remaining = &self.remaining[(end + 1).min(self.remaining.len())..];
            if sensitive(line) {
                return Some(DiagnosticPart::Redacted);
            }
            self.line = line;
        }
        let end = self.line.len().min(2048);
        let bytes = &self.line[..end];
        self.line = &self.line[end..];
        Some(DiagnosticPart::Bytes(bytes, !self.line.is_empty()))
    }
}

impl Iterator for DiagnosticLines<'_> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.part().map(|part| match part {
            DiagnosticPart::Redacted => "[credential-bearing diagnostic line redacted]".into(),
            DiagnosticPart::Bytes(bytes, continued) => {
                let mut text = label(bytes);
                if continued {
                    text.push_str(" [continued]");
                }
                text
            }
        })
    }
    fn nth(&mut self, n: usize) -> Option<String> {
        for _ in 0..n {
            self.part()?;
        }
        self.next()
    }
    fn count(mut self) -> usize {
        let mut count = 0;
        while self.part().is_some() {
            count += 1;
        }
        count
    }
}

fn sensitive(bytes: &[u8]) -> bool {
    bytes.contains(&b'@')
        || [
            b"://".as_slice(),
            b"token",
            b"password",
            b"passwd",
            b"authorization",
            b"credential",
            b"secret",
            b"bearer",
            b"private-key",
            b"private_key",
        ]
        .iter()
        .any(|needle| {
            bytes
                .windows(needle.len())
                .any(|part| part.eq_ignore_ascii_case(needle))
        })
}

/// View roles come from typed repository items, not from escaped labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Role {
    #[default]
    Text,
    Heading,
    Metadata,
    Warning,
    Group(crate::git::Group),
    Diff(crate::git::DiffLineKind),
}

pub(crate) struct Line {
    text: String,
    selected: bool,
    role: Role,
}

impl Line {
    pub(crate) fn new(line: (String, bool), role: Role) -> Self {
        Self {
            text: line.0,
            selected: line.1,
            role,
        }
    }
}

impl From<(String, bool)> for Line {
    fn from(line: (String, bool)) -> Self {
        Self::new(line, Role::Text)
    }
}

impl Role {
    pub(super) fn foreground(
        self,
        palette: &crate::config::Palette,
        selected: bool,
    ) -> crate::config::Rgb {
        use crate::git::DiffLineKind as Kind;
        match self {
            Self::Text | Self::Diff(Kind::Context) => {
                if selected {
                    palette.tree_selected_text
                } else {
                    palette.foreground
                }
            }
            Self::Heading | Self::Diff(Kind::Header) => palette.git.heading,
            Self::Metadata | Self::Diff(Kind::Metadata) => palette.git.metadata,
            Self::Warning => palette.warning,
            Self::Group(group) => group_color(group, &palette.git),
            Self::Diff(kind) => diff_color(kind, &palette.git),
        }
    }
}

fn group_color(
    group: crate::git::Group,
    palette: &crate::config::GitPalette,
) -> crate::config::Rgb {
    use crate::git::Group;
    match group {
        Group::Untracked => palette.untracked,
        Group::Unstaged => palette.unstaged,
        Group::Staged => palette.staged,
        Group::Unmerged => palette.conflict,
    }
}

fn diff_color(
    kind: crate::git::DiffLineKind,
    palette: &crate::config::GitPalette,
) -> crate::config::Rgb {
    use crate::git::DiffLineKind as Kind;
    match kind {
        Kind::Added => palette.added,
        Kind::Removed => palette.removed,
        Kind::Hunk => palette.hunk,
        Kind::Header => palette.heading,
        Kind::Metadata => palette.metadata,
        Kind::Context => unreachable!("context uses the selected document foreground"),
    }
}

impl Frame {
    pub(crate) fn git_popup(
        &mut self,
        composition: &crate::panel::Composition,
    ) -> Result<(), Error> {
        for border in composition.borders() {
            self.projection_text("", *border)?;
        }
        Ok(())
    }

    pub(crate) fn git<L: Into<Line>>(
        mut self,
        echo: &str,
        footer: &str,
        admission: crate::git::budget::Lease,
        mut line: impl FnMut(usize) -> L,
    ) -> Result<Self, Error> {
        let size = self.size;
        self._git_admission = Some(admission);
        if size.width == 0 || size.height == 0 {
            return Ok(self);
        }
        self.rows.pop();
        self.echo(echo)?;
        for row in 0..size.height.saturating_sub(2) {
            let Line {
                text,
                selected,
                role,
            } = line(usize::from(row)).into();
            let area = Area::new(0, row, size.width, 1).ok_or(CoordinateError::Overflow)?;
            self.projection_text(&text, area)?;
            self.rows[usize::from(row)].tone = Tone::Git(role, selected);
            // The Git role owns this full-width row; projection faces otherwise
            // override its foreground with the ordinary document color.
            self.rows[usize::from(row)].faces.clear();
        }
        self.git_footer(footer)?;
        self.cursor = None;
        Ok(self)
    }
    fn git_footer(&mut self, footer: &str) -> Result<(), Error> {
        if self.size.height >= 2 {
            let row = self.size.height - 2;
            self.blit(
                Area::new(0, row, self.size.width, 1).ok_or(CoordinateError::Overflow)?,
                Row {
                    row: 0,
                    text: clip::line(Glyphs::for_text(footer)?, 0, self.size.width)?,
                    band: Band::Status,
                    tone: Tone::Status(true),
                    faces: Vec::new(),
                    syntax: Vec::new(),
                    selection: None,
                    search: None,
                },
            )?;
        }
        Ok(())
    }
}
