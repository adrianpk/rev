use std::sync::{Arc, Mutex};

use crate::buffer::Buffer;

use super::Checkpoints;

/// One native step shares this allowance across cold source projections.
pub(crate) struct Budget {
    pub(crate) remaining: usize,
}

impl Budget {
    pub(crate) const GLYPHS: usize = 4096;

    pub(crate) fn new() -> Self {
        Self {
            remaining: Self::GLYPHS,
        }
    }

    pub(crate) fn charge(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.remaining -= 1;
        true
    }
}

pub(crate) type Work = Arc<Mutex<Budget>>;

pub(crate) struct Scope {
    cache: Arc<Mutex<Checkpoints>>,
    work: Work,
    owner: bool,
}

impl Scope {
    pub(crate) fn new(buffer: &Buffer, work: Work) -> Self {
        let cache = std::sync::Arc::clone(&buffer.coordinates);
        let (work, owner) = {
            let mut entries = cache.lock().expect("coordinate cache poisoned");
            let selected = if let Some(current) = &entries.work {
                (std::sync::Arc::clone(current), false)
            } else {
                entries.work = Some(std::sync::Arc::clone(&work));
                (work, true)
            };
            drop(entries);
            selected
        };
        Self { cache, work, owner }
    }

    pub(crate) fn work(&self) -> Work {
        std::sync::Arc::clone(&self.work)
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.owner {
            self.cache.lock().expect("coordinate cache poisoned").work = None;
        }
    }
}
