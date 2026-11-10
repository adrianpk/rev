use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Cooperative cancellation, checked between bounded compute or IO steps.
#[derive(Clone, Default)]
pub struct Cancellation {
    flag: Arc<AtomicBool>,
    deadline: Option<std::time::Instant>,
}

impl Cancellation {
    pub(crate) fn shared(flag: Arc<AtomicBool>, deadline: std::time::Instant) -> Self {
        Self {
            flag,
            deadline: Some(deadline),
        }
    }

    pub(crate) fn bounded(&self, deadline: std::time::Instant) -> Self {
        Self {
            flag: Arc::clone(&self.flag),
            deadline: Some(self.deadline.map_or(deadline, |old| old.min(deadline))),
        }
    }
    pub(crate) fn expired(&self) -> bool {
        self.deadline
            .is_some_and(|end| std::time::Instant::now() >= end)
    }
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Acquire) || self.expired()
    }
}
