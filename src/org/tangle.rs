//! Explicit native tangling: a bounded immutable expansion plan precedes every
//! checked file effect. Source text and history are never acknowledged as saved.
pub(in crate::org) mod expand;
mod options;
mod output;
use crate::{
    buffer::{Buffer, Stamp},
    coordinates::Byte,
    file::Publication,
    org::{
        Cancellation,
        block::{Definition, Scan},
    },
};
pub(crate) use output::{Prepared, Write};
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
};
const BYTES: usize = 8 * 1024 * 1024;
const FILES: usize = 256;
/// Explicit selection within the current canonical source.
#[derive(Clone, Copy)]
pub enum Selection {
    Buffer,
    Block(Byte),
}
/// Captures revision-qualified source without reading files or publishing data.
pub struct Work {
    scan: Scan,
    source: Option<PathBuf>,
    stamp: Stamp,
    selection: Selection,
}
/// Immutable planned output. Relative destinations resolve at the Org file.
#[derive(Clone, Debug)]
pub struct Target {
    path: PathBuf,
    text: String,
    mkdir: bool,
}
impl Target {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}
pub struct Plan {
    stamp: Stamp,
    source: Option<PathBuf>,
    targets: Vec<Target>,
}
impl Plan {
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
}
/// Actual per-file outcome, retained independently of subsequent failures.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub target: PathBuf,
    pub publication: Publication,
    pub detail: String,
}
impl Work {
    pub fn new(buffer: &Buffer, selection: Selection) -> Result<Self, String> {
        let at = match selection {
            Selection::Buffer => Byte(0),
            Selection::Block(at) => at,
        };
        Ok(Self {
            scan: Scan::new(buffer, at)?,
            source: buffer.file().map(|f| f.path().into()),
            stamp: buffer.stamp(),
            selection,
        })
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Plan, String> {
        let selected = match self.selection {
            Selection::Buffer => None,
            Selection::Block(_) => Some(self.scan.clone().run(cancel)?.body),
        };
        let blocks = self.scan.all(cancel)?;
        let mut expansion = expand::Expansion::new(&blocks, cancel)?;
        let mut plan = Plan {
            stamp: self.stamp,
            source: self.source,
            targets: Vec::new(),
        };
        let mut bytes = 0usize;
        for (index, block) in blocks.iter().enumerate() {
            options::check(cancel)?;
            if selected.as_ref().is_some_and(|r| r != &block.body) {
                continue;
            }
            plan.block((index, block), &mut expansion, &mut bytes)?;
        }
        if plan.targets.is_empty() {
            return Err("No selected source blocks declare a tangle destination".into());
        }
        Ok(plan)
    }
}
impl Plan {
    fn block(
        &mut self,
        entry: (usize, &Definition),
        expansion: &mut expand::Expansion<'_>,
        bytes: &mut usize,
    ) -> Result<(), String> {
        let (index, block) = entry;
        let Some(path) = options::destination(block, self.source.as_deref())? else {
            return Ok(());
        };
        options::supported(block)?;
        let text = expansion.block(index)?;
        let mkdir = options::yes(block, "mkdirp", false)?;
        let pad = options::yes(block, "padline", true)?;
        let target = self.target(path, mkdir)?;
        let separator = if pad && !target.text.is_empty() {
            "\n"
        } else {
            ""
        };
        *bytes = bytes
            .checked_add(text.len() + separator.len())
            .filter(|n| *n <= BYTES)
            .ok_or("Tangle output exceeds 8 MiB")?;
        target.text.push_str(separator);
        target.text.push_str(&text);
        Ok(())
    }
    fn target(&mut self, path: PathBuf, mkdir: bool) -> Result<&mut Target, String> {
        let index = match self.targets.iter().position(|t| t.path == path) {
            Some(i) => {
                if self.targets[i].mkdir != mkdir {
                    return Err("Conflicting mkdirp declarations for one tangle file".into());
                }
                i
            }
            None => {
                if self.targets.len() >= FILES {
                    return Err("Tangle exceeds 256 files".into());
                }
                self.targets.push(Target {
                    path,
                    text: String::new(),
                    mkdir,
                });
                self.targets.len() - 1
            }
        };
        Ok(&mut self.targets[index])
    }
    pub fn current(&self, buffer: &Buffer) -> Result<(), String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())
    }
    pub(crate) fn prepare(
        self,
        mut inputs: Vec<PathBuf>,
        cancel: &Cancellation,
    ) -> Result<PreparedPlan, String> {
        if let Some(source) = self.source {
            inputs.push(source);
        }
        inputs.sort();
        inputs.dedup();
        if inputs.len() > 4096
            || inputs.iter().map(|p| p.as_os_str().len()).sum::<usize>() > 1024 * 1024
        {
            return Err("Tangle input protection exceeds 4096 paths or 1 MiB".into());
        }
        let inputs: std::sync::Arc<[PathBuf]> = inputs.into();
        let mut files = VecDeque::new();
        let mut identities = BTreeMap::new();
        for target in self.targets {
            options::check(cancel)?;
            let prepared = Prepared::new(target, inputs.clone(), cancel)?;
            if let Some(o) = prepared.target.observation()
                && identities
                    .insert((o.device, o.inode), prepared.target.path().to_owned())
                    .is_some()
            {
                return Err("Tangle destinations alias one another".into());
            }
            files.push_back(prepared);
        }
        Ok(PreparedPlan {
            stamp: self.stamp,
            files,
        })
    }
}
pub(crate) struct PreparedPlan {
    pub stamp: Stamp,
    pub files: VecDeque<Prepared>,
}
pub(crate) enum JobWork {
    Prepare(Work, Vec<PathBuf>),
    Write(Box<Write>),
}
pub(crate) enum Output {
    Plan(PreparedPlan),
    File(Outcome),
}
impl JobWork {
    pub fn effect(&self) -> bool {
        matches!(self, Self::Write(_))
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Output, String> {
        match self {
            Self::Prepare(work, inputs) => {
                Ok(Output::Plan(work.run(cancel)?.prepare(inputs, cancel)?))
            }
            Self::Write(work) => Ok(Output::File(work.run(cancel))),
        }
    }
}
