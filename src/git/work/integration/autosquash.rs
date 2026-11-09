use super::*;

impl Runner {
    pub(super) fn prepare_autosquash(
        &mut self,
        refs: Arc<Refs>,
        target_name: &[u8],
        admission: &mut Lease,
    ) -> Result<AutosquashPlan, String> {
        let current = self.check_integration_source(&refs, admission)?;
        if self.read_sequence(&current, admission)?.family.is_some() {
            return Err("Autosquash preparation requires no active Git-owned sequence".into());
        }
        self.integration_policy(&refs, IntegrationMode::Rebase)?;
        let target = refs
            .reference(target_name)
            .and_then(Reference::commit)
            .ok_or("Autosquash target expired")?;
        let head = refs.head.ok_or("Autosquash HEAD is unborn")?;
        let bases = self.command(
            &["merge-base", "--all", &target.hex(), &head.hex()],
            Phase::Checking,
            false,
        )?;
        let base = ObjectId::parse(
            bases.strip_suffix(b"\n").unwrap_or(&bases),
            refs.observation.identity.format,
        )
        .map_err(|_| "Autosquash requires one unambiguous source merge base")?;
        let range = format!("{}..{}", base.hex(), head.hex());
        let log = self.command(
            &[
                "log",
                "--reverse",
                "--topo-order",
                "--max-count=257",
                "--format=%H%x00%P%x00%s%x00",
                &range,
                "--",
            ],
            Phase::Checking,
            false,
        )?;
        admission.grow(log.len() * 3 + 256 * 256)?;
        let fields: Vec<_> = log.split(|b| *b == 0).collect();
        if fields.len() < 4 || fields.len() % 3 != 1 || fields.last() != Some(&b"\n".as_slice()) {
            return Err(
                "Autosquash has no finite source commits or malformed source metadata".into(),
            );
        }
        let count = fields.len() / 3;
        if count > 256 {
            return Err("Autosquash exceeds the 256-source-commit admission".into());
        }
        let mut source = Vec::with_capacity(count);
        let mut parent = base;
        for entry in fields[..fields.len() - 1].as_chunks::<3>().0 {
            let object = ObjectId::parse(
                entry[0].strip_prefix(b"\n").unwrap_or(entry[0]),
                refs.observation.identity.format,
            )?;
            if entry[1] != parent.hex().as_bytes() {
                return Err("Reviewed autosquash requires a linear source range; merge commits need a different explicit workflow".into());
            }
            let subject =
                std::str::from_utf8(entry[2]).map_err(|_| "Autosquash subject is not UTF-8")?;
            if subject.len() > 4096 || subject.chars().any(char::is_control) {
                return Err("Autosquash subject exceeds the 4096-byte metadata admission or contains control characters".into());
            }
            source.push((object, subject.to_owned()));
            parent = object;
        }
        if parent != head {
            return Err("Autosquash source range does not end at the captured HEAD".into());
        }
        let mut groups: Vec<Vec<AutosquashStep>> = (0..count).map(|_| Vec::new()).collect();
        let mut roots = Vec::with_capacity(count);
        for (index, (object, subject)) in source.iter().enumerate() {
            let special = [
                ("fixup! ", AutosquashAction::Fixup),
                ("squash! ", AutosquashAction::Squash),
                ("amend! ", AutosquashAction::ReplaceMessage),
            ]
            .into_iter()
            .find_map(|(prefix, action)| {
                subject.strip_prefix(prefix).map(|target| (action, target))
            });
            let (action, root) = if let Some((action, target)) = special {
                if target.trim().is_empty() {
                    return Err("Autosquash target subject is empty; no target is guessed".into());
                }
                let candidates = &source[..index];
                let exact: Vec<_> = candidates
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, s))| s == target)
                    .map(|(i, _)| i)
                    .collect();
                let matches: Vec<_> = if target.len() >= 4
                    && target.len() <= 64
                    && target.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    candidates
                        .iter()
                        .enumerate()
                        .filter(|(_, (id, _))| id.hex().starts_with(target))
                        .map(|(i, _)| i)
                        .collect()
                } else if !exact.is_empty() {
                    exact
                } else {
                    candidates
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, s))| s.starts_with(target))
                        .map(|(i, _)| i)
                        .collect()
                };
                if matches.len() != 1 {
                    return Err("Autosquash fixup/squash target is missing or ambiguous; no target is guessed".into());
                }
                (action, roots[matches[0]])
            } else {
                (AutosquashAction::Pick, index)
            };
            roots.push(root);
            groups[root].push(AutosquashStep {
                action,
                object: *object,
                subject: subject.clone(),
            });
        }
        let steps = groups.into_iter().flatten().collect();
        self.check_integration_source(&refs, admission)?;
        let mut plan = AutosquashPlan {
            refs,
            target_name: target_name.into(),
            target,
            base,
            steps,
            _admission: admission.split(0),
        };
        admission.grow(plan.bytes())?;
        plan._admission = admission.split(plan.bytes());
        Ok(plan)
    }
    pub(super) fn autosquash_args(
        &mut self,
        plan: &AutosquashPlan,
        admission: &mut Lease,
    ) -> Result<Vec<OsString>, String> {
        let actual = self.prepare_autosquash(
            std::sync::Arc::clone(&plan.refs),
            &plan.target_name,
            admission,
        )?;
        if !plan.same(&actual) {
            return Err("Autosquash source objects or ordered plan changed during review".into());
        }
        let size = plan
            .steps
            .iter()
            .map(|s| s.action.label().len() + s.object.hex().len() + 2)
            .sum::<usize>();
        admission.grow(size * 2)?;
        let mut todo = Vec::with_capacity(size);
        for step in &plan.steps {
            todo.extend_from_slice(step.action.label().as_bytes());
            todo.push(b' ');
            todo.extend_from_slice(step.object.hex().as_bytes());
            todo.push(b'\n');
        }
        self.todo_editor = crate::process::input::stage(Some(&todo))?;
        // The descriptor can outlive this worker when cleanup is retained.
        // Move its exact byte reservation alongside that process owner.
        self.driver_admission = Some(admission.split(size));
        Ok([
            "rebase",
            "--interactive",
            "--autosquash",
            "--no-autostash",
            "--no-update-refs",
            "--no-rebase-merges",
            "--no-fork-point",
            "--onto",
        ]
        .into_iter()
        .map(OsString::from)
        .chain([plan.target.hex().into(), plan.base.hex().into()])
        .collect())
    }
}
