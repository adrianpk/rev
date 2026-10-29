//! RFC 5545 data emission has no file, clock, process or network authority.
mod component;
mod dates;
mod observe;
mod recurrence;
mod wire;
mod zones;
use super::*;
use crate::org::{
    property::Properties,
    time::{CivilDateTime, Date, Resolution, TimestampRange},
};
use std::collections::BTreeMap;

struct Calendar<'a> {
    snapshot: &'a Snapshot,
    cancel: &'a Cancellation,
    output: String,
    diagnostics: Vec<String>,
    width: usize,
    identities: BTreeMap<String, Vec<u8>>,
    count: usize,
    zone: &'a crate::org::time::Zone,
}
struct Entry {
    node: usize,
    raw: String,
    summary: String,
    task: Option<bool>,
    excluded: bool,
    body: Range<usize>,
    drawers: Vec<Range<usize>>,
    properties: BTreeMap<&'static str, String>,
    scheduled: Option<TimestampRange>,
    deadline: Option<TimestampRange>,
    appointments: Vec<TimestampRange>,
}
pub(super) fn render(snapshot: &Snapshot, cancel: &Cancellation) -> Result<Rendered, String> {
    let entries = observe::entries(snapshot, cancel)?;
    let mut out = Calendar {
        snapshot,
        cancel,
        output: String::new(),
        diagnostics: snapshot.diagnostics.clone(),
        width: 0,
        identities: BTreeMap::new(),
        count: 0,
        zone: snapshot.request.time.zone(),
    };
    out.preamble()?;
    for entry in &entries {
        out.entry(entry)?;
    }
    if out.count == 0 {
        return Err("No native Org calendar entries in selection".into());
    }
    out.raw("END", "VCALENDAR")?;
    Ok(Rendered {
        text: out.output,
        diagnostics: out.diagnostics,
    })
}
impl Calendar<'_> {
    fn preamble(&mut self) -> Result<(), String> {
        self.raw("BEGIN", "VCALENDAR")?;
        self.raw("VERSION", "2.0")?;
        self.raw("PRODID", "-//Rev//Native Org Calendar//EN")?;
        self.raw("CALSCALE", "GREGORIAN")?;
        if let Some(title) = self.snapshot.metadata.title.as_ref().filter(|_| {
            self.snapshot.options.title && !matches!(self.snapshot.request.scope, Scope::Region(_))
        }) {
            self.property("X-WR-CALNAME", title)?;
        }
        Ok(())
    }
    fn diagnostic(&mut self, message: &str) -> Result<(), String> {
        if self.diagnostics.len() >= 128 {
            return Err("Calendar diagnostic limit reached".into());
        }
        self.diagnostics.push(message.into());
        Ok(())
    }
    fn identity(&mut self, entry: &Entry) -> Result<String, String> {
        let key = identity(self.snapshot, entry)?;
        let id = digest(&key);
        if let Some(old) = self.identities.insert(id.clone(), key.clone()) {
            let message = if old == key {
                "Duplicate calendar entry identity; use unique ID properties"
            } else {
                "Calendar UID collision; no output published"
            };
            return Err(message.into());
        }
        Ok(id)
    }
}
fn identity(snapshot: &Snapshot, entry: &Entry) -> Result<Vec<u8>, String> {
    if let Some(id) = entry.properties.get("ID") {
        if id.is_empty() || id.len() > 1024 || id.chars().any(char::is_control) {
            return Err("Calendar ID requires 1..1024 bytes without controls".into());
        }
        return Ok([b"id\0".as_slice(), id.as_bytes()].concat());
    }
    let source = snapshot
        .sources
        .first()
        .ok_or("Calendar snapshot has no source identity")?;
    let origin = match &source.path {
        Some(path) => path.as_os_str().as_encoded_bytes().to_vec(),
        None => match source.version {
            SourceVersion::Buffer(stamp) => format!("buffer:{:?}", stamp.buffer).into_bytes(),
            _ => return Err("Calendar disk source has no path".into()),
        },
    };
    let options = Options {
        todo: false,
        tags: false,
        ..snapshot.options.clone()
    };
    let range = super::render::headings::title_range(
        &snapshot.text,
        &snapshot.nodes[entry.node],
        &options,
        &snapshot.todo,
    );
    let title = snapshot.text.byte_slice(range).to_string();
    Ok([
        b"source\0".as_slice(),
        &origin,
        b"\0heading\0",
        title.trim().as_bytes(),
    ]
    .concat())
}
// Two independent stable accumulators keep paths out of output. Every digest
// is checked against its full source identity before any component publication.
fn digest(bytes: &[u8]) -> String {
    let mut left = 0xcbf29ce484222325u64;
    let mut right = 0x84222325cbf29ce4u64;
    for byte in bytes {
        left = (left ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        right = (right ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
    }
    format!("{left:016x}{right:016x}")
}

impl Entry {
    fn has_data(&self) -> bool {
        self.task.is_some()
            || self.scheduled.is_some()
            || self.deadline.is_some()
            || !self.appointments.is_empty()
    }
}
