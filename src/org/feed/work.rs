//! One explicit feed job: retained target, bounded input, no background service.
use super::*;
use crate::{
    buffer::Buffers,
    config::{FeedSource, Org, Snapshot},
    org::{
        execution::scratch::Scratch,
        io::{CaptureRead, DestinationSource, process::Program},
    },
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
#[derive(Clone)]
pub(crate) struct Choice {
    pub definition: FeedSource,
    pub path: PathBuf,
    input: String,
    argv: Vec<String>,
    policy: Org,
}
pub(crate) struct Work {
    choice: Choice,
    target: CaptureRead,
    input: Option<CaptureRead>,
    inbox: bool,
}
impl Choice {
    pub fn configured(snapshot: &Snapshot, name: &str) -> Result<Self, String> {
        let definition = snapshot
            .settings()
            .org
            .feeds
            .sources
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or("Select a declared feed name")?;
        let root = snapshot
            .origin(&["org", "feeds", "sources"])
            .and_then(Path::parent)
            .unwrap_or(Path::new("."));
        let input = if definition.remote() {
            definition.source.clone()
        } else {
            root.join(&definition.source)
                .to_str()
                .ok_or("Feed path must be UTF-8")?
                .into()
        };
        let argv = transport(snapshot, &definition)?;
        Ok(Self {
            path: root.join(&definition.file),
            input,
            definition,
            argv,
            policy: snapshot.settings().org.clone(),
        })
    }
    pub fn program(&self) -> Option<&str> {
        self.argv.first().map(String::as_str)
    }
    pub fn prepare(self, buffers: &Buffers, inbox: bool) -> Result<Work, String> {
        let target = CaptureRead::new(buffers, &self.path, &self.policy.limits)?;
        let input = if inbox || !self.argv.is_empty() {
            None
        } else {
            if self.definition.remote() {
                return Err("No feed transport configured; set org.tools.fetch".into());
            }
            let mut limits = self.policy.limits.clone();
            limits.agenda_file_bytes = limits.feed_bytes;
            limits.agenda_total_bytes = limits.feed_bytes;
            Some(CaptureRead::new(buffers, Path::new(&self.input), &limits)?)
        };
        Ok(Work {
            choice: self,
            target,
            input,
            inbox,
        })
    }
}
fn transport(snapshot: &Snapshot, feed: &FeedSource) -> Result<Vec<String>, String> {
    let (mut argv, leaf) = if let Some(argv) = &feed.transport {
        (argv.clone(), ["org", "feeds", "sources"])
    } else if feed.remote() {
        (
            snapshot.settings().org.tools.fetch.clone(),
            ["org", "tools", "fetch"],
        )
    } else {
        (Vec::new(), ["org", "tools", "fetch"])
    };
    crate::config::feeds::argv_check(&argv)?;
    if let Some(executable) = argv.first_mut() {
        let path = Path::new(executable);
        if path.is_relative()
            && executable.contains('/')
            && let Some(root) = snapshot.origin(&leaf).and_then(Path::parent)
        {
            *executable = root
                .join(path)
                .to_str()
                .ok_or("Feed executable must be UTF-8")?
                .into();
        }
    }
    Ok(argv)
}
impl Work {
    pub fn run(self, cancel: &Cancellation) -> Result<Plan, String> {
        let cancel = cancel.bounded(
            Instant::now() + Duration::from_secs(self.choice.policy.limits.job_seconds as u64),
        );
        let target = self.target.run(&cancel)?;
        if target.readonly && !self.inbox {
            return Err("Feed destination is read-only".into());
        }
        let locator = target::resolve(
            &target,
            &self.choice.definition,
            &self.choice.policy,
            &cancel,
        )?;
        target::admit(&target, &locator, &self.choice.definition.drawer, &cancel)?;
        if self.inbox {
            return target::inbox(target, locator, &cancel);
        }
        update(self.choice, self.input, target, locator, &cancel)
    }
}
fn update(
    choice: Choice,
    read: Option<CaptureRead>,
    target: DestinationSource,
    locator: crate::org::capture::Target,
    cancel: &Cancellation,
) -> Result<Plan, String> {
    let (value, evidence) = input(read, &choice, cancel)?;
    let base = choice.definition.remote().then_some(choice.input.as_str());
    let items = Feed::parse(&value, base, &choice.policy.limits, cancel)?;
    let mut plan = target::plan(target, locator, &choice.definition, items, cancel)?;
    plan.evidence = evidence.into_iter().collect();
    xml::check(cancel)?;
    Ok(plan)
}
fn input(
    read: Option<CaptureRead>,
    choice: &Choice,
    cancel: &Cancellation,
) -> Result<(String, Option<DestinationSource>), String> {
    if let Some(read) = read {
        let source = read.run(cancel)?;
        if matches!(source.version, crate::org::io::DestinationVersion::Missing) {
            return Err("Feed source file is missing".into());
        }
        if source.text().len_bytes() > choice.policy.limits.feed_bytes {
            return Err("Feed byte limit reached".into());
        }
        return Ok((source.text().to_string(), Some(source)));
    }
    let value = fetch(choice, cancel)?;
    String::from_utf8(value)
        .map(|s| (s, None))
        .map_err(|_| "Feed transport returned invalid UTF-8".into())
}
fn fetch(choice: &Choice, cancel: &Cancellation) -> Result<Vec<u8>, String> {
    let mut scratch = Scratch::new()?;
    let result = fetch_in(choice, &scratch, cancel);
    scratch.clean()?;
    result
}
fn fetch_in(choice: &Choice, scratch: &Scratch, cancel: &Cancellation) -> Result<Vec<u8>, String> {
    let mut argv = choice.argv.clone();
    if argv.is_empty() {
        return Err("No feed transport configured; set org.tools.fetch".into());
    }
    if let Some(token) = argv.iter_mut().find(|a| a.as_str() == "{target}") {
        *token = choice.input.clone();
    } else {
        argv.push(choice.input.clone());
    }
    let limits = &choice.policy.limits;
    let result = Program {
        argv,
        directory: scratch.root().into(),
        seconds: limits.job_seconds,
        output_bytes: limits.feed_bytes.min(limits.job_output_bytes),
    }
    .run_input(None, Some(scratch.root()), cancel)?;
    if !result.status.success() {
        let detail = String::from_utf8_lossy(&result.stderr)
            .chars()
            .take(256)
            .flat_map(char::escape_default)
            .collect::<String>();
        return Err(format!(
            "Feed transport failed ({}): {detail}",
            result.status
        ));
    }
    Ok(result.stdout)
}
