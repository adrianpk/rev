use super::inbox::{QUEUE_BYTES, QUEUE_MESSAGES};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(super) struct Budget(Mutex<(usize, usize)>);

/// One permit follows an observation into the encoded writer queue, preventing
/// double buffering from multiplying the connection's message and byte limits.
pub(super) struct Permit {
    budget: Arc<Budget>,
    bytes: usize,
}

impl Budget {
    pub fn reserve(self: &Arc<Self>, bytes: usize) -> Result<Permit, String> {
        let mut used = self.0.lock().expect("LSP budget lock");
        if used.0 >= QUEUE_MESSAGES || used.1.saturating_add(bytes) > QUEUE_BYTES {
            return Err("LSP outbound queue limit exceeded".into());
        }
        used.0 += 1;
        used.1 += bytes;
        drop(used);
        Ok(Permit {
            budget: Arc::clone(self),
            bytes,
        })
    }
}

impl Permit {
    pub fn resize(&mut self, bytes: usize) -> Result<(), String> {
        let mut used = self.budget.0.lock().expect("LSP budget lock");
        let total = (used.1 - self.bytes).saturating_add(bytes);
        if total > QUEUE_BYTES {
            return Err("LSP outbound byte queue limit exceeded".into());
        }
        used.1 = total;
        drop(used);
        self.bytes = bytes;
        Ok(())
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        let mut used = self.budget.0.lock().expect("LSP budget lock");
        used.0 -= 1;
        used.1 -= self.bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Observation-to-frame transfer keeps one permit. Message count and byte
    // admission remain bounded across both stages, including failed expansion.
    #[test]
    fn shared_queue_limits() {
        let budget = Arc::new(Budget::default());
        let mut permits = Vec::new();
        for _ in 0..QUEUE_MESSAGES {
            permits.push(budget.reserve(0).unwrap());
        }
        assert!(budget.reserve(0).is_err());
        permits[0].resize(QUEUE_BYTES).unwrap();
        assert!(permits[1].resize(1).is_err());
        permits[0].resize(QUEUE_BYTES / 2).unwrap();
        permits[1].resize(QUEUE_BYTES / 2).unwrap();
        drop(permits);
        assert!(budget.reserve(QUEUE_BYTES).is_ok());
    }
}
