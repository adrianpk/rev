//! A state snapshot retains its anchored directory, lock and prior bytes until
//! the read completes or a prepared CAS replacement is published.
use super::{
    Operation, Store, codec, conflict, error, invalid, limit, storage, unavailable, value, version,
};
use crate::file::Output;
use rev_plugin_sdk::{identity::*, methods::*, rpc::*};
use std::{fs::File, io::Write, path::PathBuf};

pub(super) struct Snapshot {
    parent: File,
    path: PathBuf,
    lock: Option<storage::Lock>,
    before: Option<Vec<u8>>,
    pub catalog: codec::Catalog,
}
impl Snapshot {
    pub fn read(store: &Store, mutation: bool) -> Result<Option<Self>, RpcError> {
        let parent = store.parent(mutation)?;
        let Some(parent) = parent else {
            return Ok(None);
        };
        storage::private(&parent)?;
        let base = storage::anchor(&parent);
        let lock = storage::lock(&base.join("state.toml.lock"), mutation)?;
        store.check_parent(&parent)?;
        let path = base.join("state.toml");
        let before = storage::read(&path)?;
        let catalog = codec::Catalog::decode(before.as_deref())?;
        Ok(Some(Self {
            parent,
            path,
            lock,
            before,
            catalog,
        }))
    }
    fn lock_path(&self) -> PathBuf {
        self.path.with_file_name("state.toml.lock")
    }
    pub fn prepare(mut self, operation: Operation) -> Result<Publication, RpcError> {
        let next = self
            .catalog
            .revision
            .checked_add(1)
            .filter(|v| *v <= i64::MAX as u64)
            .ok_or_else(|| limit("Package store revision exhausted"))?;
        let change = Change::new(operation)?;
        let outcome = change.apply(&mut self.catalog, next)?;
        let bytes = self.catalog.encode()?;
        Ok(Publication {
            snapshot: self,
            bytes,
            outcome,
        })
    }
}
pub(super) struct Publication {
    snapshot: Snapshot,
    bytes: Vec<u8>,
    outcome: Outcome,
}
impl Publication {
    fn stage(&self, output: &mut impl Output) -> Result<storage::Staging, RpcError> {
        let mut temporary = storage::Staging::new(&storage::anchor(&self.snapshot.parent))?;
        temporary
            .file()
            .write_all(&self.bytes)
            .and_then(|()| output.flush(temporary.file()))
            .and_then(|()| output.sync_file(temporary.file()))
            .map_err(|_| {
                unavailable("Cannot synchronize staged package state; destination unchanged")
            })?;
        Ok(temporary)
    }
}
struct Change {
    concern: RecordId,
    record: RecordId,
    expected: Option<Revision>,
    replacement: Option<String>,
}
impl Change {
    fn new(operation: Operation) -> Result<Self, RpcError> {
        let (concern, record, expected, replacement) = match operation {
            Operation::Put(params) => (
                params.concern,
                params.record,
                params.expected_version.0,
                Some(
                    serde_json::to_string(&params.value)
                        .map_err(|_| invalid("Invalid state JSON value"))?,
                ),
            ),
            Operation::Delete(params) => (
                params.concern,
                params.record,
                Some(params.expected_version),
                None,
            ),
            _ => unreachable!("read operations returned before publication"),
        };
        Ok(Self {
            concern,
            record,
            expected,
            replacement,
        })
    }
    fn check_version(
        &self,
        records: &[codec::Record],
        index: Option<usize>,
    ) -> Result<(), RpcError> {
        match (index, self.expected.as_ref()) {
            (None, None) if self.replacement.is_some() => {}
            (Some(i), Some(expected)) if records[i].version.to_string() == expected.as_str() => {}
            _ => {
                return Err(conflict(
                    "Package state record version changed; read current state before retrying",
                ));
            }
        }
        Ok(())
    }
    fn apply(self, catalog: &mut codec::Catalog, next: u64) -> Result<Outcome, RpcError> {
        let put = self.replacement.is_some();
        let mut records = std::mem::take(&mut catalog.records).into_vec();
        let index = records
            .iter()
            .position(|r| r.concern == self.concern && r.record == self.record);
        self.check_version(&records, index)?;
        match (index, self.replacement) {
            (Some(i), Some(json)) => {
                records[i] = codec::Record {
                    concern: self.concern.clone(),
                    record: self.record.clone(),
                    version: next,
                    value_json: json,
                }
            }
            (None, Some(json)) => records.push(codec::Record {
                concern: self.concern.clone(),
                record: self.record.clone(),
                version: next,
                value_json: json,
            }),
            (Some(i), None) => {
                records.remove(i);
            }
            _ => unreachable!("delete requires an existing observed record"),
        }
        records.sort_by(|a, b| (&a.concern, &a.record).cmp(&(&b.concern, &b.record)));
        catalog.records = List::new(records)?;
        catalog.revision = next;
        Ok(Outcome {
            concern: self.concern,
            record: self.record,
            version: next,
            put,
        })
    }
}
struct Outcome {
    concern: RecordId,
    record: RecordId,
    version: u64,
    put: bool,
}
impl Outcome {
    fn value(&self) -> serde_json::Value {
        if self.put {
            value(StatePutResult {
                version: version(self.version),
            })
        } else {
            value(Empty {})
        }
    }
}
impl Store {
    pub(super) fn reconcile_read(
        &mut self,
        snapshot: &Snapshot,
        output: &mut impl Output,
    ) -> Result<(), RpcError> {
        self.check_parent(&snapshot.parent)?;
        if let Some(lock) = &snapshot.lock {
            lock.check(&snapshot.lock_path())?;
        }
        if self.uncertain.is_some() {
            output
                .sync_directory(&snapshot.parent)
                .map_err(|_| conflict(self.uncertain.as_deref().expect("uncertain publication")))?;
            self.check_parent(&snapshot.parent)?;
            self.uncertain = None;
        }
        Ok(())
    }
    fn check_publication(
        &self,
        snapshot: &Snapshot,
        temporary: &storage::Staging,
    ) -> Result<(), RpcError> {
        self.check_parent(&snapshot.parent)?;
        snapshot
            .lock
            .as_ref()
            .expect("mutation owns a lock")
            .check(&snapshot.lock_path())?;
        temporary.check()?;
        if storage::read(&snapshot.path)? != snapshot.before {
            return Err(conflict("Package store changed before replacement"));
        }
        Ok(())
    }
    pub(super) fn publish(
        &mut self,
        publication: &Publication,
        live: impl Fn() -> bool,
        output: &mut impl Output,
    ) -> Result<serde_json::Value, RpcError> {
        let mut temporary = publication.stage(output)?;
        self.check_publication(&publication.snapshot, &temporary)?;
        if !live() {
            return Err(error(
                ErrorKind::Cancelled,
                "State request retired before publication",
            ));
        }
        output
            .replace(temporary.path(), &publication.snapshot.path)
            .map_err(|_| unavailable("Cannot replace package state; destination unchanged"))?;
        temporary.disarm();
        if !self.proven(publication, output) {
            return Err(self.uncertain_publication(&publication.outcome));
        }
        Ok(publication.outcome.value())
    }
    fn proven(&self, publication: &Publication, output: &mut impl Output) -> bool {
        output
            .sync_directory(&publication.snapshot.parent)
            .and_then(|()| output.observe(&publication.snapshot.path).map(|_| ()))
            .is_ok()
            && self.check_parent(&publication.snapshot.parent).is_ok()
            && storage::read(&publication.snapshot.path)
                .is_ok_and(|v| v.as_deref() == Some(publication.bytes.as_slice()))
    }
    fn uncertain_publication(&mut self, outcome: &Outcome) -> RpcError {
        let Outcome {
            concern,
            record,
            version,
            ..
        } = outcome;
        let evidence = format!(
            "Package state {concern}/{record} replacement committed at version {version}; durability/observation uncertain. Read current state before another mutation; do not assume rollback"
        );
        self.uncertain = Some(evidence.clone());
        conflict(&evidence)
    }
}
