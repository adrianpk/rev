use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub const SESSION_BYTES: usize = 33554432;
pub const SCRATCH_BYTES: usize = 1048576;

/// Reservations cover owned capacities and conservative metadata before launch
/// or publication. External Arc readers retain their reservation after refresh.
#[derive(Clone, Debug, Default)]
pub(super) struct Budget(Arc<AtomicUsize>);

#[derive(Debug)]
pub(super) struct Lease {
    budget: Budget,
    bytes: usize,
}

impl Budget {
    pub fn used(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    pub fn reserve(&self, bytes: usize) -> Result<Lease, String> {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes)
                    .filter(|next| *next <= SESSION_BYTES)
            })
            .map_err(|_| "Git session admission exceeds 32 MiB; retained state was preserved")?;
        Ok(Lease {
            budget: self.clone(),
            bytes,
        })
    }
}

impl Lease {
    pub fn grow(&mut self, bytes: usize) -> Result<(), String> {
        let mut extra = self.budget.reserve(bytes)?;
        self.bytes += extra.bytes;
        extra.bytes = 0;
        Ok(())
    }
    pub fn split(&mut self, bytes: usize) -> Self {
        assert!(
            bytes <= self.bytes,
            "Git reservation is split within its bound"
        );
        self.bytes -= bytes;
        Self {
            budget: self.budget.clone(),
            bytes,
        }
    }
    pub fn retain(&mut self, bytes: usize) {
        assert!(
            bytes <= self.bytes,
            "Git publication fits its prior reservation"
        );
        self.budget
            .0
            .fetch_sub(self.bytes - bytes, Ordering::AcqRel);
        self.bytes = bytes;
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.budget.0.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
