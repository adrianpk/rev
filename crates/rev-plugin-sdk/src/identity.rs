//! Validated public identities. Deserialization applies the same rules as parsing.

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidIdentity(pub &'static str);

impl fmt::Display for InvalidIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for InvalidIdentity {}

macro_rules! identity {
    ($name:ident, $doc:literal, $check:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, InvalidIdentity> {
                let value = value.into();
                if ($check)(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidIdentity(concat!("invalid ", stringify!($name))))
                }
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = InvalidIdentity;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

fn lexical(value: &str, limit: usize, first_letter: bool) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && (!first_letter || value.as_bytes()[0].is_ascii_lowercase())
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn package(value: &str) -> bool {
    value.len() <= 65
        && value.split_once('.').is_some_and(|(publisher, name)| {
            publisher != "rev" && lexical(publisher, 32, true) && lexical(name, 32, true)
        })
}
fn qualified(value: &str) -> bool {
    value
        .rsplit_once('.')
        .is_some_and(|(owner, name)| package(owner) && lexical(name, 64, false))
}
fn opaque(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_:.".contains(&c))
}
fn decimal(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|c| c.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
        && value.parse::<u64>().is_ok()
}

identity!(
    PackageId,
    "Two publisher.package segments; publisher rev is reserved.",
    package
);
identity!(
    CommandName,
    "Native 1..=64 lowercase letters, digits or hyphens.",
    |s: &str| lexical(s, 64, false)
);
identity!(
    CommandId,
    "Canonical publisher.package.command identity, at most 130 bytes.",
    qualified
);
identity!(
    RecordId,
    "Lexical package-state concern or record; never a filesystem path.",
    |s: &str| lexical(s, 64, false)
);
identity!(
    OpaqueId,
    "Connection-local opaque token, 1..=128 ASCII bytes.",
    opaque
);
identity!(
    Revision,
    "Opaque canonical decimal-string source or state revision.",
    decimal
);
identity!(
    Generation,
    "Opaque canonical decimal-string process generation.",
    decimal
);

impl PackageId {
    pub fn command(&self, name: &CommandName) -> CommandId {
        CommandId(format!("{}.{}", self.0, name.0))
    }
}
impl CommandId {
    pub fn belongs_to(&self, owner: &PackageId) -> bool {
        self.0
            .rsplit_once('.')
            .is_some_and(|(prefix, _)| prefix == owner.as_str())
    }
}

/// A native buffer or pane identity is meaningful only in its host session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Handle {
    pub session: OpaqueId,
    pub id: OpaqueId,
}

/// An observed source stamp; never substitute a newly observed revision on retry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    pub buffer: Handle,
    pub revision: Revision,
}

/// Scalar offsets form a half-open [start, end) interval; reversed ranges fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RangeFields", into = "RangeFields")]
pub struct ScalarRange {
    start: u64,
    end: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RangeFields {
    start: u64,
    end: u64,
}
impl ScalarRange {
    pub fn new(start: u64, end: u64) -> Result<Self, InvalidIdentity> {
        if start > end {
            return Err(InvalidIdentity("reversed scalar range"));
        }
        Ok(Self { start, end })
    }
    pub fn start(self) -> u64 {
        self.start
    }
    pub fn end(self) -> u64 {
        self.end
    }
}
impl TryFrom<RangeFields> for ScalarRange {
    type Error = InvalidIdentity;
    fn try_from(value: RangeFields) -> Result<Self, Self::Error> {
        Self::new(value.start, value.end)
    }
}
impl From<ScalarRange> for RangeFields {
    fn from(value: ScalarRange) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}
