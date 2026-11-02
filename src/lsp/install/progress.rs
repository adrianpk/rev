use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Idle,
    Checking,
    Resolving,
    Acquiring,
    Verifying,
    Publishing,
    Ready,
    Failed,
    Cancelled,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Checking => "checking",
            Self::Resolving => "resolving",
            Self::Acquiring => "acquiring",
            Self::Verifying => "verifying",
            Self::Publishing => "publishing",
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct Progress(Arc<AtomicU8>);

impl Progress {
    pub fn set(&self, phase: Phase) {
        self.0.store(phase as u8, Ordering::Release);
    }
    pub fn phase(&self) -> Phase {
        match self.0.load(Ordering::Acquire) {
            0 => Phase::Idle,
            1 => Phase::Checking,
            2 => Phase::Resolving,
            3 => Phase::Acquiring,
            4 => Phase::Verifying,
            5 => Phase::Publishing,
            6 => Phase::Ready,
            7 => Phase::Failed,
            8 => Phase::Cancelled,
            _ => unreachable!("only admitted management phases are stored"),
        }
    }
}
