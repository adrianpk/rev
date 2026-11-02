use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::cache::Cache;
use super::helper::{Helper, executable, owned_file};
use super::{Candidate, Outcome, Storage, download, package};
use crate::config::Server;
use crate::org::Cancellation;

struct Tools {
    curl: PathBuf,
    sha: PathBuf,
}

pub(super) fn install(
    storage: &mut Storage,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    package::platform()?;
    let path = std::env::var_os("PATH");
    let tools = Tools {
        curl: executable("curl", path.as_deref())?,
        sha: executable("sha256sum", path.as_deref())?,
    };
    let mut cache = Cache::new(&storage.roots().cache)?;
    let result = acquire(storage, &tools, &cache, inherited, cancel);
    cache.finish(result)
}

fn acquire(
    storage: &mut Storage,
    tools: &Tools,
    cache: &Cache,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    storage.progress.set(super::Phase::Resolving);
    let bytes = download::curl(
        &tools.curl,
        "https://api.github.com/repos/artempyanykh/marksman/releases/latest",
        &cache.directory,
        &cache.root,
        None,
    )
    .run(cancel, None)?;
    let exact = download::release(&bytes, "artempyanykh/marksman", "marksman-linux-x64")?;
    let mut candidate =
        storage.candidate("marksman", &exact.version, "linux-x86_64", &exact.url)?;
    candidate.installation.launches.push(package::launch(
        crate::lsp::catalog::launch("marksman", "markdown").expect("Marksman catalog"),
        inherited,
    ));
    let result = verify(tools, cache, &mut candidate, &exact, cancel);
    package::publish(storage, candidate, result)
}

fn verify(
    tools: &Tools,
    cache: &Cache,
    candidate: &mut Candidate,
    exact: &download::Exact,
    cancel: &Cancellation,
) -> Result<(), String> {
    candidate.progress.set(super::Phase::Acquiring);
    let artifact = cache.directory.join("marksman");
    download::curl(
        &tools.curl,
        &exact.url,
        &cache.directory,
        &cache.root,
        Some(&artifact),
    )
    .run(cancel, None)?;
    if fs::metadata(&artifact).map_err(|e| e.to_string())?.len() != exact.bytes {
        return Err("downloaded Marksman length differs from release asset".into());
    }
    let helper = Helper {
        executable: tools.sha.clone(),
        args: vec!["--".into(), package::path(&artifact)?],
        directory: cache.directory.clone(),
        env: Default::default(),
        seconds: 10,
        watch: vec![(cache.root.clone(), super::CACHE_BYTES)],
    };
    let digest = helper.run(cancel, None)?;
    let digest = std::str::from_utf8(&digest)
        .map_err(|_| "invalid SHA-256 helper output")?
        .split_whitespace()
        .next()
        .unwrap_or("");
    if digest != exact.digest {
        return Err("downloaded Marksman SHA-256 mismatch".into());
    }
    copy(&artifact, &candidate.directory.join("marksman"))?;
    candidate.installation.evidence = format!("sha256:{digest}");
    candidate.installation.runtime = runtime(cache, candidate, cancel)?;
    package::probe(candidate, cancel)
}

fn copy(source: &std::path::Path, destination: &std::path::Path) -> Result<(), String> {
    let mut input = fs::File::open(source)
        .map_err(|e| e.to_string())?
        .take(super::ARTIFACT_BYTES + 1);
    let mut output = owned_file(destination)?;
    let count = std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
    if count > super::ARTIFACT_BYTES {
        return Err("Marksman native artifact exceeds 256 MiB".into());
    }
    output
        .set_permissions(fs::Permissions::from_mode(0o700))
        .and_then(|()| output.sync_all())
        .map_err(|e| format!("sync Marksman executable: {e}"))
}

fn runtime(cache: &Cache, candidate: &Candidate, cancel: &Cancellation) -> Result<String, String> {
    let helper = Helper {
        executable: candidate.directory.join("marksman"),
        args: vec!["--version".into()],
        directory: cache.directory.clone(),
        env: candidate.installation.launches[0].env.clone(),
        seconds: 10,
        watch: vec![(candidate.directory.clone(), super::PAYLOAD_BYTES)],
    };
    package::text(&helper.run(cancel, None)?)
}
