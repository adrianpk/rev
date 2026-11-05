//! Explicit manual classifications; no command runs a build or infers a verdict.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BisectAction {
    Good,
    Bad,
    Skip,
    Reset,
}
impl BisectAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Good => "good",
            Self::Bad => "bad",
            Self::Skip => "skip",
            Self::Reset => "reset",
        }
    }
}
#[derive(Debug)]
pub struct BisectPlan {
    pub refs: Arc<Refs>,
    pub bad: ObjectId,
    pub good: ObjectId,
    pub(in crate::git) _admission: Lease,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BisectResult {
    Searching,
    Found(ObjectId),
    Ambiguous,
}
impl BisectResult {
    pub fn label(self) -> &'static str {
        match self {
            Self::Searching => "awaiting explicit candidate classification",
            Self::Found(_) => "unique first bad commit verified",
            Self::Ambiguous => "ambiguous because of skipped candidates",
        }
    }
}
#[derive(Debug)]
pub struct BisectState {
    pub plan: Option<Arc<BisectPlan>>,
    pub bad: ObjectId,
    pub good: Vec<ObjectId>,
    pub skipped: Vec<ObjectId>,
    pub remaining: usize,
    pub result: BisectResult,
}
impl BisectState {
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.good.capacity() + self.skipped.capacity()) * std::mem::size_of::<ObjectId>()
    }
    pub(in crate::git) fn same(&self, other: &Self) -> bool {
        self.bad == other.bad
            && self.good == other.good
            && self.skipped == other.skipped
            && self.remaining == other.remaining
            && self.result == other.result
            && match (&self.plan, &other.plan) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
impl Sequence {
    /// Exact native original context is required even when Git labels an
    /// external operation as bisect. Classification always targets current HEAD.
    pub fn bisect_admission(&self, action: BisectAction) -> Result<(), String> {
        if self.family != Some(SequenceFamily::Bisect) {
            return Err("Manual classification requires an actual compatible bisect family".into());
        }
        let state = self
            .bisect
            .as_ref()
            .ok_or("Bisect uses unsupported terms, paths or checkout mode")?;
        let plan = state.plan.as_ref().ok_or("Bisect original context lineage is unavailable; it cannot be inferred from a branch label")?;
        if self.original != plan.refs.head
            || self.branch != plan.refs.head_ref
            || self.refs.head.is_none()
            || self
                .refs
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err(
                "Manual bisect requires its exact original context and complete clean index/disk"
                    .into(),
            );
        }
        if action != BisectAction::Reset && state.result != BisectResult::Searching {
            return Err("Bisect has no unclassified current candidate; explicitly reset the retained result".into());
        }
        if matches!(action, BisectAction::Good | BisectAction::Skip)
            && state.good.len() + state.skipped.len() + 1 >= 256
        {
            return Err("Bisect classification exceeds 256-ref admission".into());
        }
        Ok(())
    }
}
pub(super) enum Request {
    Prepare {
        refs: Arc<Refs>,
        bad: String,
        good: String,
        _admission: Lease,
    },
    Start(Arc<BisectPlan>),
    Control {
        sequence: Arc<Sequence>,
        action: BisectAction,
    },
}
impl Repositories {
    /// Resolve explicit boundaries once, retaining full IDs for final review.
    pub fn prepare_bisect(
        &mut self,
        refs: Arc<Refs>,
        bad: &str,
        good: &str,
    ) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        for boundary in [bad, good] {
            if boundary.is_empty() || boundary.len() > 4000 || boundary.contains('\0') {
                return Err("Bisect boundary must contain 1..4000 bytes without NUL".into());
            }
        }
        let root = self.mutation_identity_root(&refs.observation.identity)?;
        let admission = self.budget.reserve(bad.len() + good.len() + 4096)?;
        self.start(
            root,
            Some(refs.observation.context),
            work::Request::Bisect(Request::Prepare {
                refs,
                bad: bad.into(),
                good: good.into(),
                _admission: admission,
            }),
        )
    }
    pub fn start_bisect(&mut self, plan: Arc<BisectPlan>) -> Result<OperationId, String> {
        self.check_refs(&plan.refs)?;
        if !self
            .context(plan.refs.observation.context)
            .and_then(|c| c.bisect_plan.as_ref())
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Bisect plan expired; prepare its exact boundaries again".into());
        }
        let root = self.mutation_identity_root(&plan.refs.observation.identity)?;
        self.start(
            root,
            Some(plan.refs.observation.context),
            work::Request::Bisect(Request::Start(plan)),
        )
    }
    pub fn classify_bisect(
        &mut self,
        sequence: Arc<Sequence>,
        action: BisectAction,
    ) -> Result<OperationId, String> {
        self.check_sequence(&sequence)?;
        sequence.bisect_admission(action)?;
        let root = self.mutation_identity_root(&sequence.refs.observation.identity)?;
        self.start(
            root,
            Some(sequence.refs.observation.context),
            work::Request::Bisect(Request::Control { sequence, action }),
        )
    }
}
