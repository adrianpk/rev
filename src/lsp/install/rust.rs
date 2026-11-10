use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Server;
use crate::org::Cancellation;

use super::cache::Cache;
use super::download::{self, Exact};
use super::helper::{Helper, executable, owned_file};
use super::{Candidate, Launch, Outcome, State, Storage};

const API: &str = "https://api.github.com/repos/rust-lang/rust-analyzer/releases/latest";
const ASSET: &str = "rust-analyzer-x86_64-unknown-linux-gnu.gz";
const TARGET: &str = "x86_64-unknown-linux-gnu";

struct Tools {
    curl: PathBuf,
    gzip: PathBuf,
    sha256sum: PathBuf,
}

/// Explicit worker operation, independent of live LSP enablement. It returns
/// ready installation facts; selection remains a separate checked transaction.
pub fn install_rust(
    storage: &mut Storage,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    if std::env::consts::OS != "linux"
        || std::env::consts::ARCH != "x86_64"
        || !cfg!(target_env = "gnu")
    {
        return Err(
            "unsupported platform: managed Rust acquisition is verified on GNU/Linux x86-64".into(),
        );
    }
    storage.progress.set(super::Phase::Checking);
    let tools = Tools::discover()?;
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
    let metadata =
        download::curl(&tools.curl, API, &cache.directory, &cache.root, None).run(cancel, None)?;
    let exact = download::release(&metadata, "rust-lang/rust-analyzer", ASSET)?;
    let mut candidate = storage.candidate("rust-analyzer", &exact.version, TARGET, &exact.url)?;
    candidate.installation.launches.push(launch(inherited));
    let result = verify(tools, cache, &mut candidate, &exact, cancel);
    if let Err(error) = result {
        candidate.installation.status = State::Incomplete;
        let saved = storage.save(candidate.installation);
        return Err(incomplete(&error, saved));
    }
    candidate.installation.status = State::Ready;
    candidate.installation.verified_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs(),
    );
    let warning = storage.save(candidate.installation.clone())?;
    let prerequisites = rust_sources(cache, &candidate.installation.launches[0], cancel);
    let mut message = format!("Rust server {} ready", exact.version);
    if let Some(warning) = warning {
        message.push_str(&format!("; {warning}"));
    }
    if let Some(prerequisites) = prerequisites {
        message.push_str(&format!("; {prerequisites}"));
    }
    Ok(Outcome {
        installation: candidate.installation,
        selection: None,
        message,
    })
}

fn incomplete(error: &str, saved: Result<Option<String>, String>) -> String {
    match saved {
        Ok(None) => format!("{error}; candidate retained as incomplete"),
        Ok(Some(warning)) => format!("{error}; candidate marked incomplete; {warning}"),
        Err(save) => format!(
            "{error}; incomplete record publication failed: {save}; preserve preparing data"
        ),
    }
}

fn verify(
    tools: &Tools,
    cache: &Cache,
    candidate: &mut Candidate,
    exact: &Exact,
    cancel: &Cancellation,
) -> Result<(), String> {
    candidate.progress.set(super::Phase::Acquiring);
    let artifact = cache.directory.join("server.gz");
    download::curl(
        &tools.curl,
        &exact.url,
        &cache.directory,
        &cache.root,
        Some(&artifact),
    )
    .run(cancel, None)?;
    let metadata = fs::metadata(&artifact).map_err(|e| format!("downloaded asset: {e}"))?;
    if metadata.len() != exact.bytes || metadata.len() > super::ARTIFACT_BYTES {
        return Err("downloaded artifact length differs from the bounded release asset".into());
    }
    let digest = tools
        .helper(&tools.sha256sum, vec!["--".into(), path(&artifact)?], cache)
        .run(cancel, None)?;
    let actual = std::str::from_utf8(&digest)
        .map_err(|_| "invalid SHA-256 helper output")?
        .split_whitespace()
        .next()
        .unwrap_or("");
    if actual != exact.digest {
        return Err("downloaded artifact SHA-256 mismatch".into());
    }
    unpack(tools, cache, candidate, &artifact, cancel)?;
    candidate.progress.set(super::Phase::Verifying);
    candidate.installation.evidence = format!("sha256:{}", exact.digest);
    probe(candidate, cancel)
}

fn unpack(
    tools: &Tools,
    cache: &Cache,
    candidate: &Candidate,
    artifact: &Path,
    cancel: &Cancellation,
) -> Result<(), String> {
    let destination = candidate.directory.join("rust-analyzer");
    let output = owned_file(&destination)?;
    let mut helper = tools.helper(
        &tools.gzip,
        vec![
            "--decompress".into(),
            "--stdout".into(),
            "--".into(),
            path(artifact)?,
        ],
        cache,
    );
    helper
        .watch
        .push((candidate.directory.clone(), super::PAYLOAD_BYTES));
    helper.run(cancel, Some(output.try_clone().map_err(|e| e.to_string())?))?;
    output
        .set_permissions(fs::Permissions::from_mode(0o700))
        .and_then(|()| output.sync_all())
        .map_err(|e| format!("prepare executable durability: {e}"))?;
    fs::File::open(&candidate.directory)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| format!("prepare payload directory durability: {e}"))
}

fn probe(candidate: &mut Candidate, cancel: &Cancellation) -> Result<(), String> {
    let root = candidate.directory.join("probe-root");
    fs::create_dir(&root).map_err(|e| format!("create empty probe root: {e}"))?;
    let server = candidate.installation.launches[0].server(&candidate.directory);
    candidate.installation.runtime = runtime(&server, &root, &candidate.directory, cancel)?;
    if fs::read_dir(&root)
        .map_err(|e| e.to_string())?
        .next()
        .is_some()
    {
        return Err("protocol probe requires an empty owned root".into());
    }
    super::probe::verify(&server, &root, cancel)
        .map_err(|e| format!("Rust protocol probe failed: {e}"))?;
    super::tree::bytes(&candidate.directory, super::PAYLOAD_BYTES)?;
    fs::remove_dir_all(&root).map_err(|e| format!("clean owned probe root: {e}"))
}

fn runtime(
    server: &Server,
    root: &Path,
    payload: &Path,
    cancel: &Cancellation,
) -> Result<String, String> {
    let helper = Helper {
        executable: server.command.clone().into(),
        args: vec!["--version".into()],
        directory: root.to_owned(),
        env: server.env.clone(),
        seconds: 10,
        watch: vec![(payload.to_owned(), super::PAYLOAD_BYTES)],
    };
    let version = helper.run(cancel, None)?;
    let version = std::str::from_utf8(&version)
        .map_err(|_| "Rust server version is not UTF-8")?
        .trim();
    if version.is_empty() || version.len() > 4096 || version.contains('\0') {
        return Err("Rust server lacks a bounded runtime identity".into());
    }
    Ok(version.into())
}

fn launch(inherited: Option<Server>) -> Launch {
    let base = inherited.unwrap_or_else(|| crate::config::Lsp::default().servers["rust"].clone());
    Launch {
        id: "rust-analyzer".into(),
        languages: vec!["rust".into()],
        executable: "rust-analyzer".into(),
        args: Vec::new(),
        env: base.env,
        root_markers: base.root_markers,
        initialization_options: base.initialization_options,
        settings: base.settings,
    }
}

impl Tools {
    fn discover() -> Result<Self, String> {
        let path = std::env::var_os("PATH");
        Ok(Self {
            curl: executable("curl", path.as_deref())?,
            gzip: executable("gzip", path.as_deref())?,
            sha256sum: executable("sha256sum", path.as_deref())?,
        })
    }

    fn helper(&self, executable: &Path, args: Vec<String>, cache: &Cache) -> Helper {
        Helper {
            executable: executable.to_owned(),
            args,
            directory: cache.directory.clone(),
            env: Default::default(),
            seconds: 60,
            watch: vec![(cache.root.clone(), super::CACHE_BYTES)],
        }
    }
}

fn path(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "owned installation path is not UTF-8".into())
}

fn rust_sources(cache: &Cache, launch: &Launch, cancel: &Cancellation) -> Option<String> {
    let inherited = std::env::var_os("PATH");
    let path = launch
        .env
        .get("PATH")
        .map(std::ffi::OsStr::new)
        .or(inherited.as_deref());
    let rustc = match executable("rustc", path) {
        Ok(rustc) => rustc,
        Err(_) => {
            return Some(
                "Rust toolchain missing; install rustc, cargo and Rust sources separately".into(),
            );
        }
    };
    if executable("cargo", path).is_err() {
        return Some("cargo missing; install the Rust toolchain and sources separately".into());
    }
    let helper = Helper {
        executable: rustc,
        args: vec!["--print".into(), "sysroot".into()],
        directory: cache.directory.clone(),
        env: launch.env.clone(),
        seconds: 10,
        watch: Vec::new(),
    };
    let output = match helper.run(cancel, None) {
        Ok(output) => output,
        Err(_) => {
            return Some(
                "Rust toolchain/source check failed; install or repair it separately".into(),
            );
        }
    };
    let root =
        match String::from_utf8(output) {
            Ok(root) if root.len() <= 4096 && Path::new(root.trim()).is_absolute() => root,
            _ => return Some(
                "Rust source check returned an invalid sysroot; repair the toolchain separately"
                    .into(),
            ),
        };
    if !Path::new(root.trim())
        .join("lib/rustlib/src/rust/library")
        .is_dir()
    {
        return Some(
            "Rust sources missing; install rust-src separately for project analysis".into(),
        );
    }
    None
}
