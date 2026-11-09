//! Package-owned CAS records. Only supervisors call this bounded disk owner;
//! native input and configuration never perform package-state IO.
mod codec;
mod storage;
mod transaction;

use crate::{
    config::Roots,
    file::{Files, Output},
};
use rev_plugin_sdk::{identity::*, methods::*, rpc::*};
use std::path::PathBuf;

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
        operation.validate()?;
        let mutation = operation.is_mutation();
        if !live() {
            return Err(error(
                ErrorKind::Cancelled,
                "State request retired before admission",
            ));
        }
        self.admit_mutation(mutation)?;
        let Some(snapshot) = transaction::Snapshot::read(self, mutation)? else {
            return Self::read_result(&codec::Catalog::default(), &operation);
        };
        if !mutation {
            self.reconcile_read(&snapshot, output)?;
            return Self::read_result(&snapshot.catalog, &operation);
        }
        let publication = snapshot.prepare(operation)?;
        self.publish(&publication, live, output)
    }
    fn admit_mutation(&self, mutation: bool) -> Result<(), RpcError> {
        if mutation && let Some(evidence) = &self.uncertain {
            return Err(conflict(evidence));
        }
        Ok(())
    }
    fn read_result(
        catalog: &codec::Catalog,
        operation: &Operation,
    ) -> Result<serde_json::Value, RpcError> {
        match operation {
            Operation::Get(params) => Self::get_result(catalog, params),
            Operation::List(params) => Self::list_result(catalog, params),
            _ => unreachable!("an absent parent occurs only for read operations"),
        }
    }
    fn get_result(
        catalog: &codec::Catalog,
        params: &StateGetParams,
    ) -> Result<serde_json::Value, RpcError> {
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
    fn list_result(
        catalog: &codec::Catalog,
        params: &StateListParams,
    ) -> Result<serde_json::Value, RpcError> {
        let prefix = format!("state:{}:", params.concern);
        let after = state_cursor(&params.page.cursor, &prefix, catalog.revision)?;
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
}

impl Operation {
    fn is_mutation(&self) -> bool {
        matches!(self, Self::Put(_) | Self::Delete(_))
    }
    fn validate(&self) -> Result<(), RpcError> {
        match self {
            Self::Put(params) => params.validate(&Default::default()),
            Self::List(params) => params.validate(&Default::default()),
            _ => Ok(()),
        }
    }
}
fn state_cursor(cursor: &Option<OpaqueId>, prefix: &str, revision: u64) -> Result<usize, RpcError> {
    cursor
        .as_ref()
        .map(|cursor| cursor_position(cursor, prefix, revision))
        .transpose()
        .map(|position| position.unwrap_or(0))
}
fn cursor_position(
    cursor: &OpaqueId,
    prefix: &str,
    catalog_revision: u64,
) -> Result<usize, RpcError> {
    let (revision, index) = cursor
        .as_str()
        .strip_prefix(prefix)
        .and_then(|v| v.split_once(':'))
        .ok_or_else(|| invalid("Foreign or malformed state cursor"))?;
    let revision =
        Revision::new(revision).map_err(|_| invalid("Malformed state cursor revision"))?;
    if revision.as_str() != catalog_revision.to_string() {
        return Err(conflict(
            "Package store changed between pages; restart listing",
        ));
    }
    index
        .parse::<usize>()
        .ok()
        .filter(|i| *i <= 256)
        .ok_or_else(|| invalid("Malformed state cursor position"))
}

#[cfg(test)]
mod tests;
