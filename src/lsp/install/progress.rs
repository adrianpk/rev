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
    Building,
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
            Self::Building => "building",
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
        const PHASES: [Phase; 10] = [
            Phase::Idle,
            Phase::Checking,
            Phase::Resolving,
            Phase::Acquiring,
            Phase::Building,
            Phase::Verifying,
            Phase::Publishing,
            Phase::Ready,
            Phase::Failed,
            Phase::Cancelled,
        ];
        *PHASES
            .get(self.0.load(Ordering::Acquire) as usize)
            .expect("only admitted management phases are stored")
    }
}
