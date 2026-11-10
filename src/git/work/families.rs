use super::*;
use crate::git::conflicts::resolution::fingerprint;
use crate::git::{Family, FamilyPlan, FamilyResult, families::Request};
mod creation;
mod direct;
mod notes;
mod output;
mod patch;
mod preparation;
mod sparse;
mod state;
mod submodule;
mod subtree;

impl Runner {
    pub(super) fn family(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Prepare {
                family,
                args,
                observation,
                output,
                shell,
                ..
            } => self.prepare_family(
                preparation::Preparation {
                    family,
                    args,
                    observation,
                    output,
                    shell,
                },
                admission,
            ),
            Request::Execute(plan) => self.execute_family(plan, admission),
        }
    }
    fn family_destination_stamp(path: &Path) -> Result<[u64; 8], String> {
        if !std::fs::symlink_metadata(path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            return Err("Prepared destination already exists or cannot be checked".into());
        }
        let parent = path.parent().ok_or("Prepared destination has no parent")?;
        let metadata = std::fs::symlink_metadata(parent)
            .map_err(|e| format!("Observe destination parent: {e}"))?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || std::fs::canonicalize(parent).map_err(|e| e.to_string())? != parent
        {
            return Err(
                "Prepared destination parent is not an ordinary canonical directory".into(),
            );
        }
        Ok(fingerprint(&metadata))
    }
    fn family_destination(&self, arg: &std::ffi::OsStr) -> Result<PathBuf, String> {
        let input = Path::new(arg);
        let input = if input.is_absolute() {
            input.to_owned()
        } else {
            self.directory.join(input)
        };
        let mut path = PathBuf::new();
        for part in input.components() {
            match part {
                std::path::Component::RootDir => path.push("/"),
                std::path::Component::Normal(p) => path.push(p),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir if path.pop() => {}
                _ => return Err("Prepared destination escapes its absolute root".into()),
            }
        }
        if path.file_name().is_none() || path.as_os_str().len() > 4096 {
            return Err("Prepared destination must be a non-root path within 4096 bytes".into());
        }
        Ok(path)
    }
    fn family_relative_destination(&self, arg: &std::ffi::OsStr) -> Result<PathBuf, String> {
        let relative = Path::new(arg);
        if relative
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(n) if n != ".git"))
        {
            return Err(
                "Prepared repository path requires literal relative components outside .git".into(),
            );
        }
        let path = self.directory.join(relative);
        if path.as_os_str().len() > 4096 {
            return Err("Prepared repository path exceeds 4096 bytes".into());
        }
        // Check every existing component. Git must not follow a replaced
        // symlink into an unrelated directory, including for an empty child.
        let mut current = self.directory.clone();
        for component in relative.components() {
            current.push(component);
            match std::fs::symlink_metadata(&current) {
                Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                _ => {
                    return Err("Prepared repository path is not an ordinary directory path".into());
                }
            }
        }
        Ok(path)
    }

    fn execute_family(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_family_location(&plan)?;
        self.check_family_source(&plan, admission)?;
        if plan.family.direct() {
            return self.execute_direct(plan, admission);
        }
        self.execute_prepared_family(plan, admission)
    }

    fn check_family_location(&self, plan: &FamilyPlan) -> Result<(), String> {
        if std::fs::canonicalize(&plan.directory).map_err(|e| e.to_string())? != plan.directory
            || fingerprint(&std::fs::metadata(&plan.directory).map_err(|e| e.to_string())?)
                != plan.directory_stamp
            || plan.parent_stamp.is_some_and(|expected| {
                !Self::family_destination_stamp(&plan.destination)
                    .is_ok_and(|actual| actual == expected)
            })
        {
            return Err(
                "Prepared directory or destination parent changed while review was open".into(),
            );
        }
        Ok(())
    }

    fn check_family_source(
        &mut self,
        plan: &FamilyPlan,
        admission: &mut Lease,
    ) -> Result<(), String> {
        if let Some(source) = &plan.source {
            self.checked_sequence(source, admission)?;
            if source.refs.observation.status.is_some()
                && self.stash_evidence(source, admission)? != plan.evidence
            {
                return Err("Saved source bytes changed while prepared review was open".into());
            }
        }
        Ok(())
    }

    fn execute_prepared_family(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        use crate::git::families::FamilyProof;
        match &plan.proof {
            FamilyProof::Submodule(expected) => {
                self.execute_checked_submodule(&plan, expected, admission)
            }
            FamilyProof::Subtree(_) => self.execute_subtree(&plan, admission),
            FamilyProof::Patch(_) => self.execute_patch(&plan, admission),
            FamilyProof::Output(_) => self.execute_output(&plan, admission),
            FamilyProof::Notes(_) => self.execute_notes(&plan, admission),
            FamilyProof::Sparse(_) => self.execute_sparse(&plan, admission),
            FamilyProof::Direct(_) | FamilyProof::Creation => {
                self.execute_creation(plan, admission)
            }
        }
    }

    fn execute_checked_submodule(
        &mut self,
        plan: &Arc<FamilyPlan>,
        expected: &crate::git::families::Submodule,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let actual = self.submodule_state(
            plan.family,
            &plan.destination,
            plan.source.as_ref().expect("submodule source"),
            admission,
        )?;
        if !expected.same(&actual) {
            return Err("Submodule configuration, gitlink or saved child state changed while review was open".into());
        }
        self.execute_submodule(plan, admission)
    }
}
