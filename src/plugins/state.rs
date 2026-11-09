//! Package-owned CAS records. Only supervisors call this bounded disk owner;
//! native input and configuration never perform package-state IO.
mod codec;
mod storage;

use crate::{
    config::Roots,
    file::{Files, Output},
};
use rev_plugin_sdk::{identity::*, methods::*, rpc::*};
use std::{io::Write, path::PathBuf};

pub(crate) enum Operation {
    List(StateListParams),
    Get(StateGetParams),
    Put(StatePutParams),
    Delete(StateDeleteParams),
}
pub(crate) struct Store {
    path: PathBuf,
    uncertain: Option<String>,
}
fn error(kind: ErrorKind, message: &str) -> RpcError {
    let mut error = RpcError::application(kind);
    error.message = message.into();
    error
}
fn invalid(message: &str) -> RpcError {
    error(ErrorKind::InvalidArguments, message)
}
fn unavailable(message: &str) -> RpcError {
    error(ErrorKind::UnsupportedResource, message)
}
fn conflict(message: &str) -> RpcError {
    error(ErrorKind::StateConflict, message)
}
fn limit(message: &str) -> RpcError {
    error(ErrorKind::LimitExceeded, message)
}
fn version(value: u64) -> Revision {
    Revision::new(value.to_string()).expect("checked store revision")
}
fn value<T: serde::Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).expect("typed bounded state result")
}

pub(crate) fn root(roots: &Roots) -> Result<PathBuf, String> {
    if !roots.state.is_absolute()
        || roots
            .state
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(
            "Plugin state requires an absolute native state root without parent traversal".into(),
        );
    }
    Ok(roots.state.clone())
}
impl Store {
    pub fn new(root: &std::path::Path, package: &PackageId) -> Self {
        Self {
            path: root
                .join("plugins/packages")
                .join(package.as_str())
                .join("state.toml"),
            uncertain: None,
        }
    }
    pub fn execute(
        &mut self,
        operation: Operation,
        live: impl Fn() -> bool,
    ) -> Result<serde_json::Value, RpcError> {
        self.execute_with(operation, live, &mut Files)
    }
    fn execute_with(
        &mut self,
        operation: Operation,
        live: impl Fn() -> bool,
        output: &mut impl Output,
    ) -> Result<serde_json::Value, RpcError> {
        let put = matches!(&operation, Operation::Put(_));
        let mutation = put || matches!(&operation, Operation::Delete(_));
        if let Operation::Put(params) = &operation {
            params.validate(&Default::default())?;
        }
        if let Operation::List(params) = &operation {
            params.validate(&Default::default())?;
        }
        if !live() {
            return Err(error(
                ErrorKind::Cancelled,
                "State request retired before admission",
            ));
        }
        if mutation && let Some(evidence) = &self.uncertain {
            return Err(conflict(evidence));
        }
        let parent = self.parent(mutation)?;
        let Some(parent) = parent else {
            return Self::read_result(codec::Catalog::default(), operation);
        };
        storage::private(&parent)?;
        let base = storage::anchor(&parent);
        let lock = storage::lock(&base.join("state.toml.lock"), mutation)?;
        self.check_parent(&parent)?;
        let path = base.join("state.toml");
        let before = storage::read(&path)?;
        let mut catalog = codec::Catalog::decode(before.as_deref())?;
        if !mutation {
            self.check_parent(&parent)?;
            if let Some(lock) = &lock {
                lock.check(&base.join("state.toml.lock"))?;
            }
            if self.uncertain.is_some() {
                output.sync_directory(&parent).map_err(|_| {
                    conflict(self.uncertain.as_deref().expect("uncertain publication"))
                })?;
                self.check_parent(&parent)?;
                self.uncertain = None;
            }
            return Self::read_result(catalog, operation);
        }
        let next = catalog
            .revision
            .checked_add(1)
            .filter(|v| *v <= i64::MAX as u64)
            .ok_or_else(|| limit("Package store revision exhausted"))?;
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
        let mut records = catalog.records.into_vec();
        let index = records
            .iter()
            .position(|r| r.concern == concern && r.record == record);
        match (index, expected) {
            (None, None) if replacement.is_some() => {}
            (Some(i), Some(expected)) if records[i].version.to_string() == expected.as_str() => {}
            _ => {
                return Err(conflict(
                    "Package state record version changed; read current state before retrying",
                ));
            }
        }
        match (index, replacement) {
            (Some(i), Some(json)) => {
                records[i] = codec::Record {
                    concern: concern.clone(),
                    record: record.clone(),
                    version: next,
                    value_json: json,
                }
            }
            (None, Some(json)) => records.push(codec::Record {
                concern: concern.clone(),
                record: record.clone(),
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
        let bytes = catalog.encode()?;
        let mut temporary = storage::Staging::new(&base)?;
        temporary
            .file()
            .write_all(&bytes)
            .and_then(|()| output.flush(temporary.file()))
            .and_then(|()| output.sync_file(temporary.file()))
            .map_err(|_| {
                unavailable("Cannot synchronize staged package state; destination unchanged")
            })?;
        self.check_parent(&parent)?;
        lock.as_ref()
            .expect("mutation owns a lock")
            .check(&base.join("state.toml.lock"))?;
        temporary.check()?;
        if storage::read(&path)? != before {
            return Err(conflict("Package store changed before replacement"));
        }
        if !live() {
            return Err(error(
                ErrorKind::Cancelled,
                "State request retired before publication",
            ));
        }
        output
            .replace(temporary.path(), &path)
            .map_err(|_| unavailable("Cannot replace package state; destination unchanged"))?;
        temporary.disarm();
        let proven = output
            .sync_directory(&parent)
            .and_then(|()| output.observe(&path).map(|_| ()))
            .is_ok()
            && self.check_parent(&parent).is_ok()
            && storage::read(&path).is_ok_and(|v| v.as_deref() == Some(bytes.as_slice()));
        if !proven {
            let evidence = format!(
                "Package state {concern}/{record} replacement committed at version {next}; durability/observation uncertain. Read current state before another mutation; do not assume rollback"
            );
            self.uncertain = Some(evidence.clone());
            return Err(conflict(&evidence));
        }
        Ok(if put {
            value(StatePutResult {
                version: version(next),
            })
        } else {
            value(Empty {})
        })
    }
    fn read_result(
        catalog: codec::Catalog,
        operation: Operation,
    ) -> Result<serde_json::Value, RpcError> {
        match operation {
            Operation::Get(params) => {
                let record = catalog
                    .records
                    .iter()
                    .find(|r| r.concern == params.concern && r.record == params.record);
                Ok(value(match record {
                    None => StateGetResult::Absent,
                    Some(record) => StateGetResult::Present {
                        version: version(record.version),
                        value: serde_json::from_str(&record.value_json)
                            .map_err(|_| invalid("Invalid stored JSON"))?,
                    },
                }))
            }
            Operation::List(params) => {
                let prefix = format!("state:{}:", params.concern);
                let after = params
                    .page
                    .cursor
                    .as_ref()
                    .map(|cursor| {
                        let (revision, index) = cursor
                            .as_str()
                            .strip_prefix(&prefix)
                            .and_then(|v| v.split_once(':'))
                            .ok_or_else(|| invalid("Foreign or malformed state cursor"))?;
                        let revision = Revision::new(revision)
                            .map_err(|_| invalid("Malformed state cursor revision"))?;
                        if revision.as_str() != catalog.revision.to_string() {
                            return Err(conflict(
                                "Package store changed between pages; restart listing",
                            ));
                        }
                        index
                            .parse::<usize>()
                            .ok()
                            .filter(|i| *i <= 256)
                            .ok_or_else(|| invalid("Malformed state cursor position"))
                    })
                    .transpose()?
                    .unwrap_or(0);
                let filtered: Vec<_> = catalog
                    .records
                    .iter()
                    .filter(|r| r.concern == params.concern)
                    .collect();
                if after > filtered.len() {
                    return Err(invalid("State cursor outside concern"));
                }
                let end = after
                    .saturating_add(params.page.limit.into())
                    .min(filtered.len());
                let records = filtered[after..end]
                    .iter()
                    .map(|r| StateEntry {
                        record: r.record.clone(),
                        version: version(r.version),
                    })
                    .collect();
                let next = (end < filtered.len()).then(|| {
                    OpaqueId::new(format!("{prefix}{}:{end}", catalog.revision))
                        .expect("bounded concern/revision/position cursor")
                });
                Ok(value(StateListResult {
                    records: List::new(records)?,
                    next,
                }))
            }
            _ => unreachable!("an absent parent occurs only for read operations"),
        }
    }
}

#[cfg(test)]
mod tests;
