use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutosquashAction {
    Pick,
    Fixup,
    Squash,
    ReplaceMessage,
}
impl AutosquashAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pick => "pick",
            Self::Fixup => "fixup",
            Self::Squash => "squash",
            Self::ReplaceMessage => "fixup -C",
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct AutosquashStep {
    pub action: AutosquashAction,
    pub object: ObjectId,
    pub subject: String,
}
/// Bounded review prepared without starting a sequence. Private admission and
/// exact worker reconstruction bind every ordered action to the captured refs.
#[derive(Debug)]
pub struct AutosquashPlan {
    pub refs: Arc<Refs>,
    pub target_name: Vec<u8>,
    pub target: ObjectId,
    pub base: ObjectId,
    pub steps: Vec<AutosquashStep>,
    pub(in crate::git) _admission: Lease,
}
impl AutosquashPlan {
    pub(in crate::git) fn same(&self, other: &Self) -> bool {
        self.refs.same(&other.refs)
            && self.target_name == other.target_name
            && self.target == other.target
            && self.base == other.base
            && self.steps == other.steps
    }
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.target_name.capacity()
            + self.steps.capacity() * std::mem::size_of::<AutosquashStep>()
            + self
                .steps
                .iter()
                .map(|s| s.subject.capacity())
                .sum::<usize>()
    }
}
impl Repositories {
    /// Observe a finite linear source range and prepare only non-executable
    /// autosquash actions. This does not start or modify a Git-owned sequence.
    pub fn prepare_autosquash(
        &mut self,
        refs: Arc<Refs>,
        target: &[u8],
    ) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.head.is_none()
            || refs.head_ref.is_empty()
            || refs.reference(target).and_then(Reference::commit).is_none()
        {
            return Err("Autosquash requires a checked current branch and one exact commit-bearing target ref".into());
        }
        let root = self.mutation_identity_root(&refs.observation.identity)?;
        let admission = self.budget.reserve(target.len() + 4096)?;
        self.start(
            root,
            Some(refs.observation.context),
            work::Request::Integration(Request::PrepareAutosquash {
                refs,
                target: target.into(),
                _admission: admission,
            }),
        )
    }
    /// Apply exactly one reviewed plan. Every source object and ordered action
    /// is reconstructed and compared before the changing child is launched.
    pub fn autosquash(&mut self, plan: Arc<AutosquashPlan>) -> Result<OperationId, String> {
        self.check_refs(&plan.refs)?;
        if !self
            .context(plan.refs.observation.context)
            .and_then(|c| c.autosquash.as_ref())
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Autosquash plan expired; prepare a fresh review".into());
        }
        let root = self.mutation_identity_root(&plan.refs.observation.identity)?;
        self.start(
            root,
            Some(plan.refs.observation.context),
            work::Request::Integration(Request::Autosquash(plan)),
        )
    }
}
