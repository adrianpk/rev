use super::*;
mod syntax;
use crate::org::io::{DestinationVersion, Sources};
pub(super) fn expand(
    source: Evidence,
    range: Range<usize>,
    reader: &Sources,
    policy: &Org,
    cancel: &Cancellation,
) -> Result<(Rope, Vec<Evidence>), String> {
    let mut work = Expand {
        reader,
        policy,
        budget: Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(policy.limits.job_seconds as u64),
        },
        sources: Vec::new(),
        active: Vec::new(),
        occurrences: 0,
        bytes: 0,
        nodes: 0,
    };
    let result = work.document(source, range)?;
    Ok((Rope::from_str(&result), work.sources))
}
struct Expand<'a> {
    reader: &'a Sources,
    policy: &'a Org,
    budget: Budget<'a>,
    sources: Vec<Evidence>,
    active: Vec<Evidence>,
    occurrences: usize,
    bytes: usize,
    nodes: usize,
}
impl Expand<'_> {
    fn document(&mut self, source: Evidence, range: Range<usize>) -> Result<String, String> {
        let nodes = self.admit(&source, &range)?;
        self.active.push(source.clone());
        let output = self.body(&source, &nodes, range)?;
        self.active.pop();
        Ok(output)
    }
    fn admit(
        &mut self,
        source: &Evidence,
        range: &Range<usize>,
    ) -> Result<Vec<structure::Raw>, String> {
        self.budget.check()?;
        if self.active.iter().any(|s| same(s, source)) {
            return Err("Org include cycle detected".into());
        }
        self.bytes = self
            .bytes
            .checked_add(source.text.len_bytes())
            .ok_or("Export source bytes overflow")?;
        if self.bytes > self.policy.limits.agenda_total_bytes {
            return Err("Export total source byte limit reached".into());
        }
        if !self.sources.iter().any(|s| same(s, source)) {
            self.sources.push(source.clone());
        }
        let nodes = document::nodes(&source.text, self.budget.cancel)?;
        scope::region(&nodes, range)?;
        self.nodes = self
            .nodes
            .checked_add(nodes.len())
            .ok_or("Export structure count overflow")?;
        if self.nodes > super::super::Limits::default().nodes {
            return Err("Export total source node limit reached".into());
        }
        Ok(nodes)
    }
    fn body(
        &mut self,
        source: &Evidence,
        nodes: &[structure::Raw],
        range: Range<usize>,
    ) -> Result<String, String> {
        let mut output = String::new();
        let mut cursor = range.start;
        for node in nodes.iter().filter(|n| within(n, &range)) {
            self.budget.check()?;
            if let Some(value) = directive(&source.text, node) {
                self.append(
                    &mut output,
                    source.text.byte_slice(cursor..node.source.start),
                )?;
                let selected = syntax::Include::parse(&value)?;
                let included = self.read(source, &selected)?;
                self.push(&mut output, &included)?;
                self.separator(&mut output, &source.text, node)?;
                cursor = node.source.end;
            }
        }
        self.append(&mut output, source.text.byte_slice(cursor..range.end))?;
        Ok(output)
    }
    fn read(&mut self, parent: &Evidence, include: &syntax::Include) -> Result<String, String> {
        if self.active.len() > self.policy.limits.include_depth {
            return Err("Org include depth limit reached".into());
        }
        self.occurrences += 1;
        if self.occurrences > self.policy.limits.include_files {
            return Err("Org include file limit reached".into());
        }
        let path = include.path(parent.path.as_deref())?;
        let source = self
            .reader
            .read(&path, &self.budget)
            .map_err(|e| format!("Org include: {e}"))?;
        let text = source.snapshot();
        let version = match source.version {
            DestinationVersion::Missing => return Err("Org include file is missing".into()),
            DestinationVersion::Buffer(stamp) => SourceVersion::Buffer(stamp),
            DestinationVersion::Disk(observation) => SourceVersion::Disk(observation),
        };
        let range = include.range(&text)?;
        self.document(
            Evidence {
                path: Some(source.path),
                version,
                text,
            },
            range,
        )
    }
    fn append(&self, out: &mut String, text: RopeSlice<'_>) -> Result<(), String> {
        for chunk in text.chunks() {
            self.budget.check()?;
            self.push(out, chunk)?;
            std::thread::yield_now();
        }
        Ok(())
    }
    fn push(&self, out: &mut String, value: &str) -> Result<(), String> {
        if value.len() > self.policy.limits.expansion_bytes.saturating_sub(out.len()) {
            return Err("Org expansion byte limit reached".into());
        }
        out.push_str(value);
        Ok(())
    }
    fn separator(
        &self,
        out: &mut String,
        text: &Rope,
        node: &structure::Raw,
    ) -> Result<(), String> {
        let original = text.byte_slice(node.source.clone());
        if original.len_bytes() > 0
            && original.byte(original.len_bytes() - 1) == b'\n'
            && !out.ends_with('\n')
        {
            let ending =
                if original.len_bytes() >= 2 && original.byte(original.len_bytes() - 2) == b'\r' {
                    "\r\n"
                } else {
                    "\n"
                };
            self.push(out, ending)?;
        }
        Ok(())
    }
}
fn directive(text: &Rope, node: &structure::Raw) -> Option<String> {
    let (key, value) = document::keyword(text, node)?;
    (key == "INCLUDE").then_some(value)
}
fn same(a: &Evidence, b: &Evidence) -> bool {
    match (&a.version, &b.version) {
        (SourceVersion::Buffer(a), SourceVersion::Buffer(b)) => a.buffer == b.buffer,
        (SourceVersion::Disk(a), SourceVersion::Disk(b)) => {
            a.device == b.device && a.inode == b.inode
        }
        _ => false,
    }
}

fn within(n: &structure::Raw, r: &Range<usize>) -> bool {
    n.source.start >= r.start && n.source.end <= r.end
}
