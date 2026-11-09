use super::*;

impl Request {
    pub fn kind(&self) -> OperationKind {
        match self {
            Self::Family(super::super::families::Request::Prepare { .. }) => {
                OperationKind::FamilyPrepare
            }
            Self::Family(_) => OperationKind::Family,
            Self::Repository(request) => repository_kind(request),
            Self::Resolution(request) => resolution_kind(request),
            Self::Bisect(request) => bisect_kind(request),
            Self::Application(request) => application_kind(request),
            Self::Integration(request) => integration_kind(request),
            Self::Refs(request) => refs_kind(request),
            _ => self.observation_kind(),
        }
    }
    fn observation_kind(&self) -> OperationKind {
        match self {
            Self::ConflictExplain(_) => OperationKind::ConflictExplain,
            Self::Conflict(_) => OperationKind::Conflict,
            Self::History(_) => OperationKind::History,
            Self::Status => OperationKind::Status,
            Self::Diff(_) => OperationKind::Diff,
            Self::Index(request) => request.direction.kind(),
            Self::Discard(_) => OperationKind::Discard,
            Self::Commit(_) => OperationKind::Commit,
            Self::CommitPrepare(_) => OperationKind::CommitPrepare,
            _ => unreachable!("operation kind dispatched by its family"),
        }
    }
    pub(super) fn seconds(&self, policy: &config::Git) -> usize {
        match self {
            Self::Family(request) => family_seconds(request, policy),
            Self::Refs(
                super::super::refs::Request::Fetch { .. } | super::super::refs::Request::Push(_),
            ) => policy.transport_timeout_seconds,
            _ if CHANGING.contains(&self.kind()) => policy.operation_timeout_seconds,
            _ => policy.observation_timeout_seconds,
        }
    }
    pub(super) fn private_output(&self) -> bool {
        [
            OperationKind::Fetch,
            OperationKind::PushPrepare,
            OperationKind::Push,
            OperationKind::FamilyPrepare,
            OperationKind::Family,
        ]
        .contains(&self.kind())
    }
}

fn family_seconds(request: &super::super::families::Request, policy: &config::Git) -> usize {
    use super::super::families::Request;
    match request {
        Request::Prepare { family, .. }
            if (family.subtree() && family.transport()) || *family == Family::RequestPull =>
        {
            policy.transport_timeout_seconds
        }
        Request::Execute(plan) if plan.family.direct() => {
            plan.direct_limits().expect("direct limits").0
        }
        Request::Execute(plan) if !plan.family.transport() => policy.operation_timeout_seconds,
        Request::Execute(_) => policy.transport_timeout_seconds,
        _ => policy.observation_timeout_seconds,
    }
}
fn repository_kind(request: &super::super::repository::Request) -> OperationKind {
    use super::super::repository::Request;
    match request {
        Request::Worktrees(request) => match request {
            super::super::repository::worktrees::Request::List(_) => OperationKind::Worktrees,
            super::super::repository::worktrees::Request::Visit { .. } => {
                OperationKind::WorktreeVisit
            }
            super::super::repository::worktrees::Request::Prepare { .. } => {
                OperationKind::WorktreePrepare
            }
            super::super::repository::worktrees::Request::Change(_) => OperationKind::Worktree,
        },
        Request::Stashes(_) => OperationKind::Stashes,
        Request::Show { .. } => OperationKind::StashShow,
        Request::Prepare { .. } => OperationKind::StashPrepare,
        _ => OperationKind::Stash,
    }
}
fn resolution_kind(request: &super::super::conflicts::ResolutionRequest) -> OperationKind {
    match request {
        super::super::conflicts::ResolutionRequest::Prepare { .. } => {
            OperationKind::ConflictPrepare
        }
        _ => OperationKind::ConflictResolve,
    }
}
fn bisect_kind(request: &super::super::bisect::Request) -> OperationKind {
    match request {
        super::super::bisect::Request::Prepare { .. } => OperationKind::BisectPrepare,
        _ => OperationKind::Bisect,
    }
}
fn application_kind(request: &super::super::sequences::Request) -> OperationKind {
    match request {
        super::super::sequences::Request::Prepare { .. } => OperationKind::ApplicationPrepare,
        _ => OperationKind::Application,
    }
}
fn integration_kind(request: &super::super::integration::Request) -> OperationKind {
    use super::super::integration::Request;
    match request {
        Request::Observe { .. } => OperationKind::Sequence,
        Request::PrepareAutosquash { .. } => OperationKind::AutosquashPrepare,
        Request::Control { sequence, .. } if sequence.application.is_some() => {
            OperationKind::ApplicationControl
        }
        Request::Control { .. } => OperationKind::SequenceControl,
        _ => OperationKind::Integration,
    }
}
fn refs_kind(request: &super::super::refs::Request) -> OperationKind {
    use super::super::refs::Request;
    match request {
        Request::PreparePush { .. } => OperationKind::PushPrepare,
        Request::Push(_) => OperationKind::Push,
        Request::Fetch { .. } => OperationKind::Fetch,
        Request::PrepareTag { .. } => OperationKind::TagPrepare,
        Request::CreateTag { .. } => OperationKind::TagCreate,
        Request::DeleteTag { .. } => OperationKind::TagDelete,
        Request::Observe(_) => OperationKind::Refs,
        Request::Branch { .. } => OperationKind::Branch,
    }
}
// This is the original mutation-timeout classification; preparation and visiting
// requests retain the observation timeout unless their family overrides it.
const CHANGING: &[OperationKind] = &[
    OperationKind::Worktree,
    OperationKind::Stash,
    OperationKind::Bisect,
    OperationKind::Application,
    OperationKind::Integration,
    OperationKind::SequenceControl,
    OperationKind::ApplicationControl,
    OperationKind::ConflictResolve,
    OperationKind::Stage,
    OperationKind::Unstage,
    OperationKind::Discard,
    OperationKind::Commit,
    OperationKind::Branch,
    OperationKind::TagCreate,
    OperationKind::TagDelete,
];
