//! Explicit bounded project publication. Every destination has an independent
//! outcome; no source save acknowledgement or project-wide disk atomicity exists.
mod discover;
mod output;
mod selection;
mod upload;
use crate::{
    buffer::{Buffer, Buffers},
    config::{self, ExportBackend, Org, PublishProject},
    file::{Location, Publication},
    org::{
        Cancellation, export,
        io::{Sources, source::Budget},
        time::ZonedOperation,
    },
};
pub(crate) use output::{Prepared, Write};
pub(crate) use selection::Selection;
use std::{
    collections::VecDeque,
    path::PathBuf,
    time::{Duration, Instant},
};
pub(crate) use upload::Upload;
/// Retained per-file result; failures never erase earlier successful outputs.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub source: PathBuf,
    pub target: PathBuf,
    pub publication: Publication,
    pub detail: String,
}
pub(crate) enum Item {
    Ready(Box<Prepared>),
    Issue(Outcome),
}
pub(crate) struct Plan {
    pub selection: Selection,
    pub items: VecDeque<Item>,
}
pub(crate) struct Work {
    selection: Selection,
    reader: Sources,
    policy: Org,
    time: ZonedOperation,
}
impl Work {
    pub fn new(
        buffers: &Buffers,
        selection: Selection,
        time: ZonedOperation,
    ) -> Result<Self, String> {
        let policy = buffers.snapshot().settings().org.clone();
        Ok(Self {
            selection,
            reader: Sources::new(buffers, &policy.limits)?,
            policy,
            time,
        })
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Plan, String> {
        let budget = Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(self.policy.limits.job_seconds as u64),
        };
        let discovered = discover::scan(&self.selection, &self.policy.limits, &budget)?;
        let mut plan = Plan {
            selection: self.selection.clone(),
            items: VecDeque::new(),
        };
        let mut bytes = 0usize;
        for entry in discovered {
            budget.check()?;
            self.prepare(entry, &mut plan, &mut bytes, &budget)?;
        }
        if plan.items.is_empty() {
            return Err("Project selected no files; nothing published or uploaded".into());
        }
        plan.collisions();
        Ok(plan)
    }
    fn prepare(
        &self,
        entry: discover::Entry,
        plan: &mut Plan,
        bytes: &mut usize,
        budget: &Budget<'_>,
    ) -> Result<(), String> {
        let source = entry.path.clone();
        let target = self.selection.output.join(&entry.relative);
        if let Some(error) = entry.error {
            return admit(
                plan,
                Item::Issue(Outcome::failed(source, target, error)),
                bytes,
                &self.policy,
            );
        }
        if entry.asset {
            let prepared = Prepared::asset(&source, &target, &self.policy.limits, budget);
            return admit(plan, item(&source, &target, prepared), bytes, &self.policy);
        }
        for backend in &self.selection.project.formats {
            budget.check()?;
            let target = target.with_extension(selection::extension(*backend));
            let prepared = self.document((&source, &target), *backend, budget);
            admit(plan, item(&source, &target, prepared), bytes, &self.policy)?;
        }
        Ok(())
    }
    fn document(
        &self,
        paths: (&std::path::Path, &std::path::Path),
        backend: ExportBackend,
        budget: &Budget<'_>,
    ) -> Result<Prepared, String> {
        let (source, target) = paths;
        discover::regular(source)?;
        let source = self.reader.read(source, budget)?;
        let request = export::Request {
            backend,
            scope: export::Scope::Buffer,
            options: None,
            time: self.time.clone(),
        };
        let snapshot = export::Work::selected(self.reader.clone(), source, request, &self.policy)?
            .run(budget.cancel)?;
        Prepared::document(snapshot, target, budget.cancel)
    }
}
fn item(
    source: &std::path::Path,
    target: &std::path::Path,
    result: Result<Prepared, String>,
) -> Item {
    match result {
        Ok(prepared) => Item::Ready(Box::new(prepared)),
        Err(e) => Item::Issue(Outcome::failed(source.into(), target.into(), e)),
    }
}
fn admit(plan: &mut Plan, item: Item, bytes: &mut usize, policy: &Org) -> Result<(), String> {
    if plan.items.len() >= policy.limits.agenda_files {
        return Err(
            "Project output count exceeds org.limits.agenda_files; nothing published".into(),
        );
    }
    if let Item::Ready(prepared) = &item {
        *bytes = bytes
            .checked_add(prepared.size())
            .ok_or("Project byte count overflow")?;
    }
    if *bytes > policy.limits.agenda_total_bytes {
        return Err("Project retained source/output bytes exceed org.limits.agenda_total_bytes; nothing published".into());
    }
    plan.items.push_back(item);
    Ok(())
}
impl Outcome {
    pub(crate) fn failed(source: PathBuf, target: PathBuf, detail: String) -> Self {
        Self {
            source,
            target,
            publication: Publication::NotPublished,
            detail,
        }
    }
}
impl Plan {
    fn collisions(&mut self) {
        let mut counts = std::collections::BTreeMap::new();
        for item in &self.items {
            if let Item::Ready(p) = item {
                *counts.entry(p.target.path().to_owned()).or_insert(0usize) += 1;
            }
        }
        for item in &mut self.items {
            if let Item::Ready(p) = item
                && counts[p.target.path()] > 1
            {
                *item =
                    Item::Issue(p.failed("Multiple project inputs select the same destination"));
            }
        }
    }
}
pub(crate) enum JobWork {
    Discover(Box<Work>),
    Write(Box<Write>),
    Upload(Upload),
}
pub(crate) enum Output {
    Plan(Box<Plan>),
    File(Outcome),
    Uploaded(Result<String, String>),
}
impl JobWork {
    pub fn run(self, cancel: &Cancellation) -> Result<Output, String> {
        match self {
            Self::Discover(work) => work.run(cancel).map(|plan| Output::Plan(Box::new(plan))),
            Self::Write(work) => Ok(Output::File(work.run(cancel))),
            Self::Upload(work) => Ok(Output::Uploaded(work.run(cancel))),
        }
    }
}
impl Plan {
    pub fn inputs(&self) -> Result<Vec<PathBuf>, String> {
        let mut paths = std::collections::BTreeSet::new();
        for item in &self.items {
            match item {
                Item::Ready(p) => paths.extend(p.inputs()),
                Item::Issue(issue) => {
                    paths.insert(issue.source.clone());
                }
            }
            if paths.len() > 4096 {
                return Err("Project inputs exceed 4096 distinct paths".into());
            }
        }
        if paths.iter().map(|p| p.as_os_str().len()).sum::<usize>() > 1048576 {
            return Err("Project input names exceed 1 MiB".into());
        }
        Ok(paths.into_iter().collect())
    }
    pub fn current(&self, buffers: &Buffers) -> Result<(), String> {
        for item in &self.items {
            if let Item::Ready(p) = item {
                p.current(buffers)?;
            }
        }
        Ok(())
    }
}
