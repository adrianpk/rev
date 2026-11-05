//! Prepared native Git arguments retain exact input and checked destinations.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Clone,
    CloneShallow,
    CloneBare,
    CloneMirror,
    Init,
    SubmoduleAdd,
    SubmoduleList,
    SubmoduleUpdate,
    SubmoduleSync,
    SubmoduleDeinit,
}
impl Family {
    pub fn label(self) -> &'static str {
        match self {
            Self::Clone => "regular clone",
            Self::CloneShallow => "shallow clone",
            Self::CloneBare => "bare clone",
            Self::CloneMirror => "mirror clone",
            Self::Init => "repository init",
            Self::SubmoduleAdd => "submodule add",
            Self::SubmoduleList => "submodule list",
            Self::SubmoduleUpdate => "submodule update",
            Self::SubmoduleSync => "submodule sync",
            Self::SubmoduleDeinit => "submodule deinit",
        }
    }
    pub fn seed(self) -> Vec<String> {
        let args: &[&str] = match self {
            Self::Clone => &["clone", "--", "", ""],
            Self::CloneShallow => &["clone", "--depth=1", "--", "", ""],
            Self::CloneBare => &["clone", "--bare", "--", "", ""],
            Self::CloneMirror => &["clone", "--mirror", "--", "", ""],
            Self::Init => &["init", "--", ""],
            Self::SubmoduleAdd => &["submodule", "add", "--name", "", "--", "", ""],
            Self::SubmoduleList => &["submodule", "status", "--recursive"],
            Self::SubmoduleUpdate => &["submodule", "update", "--init", "--checkout", "--", ""],
            Self::SubmoduleSync => &["submodule", "sync", "--", ""],
            Self::SubmoduleDeinit => &["submodule", "deinit", "--", ""],
        };
        args.iter().map(|a| (*a).into()).collect()
    }
    pub(super) fn validate(self, args: &[OsString]) -> Result<(), String> {
        let expected = self.seed();
        if args.len() != expected.len()
            || args
                .iter()
                .zip(&expected)
                .any(|(a, b)| !b.is_empty() && a != b.as_str())
        {
            return Err(
                "Keep the prepared family and its explicit option/parameter structure".into(),
            );
        }
        if args
            .iter()
            .any(|a| a.is_empty() || a.as_encoded_bytes().contains(&0))
        {
            return Err("Fill every prepared argument with nonempty, non-NUL input".into());
        }
        Ok(())
    }
    pub fn repository_action(self) -> bool {
        matches!(
            self,
            Self::SubmoduleAdd
                | Self::SubmoduleList
                | Self::SubmoduleUpdate
                | Self::SubmoduleSync
                | Self::SubmoduleDeinit
        )
    }
    pub(super) fn transport(self) -> bool {
        matches!(
            self,
            Self::Clone
                | Self::CloneShallow
                | Self::CloneBare
                | Self::CloneMirror
                | Self::SubmoduleAdd
                | Self::SubmoduleUpdate
        )
    }
}

pub(super) struct Submodule {
    pub configuration: Vec<u8>,
    pub name: Option<OsString>,
    pub object: Option<ObjectId>,
    pub url: Vec<u8>,
    pub child: Option<Arc<Sequence>>,
    pub evidence: [Vec<u8>; 3],
}
impl std::fmt::Debug for Submodule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Submodule")
            .field("name", &self.name)
            .field("object", &self.object)
            .finish_non_exhaustive()
    }
}
impl Submodule {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self.name.as_ref().map_or(0, OsString::capacity)
            + self.url.capacity()
            + self.evidence.iter().map(Vec::capacity).sum::<usize>()
    }
    pub fn same(&self, other: &Self) -> bool {
        self.configuration == other.configuration
            && self.name == other.name
            && self.object == other.object
            && self.url == other.url
            && self.evidence == other.evidence
            && match (&self.child, &other.child) {
                (Some(a), Some(b)) => a.same(b),
                (None, None) => true,
                _ => false,
            }
    }
}

/// Canonical preparation is read-only; only execute_family authorizes effects.
#[derive(Debug)]
pub struct FamilyPlan {
    pub family: Family,
    pub directory: PathBuf,
    pub program: String,
    pub args: Vec<OsString>,
    pub destination: PathBuf,
    pub observation: Option<Arc<Observation>>,
    pub(super) source: Option<Arc<Sequence>>,
    pub(super) evidence: [Vec<u8>; 3],
    pub(super) directory_stamp: [u64; 8],
    pub(super) parent_stamp: Option<[u64; 8]>,
    pub(super) submodule: Option<Submodule>,
    pub(super) _admission: Lease,
}
#[derive(Debug)]
pub struct FamilyResult {
    pub plan: Arc<FamilyPlan>,
    pub created: Option<Identity>,
    pub(super) source: Option<Observation>,
    pub(super) _admission: Lease,
}
pub(super) enum Request {
    Prepare {
        family: Family,
        args: Vec<OsString>,
        observation: Option<Arc<Observation>>,
        _admission: Lease,
    },
    Execute(Arc<FamilyPlan>),
}
impl Repositories {
    pub fn family_plan(&self) -> Option<&Arc<FamilyPlan>> {
        self.family_plan.as_ref()
    }
    pub fn family_result(&self) -> Option<&Arc<FamilyResult>> {
        self.family_result.as_ref()
    }
    fn family_idle(&self) -> Result<(), String> {
        if self.progress().next().is_some() {
            return Err("Prepared command waits for owned Git work and common storage".into());
        }
        Ok(())
    }
    pub fn prepare_family(
        &mut self,
        directory: &Path,
        context: Option<ContextId>,
        family: Family,
        args: Vec<OsString>,
    ) -> Result<OperationId, String> {
        self.family_idle()?;
        if !directory.is_absolute() || directory.as_os_str().len() > 4096 {
            return Err("Prepared directory requires an absolute path within 4096 bytes".into());
        }
        if args.len() > 64
            || args.iter().any(|a| a.len() > 4096)
            || args.iter().map(|a| a.len()).sum::<usize>() > 65536
        {
            return Err(
                "Prepared input exceeds 64 arguments, 4096 bytes per argument or 64 KiB total"
                    .into(),
            );
        }
        family.validate(&args)?;
        let observation = context
            .map(|id| {
                self.context(id)
                    .map(|c| c.observation.clone())
                    .ok_or("Prepared source context expired")
            })
            .transpose()?;
        if family.repository_action()
            && observation
                .as_ref()
                .is_none_or(|o| o.identity.worktree.is_none())
        {
            return Err("Submodule preparation requires a checked working-tree context".into());
        }
        let size = args.iter().map(OsString::capacity).sum::<usize>() + 8192;
        let admission = self.budget.reserve(size)?;
        self.family_plan = None;
        self.start(
            directory.into(),
            context,
            work::Request::Family(Request::Prepare {
                family,
                args,
                observation,
                _admission: admission,
            }),
        )
    }
    pub fn execute_family(&mut self, plan: Arc<FamilyPlan>) -> Result<OperationId, String> {
        self.family_idle()?;
        if !self
            .family_plan
            .as_ref()
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Prepared command review expired".into());
        }
        if let Some(o) = &plan.observation
            && !self
                .context(o.context)
                .is_some_and(|c| Arc::ptr_eq(&c.observation, o) && c.failure.is_none())
        {
            return Err("Prepared command source observation changed".into());
        }
        self.start(
            plan.directory.clone(),
            plan.observation.as_ref().map(|o| o.context),
            work::Request::Family(Request::Execute(plan)),
        )
    }
}
