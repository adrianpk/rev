use super::*;

impl Runner {
    pub(in crate::git::work) fn read_refs(
        &mut self,
        observation: Arc<Observation>,
        admission: &mut Lease,
    ) -> Result<Refs, String> {
        let before = self.ref_head(observation.identity.format)?;
        let compatible = head_compatible(
            observation.status.as_ref().map(|s| &s.head),
            &before.0,
            before.1,
        );
        if !compatible {
            return Err("HEAD changed between Status and Refs; refresh required".into());
        }

        let Catalog { entries, complete } =
            self.ref_catalog(observation.identity.format, admission)?;
        let RemoteCatalog {
            names: remotes,
            complete: remote_complete,
        } = self.remote_catalog(admission)?;
        let after = self.ref_head(observation.identity.format)?;
        if before != after {
            return Err("HEAD changed during ref observation".into());
        }
        let mut refs = Refs {
            id: self.progress.lock().expect("Git progress owner").operation,
            observation,
            entries,
            remotes,
            head_ref: before.0,
            head: before.1,
            completeness: if complete && remote_complete {
                Completeness::Complete
            } else {
                Completeness::Truncated {
                    reason: "max_refs or max_remotes",
                }
            },
            _admission: admission.split(0),
        };
        refs._admission = admission.split(refs.bytes());
        Ok(refs)
    }
    fn ref_catalog(
        &mut self,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<Catalog, String> {
        let count = format!("--count={}", self.policy.max_refs + 1);
        let bytes=self.command(&["for-each-ref", "--sort=refname", &count,
            "--format=%(refname)%00%(objectname)%00%(objecttype)%00%(symref)%00%(*objectname)%00%(*objecttype)%00%(upstream)%00%(upstream:track)%00",
            "refs/heads/","refs/remotes/","refs/tags/"],Phase::Refreshing,false)?;
        let count = bytes
            .iter()
            .filter(|b| **b == b'\n')
            .count()
            .min(self.policy.max_refs);
        admission.grow(count * std::mem::size_of::<Reference>() + 2 * bytes.len())?;
        Catalog::parse(&bytes, format, self.policy.max_refs, count)
    }
    fn remote_catalog(&mut self, admission: &mut Lease) -> Result<RemoteCatalog, String> {
        // Config keys disclose configured names without retaining URL values,
        // including credential-bearing remote descriptions.
        let keys = self.command(
            &[
                "config",
                "--null",
                "--name-only",
                "--get-regexp",
                "^remote\\..*\\.",
            ],
            Phase::Refreshing,
            true,
        )?;
        admission
            .grow(2 * keys.len() + self.policy.max_remotes * std::mem::size_of::<Vec<u8>>())?;
        RemoteCatalog::parse(&keys, self.policy.max_remotes)
    }
}

struct Catalog {
    entries: Vec<Reference>,
    complete: bool,
}
impl Catalog {
    fn parse(
        bytes: &[u8],
        format: ObjectFormat,
        limit: usize,
        count: usize,
    ) -> Result<Self, String> {
        let mut entries = Vec::with_capacity(count);
        let mut complete = true;
        for line in bytes.split_inclusive(|b| *b == b'\n') {
            if entries.len() == limit {
                complete = false;
                break;
            }
            entries.push(reference_record(line, format)?);
        }
        Ok(Self { entries, complete })
    }
}
struct RemoteCatalog {
    names: Vec<Vec<u8>>,
    complete: bool,
}
impl RemoteCatalog {
    fn parse(keys: &[u8], limit: usize) -> Result<Self, String> {
        let mut complete = true;
        let mut remotes = Vec::with_capacity(limit);
        if !keys.is_empty() && keys.last() != Some(&0) {
            return Err("Remote key list is incomplete".into());
        }
        for key in keys.split(|b| *b == 0).filter(|k| !k.is_empty()) {
            let name = remote_name(key)?;
            if remotes.iter().any(|r| r == name) {
                continue;
            }
            if remotes.len() == limit {
                complete = false;
                break;
            }
            remotes.push(name.to_vec());
        }
        Ok(Self {
            names: remotes,
            complete,
        })
    }
}
fn head_compatible(head: Option<&Head>, reference: &[u8], object: Option<ObjectId>) -> bool {
    match head {
        Some(Head::Branch {
            name,
            object: expected,
        }) => {
            reference.strip_prefix(b"refs/heads/") == Some(name.as_slice())
                && object == Some(*expected)
        }
        Some(Head::Unborn { branch }) => {
            reference.strip_prefix(b"refs/heads/") == Some(branch.as_slice()) && object.is_none()
        }
        Some(Head::Detached { object: expected }) => {
            reference.is_empty() && object == Some(*expected)
        }
        None => true,
    }
}
fn reference_record(line: &[u8], format: ObjectFormat) -> Result<Reference, String> {
    let fields = reference_fields(line)?;
    let kind = reference_kind(fields[0])?;
    Ok(Reference {
        name: fields[0].to_vec(),
        kind,
        object: ObjectId::parse(fields[1], format)?,
        object_type: fields[2].to_vec(),
        symbolic: fields[3].to_vec(),
        peeled: if fields[4].is_empty() {
            None
        } else {
            Some(ObjectId::parse(fields[4], format)?)
        },
        peeled_type: fields[5].to_vec(),
        upstream: fields[6].to_vec(),
        tracking: fields[7].to_vec(),
    })
}
fn reference_fields(line: &[u8]) -> Result<[&[u8]; 8], String> {
    let line = line
        .strip_suffix(b"\0\n")
        .ok_or("Refs record lacks its NUL/newline boundary")?;
    let mut parts = line.split(|b| *b == 0);
    let fields = std::array::from_fn::<_, 8, _>(|_| parts.next().unwrap_or(&[]));
    if parts.next().is_some()
        || line.iter().filter(|b| **b == 0).count() != 7
        || fields.iter().any(|f| f.len() > 4096)
    {
        return Err("Ref metadata exceeds its exact field bound".into());
    }
    Ok(fields)
}
fn reference_kind(name: &[u8]) -> Result<RefKind, String> {
    if name.starts_with(b"refs/heads/") {
        Ok(RefKind::Branch)
    } else if name.starts_with(b"refs/remotes/") {
        Ok(RefKind::Tracking)
    } else if name.starts_with(b"refs/tags/") {
        Ok(RefKind::Tag)
    } else {
        Err("Unexpected ref namespace".into())
    }
}
fn remote_name(key: &[u8]) -> Result<&[u8], String> {
    let key = key
        .strip_prefix(b"remote.")
        .ok_or("Malformed configured remote key")?;
    let end = key
        .iter()
        .rposition(|b| *b == b'.')
        .ok_or("Missing remote name boundary")?;
    let name = &key[..end];
    if name.is_empty() || name.len() > 4096 {
        return Err("Configured remote name exceeds metadata bound".into());
    }
    Ok(name)
}
