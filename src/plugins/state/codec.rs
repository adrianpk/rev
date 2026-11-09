//! Strict schema-1 envelope. JSON is one inert TOML string so nulls and every
//! JSON number/string/array/object roundtrip without TOML type reinterpretation.
use super::{invalid, limit};
use rev_plugin_sdk::{
    identity::RecordId,
    limits::STATE_BYTES,
    methods::{List, StateGetResult, Validate},
    rpc::RpcError,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fmt::Write};
use toml_edit::{ArrayOfTables, DocumentMut, Table, value};
pub(super) const BYTES: usize = 4 * 1024 * 1024;
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Catalog {
    pub schema_version: u32,
    pub revision: u64,
    pub records: List<Record, 256>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    pub concern: RecordId,
    pub record: RecordId,
    pub version: u64,
    pub value_json: String,
}
impl Catalog {
    pub fn decode(bytes: Option<&[u8]>) -> Result<Self, RpcError> {
        let Some(bytes) = bytes else {
            return Ok(Self {
                schema_version: 1,
                ..Self::default()
            });
        };
        if bytes.len() > BYTES {
            return Err(limit("Encoded package store exceeds 4 MiB"));
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| invalid("Package state must be UTF-8 TOML"))?;
        let catalog: Self = toml::from_str(text)
            .map_err(|_| invalid("Invalid package-state schema; existing bytes preserved"))?;
        catalog.validate()?;
        Ok(catalog)
    }
    fn validate(&self) -> Result<(), RpcError> {
        if self.schema_version != 1 || self.revision > i64::MAX as u64 {
            return Err(invalid("Unsupported package-state schema/revision"));
        }
        if self
            .records
            .windows(2)
            .any(|pair| (&pair[0].concern, &pair[0].record) >= (&pair[1].concern, &pair[1].record))
        {
            return Err(invalid(
                "Package-state records must be ordered by concern and record",
            ));
        }
        self.validate_records()
    }
    fn validate_records(&self) -> Result<(), RpcError> {
        let mut identities = BTreeSet::new();
        let mut versions = BTreeSet::new();
        for record in self.records.iter() {
            if !identities.insert((&record.concern, &record.record))
                || !versions.insert(record.version)
                || record.version == 0
                || record.version > self.revision
            {
                return Err(invalid(
                    "Duplicate or invalid package-state identity/version",
                ));
            }
            record.validate()?;
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, RpcError> {
        let mut doc = DocumentMut::new();
        doc["schema_version"] = value(1);
        doc["revision"] = value(
            i64::try_from(self.revision).expect("validated store revision fits TOML integer"),
        );
        if self.records.is_empty() {
            doc["records"] = value(toml_edit::Array::new());
        } else {
            let mut records = ArrayOfTables::new();
            for record in self.records.iter() {
                let mut table = Table::new();
                table["concern"] = value(record.concern.as_str());
                table["record"] = value(record.record.as_str());
                table["version"] = value(
                    i64::try_from(record.version)
                        .expect("validated record version fits TOML integer"),
                );
                table["value_json"] = value(&record.value_json);
                records.push(table);
            }
            doc["records"] = toml_edit::Item::ArrayOfTables(records);
        }
        struct Encoded(String);
        impl std::fmt::Write for Encoded {
            fn write_str(&mut self, value: &str) -> std::fmt::Result {
                if value.len() > BYTES.saturating_sub(self.0.len()) {
                    return Err(std::fmt::Error);
                }
                self.0.push_str(value);
                Ok(())
            }
        }
        let mut encoded = Encoded(String::new());
        write!(&mut encoded, "{doc}").map_err(|_| limit("Encoded package store exceeds 4 MiB"))?;
        Ok(encoded.0.into_bytes())
    }
}

impl Record {
    fn validate(&self) -> Result<(), RpcError> {
        if self.value_json.len() > STATE_BYTES {
            return Err(limit("State JSON record exceeds 256 KiB"));
        }
        let value: serde_json::Value = serde_json::from_str(&self.value_json)
            .map_err(|_| invalid("Invalid stored state JSON value"))?;
        // Compact spellings such as 1e1 can grow when encoded on the wire.
        StateGetResult::Present {
            version: super::version(self.version),
            value,
        }
        .validate(&Default::default())?;
        Ok(())
    }
}
