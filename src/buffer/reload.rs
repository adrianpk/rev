//! Revision-bound reload admission through the existing native edit/history owner.
use std::io;
use std::time::Instant;

use super::{
    Buffer, Stamp, State,
    edit::{Point, Prepared},
};
use crate::file::{self, Location};

/// IO-free capture; move it to the sole reload worker before reading disk.
pub struct Request {
    stamp: Stamp,
    state: State,
    prior: Location,
    dirty: bool,
    max_bytes: usize,
}

/// Private complete disk image. Discard consent refers to precisely this image.
pub struct Reload {
    request: Request,
    image: file::reload::Image,
}

/// A recently rechecked image, consumed once by native preparation/publication.
pub struct Checked {
    reload: Reload,
    checked: Instant,
}

/// Native edit and clean-state evidence; neither is installed until publication.
pub struct Publication {
    pub(crate) edit: Option<Prepared>,
    request: Request,
    observed: file::Observation,
    checked: Instant,
}

impl Buffer {
    /// Capture only an ordinary visiting buffer. This performs no disk access.
    pub fn request_reload(&self, max_bytes: usize) -> io::Result<Request> {
        if self.transient {
            return Err(io::Error::other(
                "transient buffers cannot reload a source file",
            ));
        }
        let prior = self
            .file
            .clone()
            .ok_or_else(|| io::Error::other("buffer has no visiting file"))?;
        if !(1..=134_217_728).contains(&max_bytes) {
            return Err(io::Error::other(
                "reload byte limit is outside the admitted range",
            ));
        }
        Ok(Request {
            stamp: self.stamp(),
            state: self.state,
            prior,
            dirty: self.is_modified(),
            max_bytes,
        })
    }

    /// Prepare the one native edit with current shared points, without disk IO.
    pub fn prepare_reload(&self, candidate: Checked, points: &[Point]) -> io::Result<Publication> {
        file::reload::deadline(candidate.checked)?;
        candidate.reload.request.current(self)?;
        let (text, bom) = candidate.reload.image.loaded.into_parts();
        let edit = if self.text == text && self.bom == bom {
            None
        } else {
            Some(
                self.prepare_reload_text(&text, bom, points)
                    .map_err(io::Error::other)?,
            )
        };
        Ok(Publication {
            edit,
            request: candidate.reload.request,
            observed: candidate.reload.image.observed,
            checked: candidate.checked,
        })
    }

    /// Publish atomically and acknowledge only that exact disk image as clean.
    /// Read-only policy and preexisting undo history remain owned by this buffer.
    pub fn publish_reload(
        &mut self,
        publication: Publication,
        points: &mut [Point],
    ) -> io::Result<bool> {
        file::reload::deadline(publication.checked)?;
        publication.request.current(self)?;
        let changed = publication.edit.is_some();
        if let Some(edit) = publication.edit {
            self.publish(edit, points).map_err(io::Error::other)?;
        }
        self.file = Some(
            publication
                .request
                .prior
                .published(Some(publication.observed)),
        );
        self.saved = self.state;
        self.warning = None;
        self.external_notice = None;
        self.break_typing();
        Ok(changed)
    }
}

impl Request {
    /// Run bounded complete ingress off the input/render thread.
    pub fn read(self) -> io::Result<Reload> {
        let image = file::reload::read(&self.prior, self.max_bytes)?;
        Ok(Reload {
            request: self,
            image,
        })
    }

    fn current(&self, buffer: &Buffer) -> io::Result<()> {
        buffer.current(self.stamp).map_err(io::Error::other)?;
        if buffer.state != self.state
            || buffer.file.as_ref() != Some(&self.prior)
            || buffer.is_modified() != self.dirty
        {
            return Err(io::Error::other(
                "buffer state or visiting association changed; reload canceled",
            ));
        }
        Ok(())
    }
}

impl Reload {
    pub(crate) fn matches(&self, buffer: &Buffer) -> bool {
        self.image.loaded.observed_text() == buffer.text.slice(..)
            && self.image.loaded.has_bom() == buffer.bom
    }
    pub fn buffer(&self) -> super::BufferId {
        self.request.stamp.buffer
    }
    pub fn needs_confirmation(&self) -> bool {
        self.request.dirty
    }

    /// Check in memory before prompting, so stale work never steals input.
    pub fn current(&self, buffer: &Buffer) -> io::Result<()> {
        self.request.current(buffer)
    }

    /// Run on the worker after the explicit discard answer (or clean invocation).
    /// Fresh disk changes invalidate consent instead of accepting a later image.
    pub fn verify(self, discard: bool) -> io::Result<Checked> {
        if self.request.dirty && !discard {
            return Err(io::Error::other(
                "unsaved text requires explicit discard confirmation",
            ));
        }
        let started = Instant::now();
        file::reload::check(self.request.prior.path(), &self.image.observed)?;
        file::reload::deadline(started)?;
        Ok(Checked {
            reload: self,
            checked: Instant::now(),
        })
    }
}

impl Checked {
    pub fn buffer(&self) -> super::BufferId {
        self.reload.buffer()
    }
}
