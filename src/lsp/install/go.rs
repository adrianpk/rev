use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::config::Server;
use crate::org::Cancellation;

use super::cache::Cache;
use super::helper::{Helper, executable};
use super::package::{path, text};
use super::{Candidate, Outcome, Storage, package};

const MODULE: &str = "golang.org/x/tools/gopls";

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Module {
    path: String,
    version: String,
    #[serde(default)]
    sum: String,
    #[serde(default)]
    go_mod_sum: String,
    error: Option<serde_json::Value>,
    replace: Option<serde_json::Value>,
}

struct Tools {
    go: PathBuf,
    env: BTreeMap<String, String>,
}

/// Resolve once and build with the installed local Go compiler. Caches and the
/// final GOBIN belong to this operation; no user Go configuration is changed.
pub fn install_go(
    storage: &mut Storage,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    package::platform()?;
    storage.progress.set(super::Phase::Checking);
    let go = executable("go", std::env::var_os("PATH").as_deref())?;
    let mut cache = Cache::new(&storage.roots().cache)?;
    let result = acquire(storage, go, &cache, inherited, cancel);
    cache.finish(result)
}

fn acquire(
    storage: &mut Storage,
    go: PathBuf,
    cache: &Cache,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    let tools = Tools::new(go, cache)?;
    let toolchain = text(&tools.helper(cache, &["version"]).run(cancel, None)?)?;
    storage.progress.set(super::Phase::Resolving);
    let metadata = tools
        .helper(cache, &["list", "-m", "-json", &format!("{MODULE}@latest")])
        .run(cancel, None)
        .map_err(|error| prerequisite(&error))?;
    let exact = module(&metadata, None)?;
    let source = format!(
        "https://proxy.golang.org/{MODULE}/@v/{}.info",
        exact.version
    );
    let mut candidate = storage.candidate("gopls", &exact.version, "linux-amd64", &source)?;
    let launch = crate::lsp::catalog::launch("gopls", "go").expect("built-in Go recipe");
    candidate
        .installation
        .launches
        .push(package::launch(launch, inherited));
    let verified = verify(
        &tools,
        cache,
        &mut candidate,
        (&exact.version, &toolchain),
        cancel,
    );
    package::publish(storage, candidate, verified)
}

fn verify(
    tools: &Tools,
    cache: &Cache,
    candidate: &mut Candidate,
    resolved: (&str, &str),
    cancel: &Cancellation,
) -> Result<(), String> {
    candidate.progress.set(super::Phase::Acquiring);
    let (version, toolchain) = resolved;
    let target = format!("{MODULE}@{version}");
    let metadata = tools
        .helper(cache, &["mod", "download", "-json", &target])
        .run(cancel, None)
        .map_err(|error| prerequisite(&error))?;
    let exact = module(&metadata, Some(version))?;
    if !super::evidence::go_sum(&exact.sum) || !super::evidence::go_sum(&exact.go_mod_sum) {
        return Err("Go module download lacks checksum-database provenance".into());
    }
    package::write(&candidate.directory.join("go-module.json"), &serde_json::to_vec_pretty(
        &serde_json::json!({"path":MODULE,"version":version,"sum":exact.sum,"go_mod_sum":exact.go_mod_sum,"toolchain":toolchain})
    ).map_err(|e| e.to_string())?)?;
    build(tools, cache, candidate, &target, cancel)?;
    identity(tools, cache, candidate, &exact, cancel)?;
    package::probe(candidate, cancel)
}

fn build(
    tools: &Tools,
    cache: &Cache,
    candidate: &Candidate,
    target: &str,
    cancel: &Cancellation,
) -> Result<(), String> {
    candidate.progress.set(super::Phase::Building);
    let mut helper = tools.helper(cache, &["install", "-trimpath", target]);
    helper
        .env
        .insert("GOBIN".into(), path(&candidate.directory)?);
    helper
        .watch
        .push((candidate.directory.clone(), super::PAYLOAD_BYTES));
    helper
        .run(cancel, None)
        .map_err(|error| prerequisite(&error))?;
    fs::File::open(candidate.directory.join("gopls"))
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("sync built Go server: {e}"))
}

fn identity(
    tools: &Tools,
    cache: &Cache,
    candidate: &mut Candidate,
    exact: &Module,
    cancel: &Cancellation,
) -> Result<(), String> {
    let binary = path(&candidate.directory.join("gopls"))?;
    let bytes = tools
        .helper(cache, &["version", "-m", "-json", &binary])
        .run(cancel, None)?;
    let value = build_info(&bytes, exact)?;
    package::write(&candidate.directory.join("go-build-info.json"), &bytes)?;
    let mut helper = tools.helper(cache, &["version"]);
    helper.executable = binary.into();
    helper.seconds = 10;
    let identity = text(&helper.run(cancel, None)?)?;
    candidate.installation.runtime = format!(
        "{identity}; {}",
        value["GoVersion"]
            .as_str()
            .ok_or("Go build info lacks compiler identity")?
    );
    candidate.installation.evidence = format!("go-module:{}", exact.sum);
    Ok(())
}

fn build_info(bytes: &[u8], exact: &Module) -> Result<serde_json::Value, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "invalid Go binary build info")?;
    let main = serde_json::to_vec(&value["Main"]).map_err(|e| e.to_string())?;
    let built = module(&main, Some(&exact.version))?;
    if built.sum != exact.sum || value["Path"] != MODULE {
        return Err("built Go server differs from resolved module provenance".into());
    }
    Ok(value)
}

fn module(bytes: &[u8], version: Option<&str>) -> Result<Module, String> {
    let module: Module =
        serde_json::from_slice(bytes).map_err(|_| "invalid exact Go module metadata")?;
    if module.path != MODULE || module.error.is_some() || module.replace.is_some() {
        return Err("Go module metadata differs from the official unreplaced gopls module".into());
    }
    if !version_valid(&module.version) || version.is_some_and(|version| module.version != version) {
        return Err("Go module metadata lacks the resolved exact stable version".into());
    }
    Ok(module)
}

fn version_valid(version: &str) -> bool {
    let Some(version) = version.strip_prefix('v') else {
        return false;
    };
    version.len() < 128
        && version.split('.').count() == 3
        && version
            .split('.')
            .all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn prerequisite(error: &str) -> String {
    format!(
        "Go acquisition failed with the installed local compiler; install a sufficient Go version separately if required: {error}"
    )
}

impl Tools {
    fn new(go: PathBuf, cache: &Cache) -> Result<Self, String> {
        let mut env: BTreeMap<String, String> = [
            ("GOTOOLCHAIN", "local"),
            ("GOENV", "off"),
            ("GOWORK", "off"),
            ("GOFLAGS", "-modcacherw -p=2"),
            ("GOOS", "linux"),
            ("GOARCH", "amd64"),
            ("GOAMD64", "v1"),
            ("GOEXPERIMENT", ""),
            ("CGO_ENABLED", "0"),
            ("GOPROXY", "https://proxy.golang.org"),
            ("GOSUMDB", "sum.golang.org"),
            ("GONOSUMDB", ""),
            ("GONOPROXY", ""),
            ("GOPRIVATE", ""),
            ("GOTELEMETRY", "off"),
            ("GOMAXPROCS", "2"),
            ("GO111MODULE", "on"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        for (key, directory) in [
            ("GOPATH", "gopath"),
            ("GOMODCACHE", "modules"),
            ("GOCACHE", "build"),
            ("GOTMPDIR", "tmp"),
        ] {
            let directory = cache.directory.join(directory);
            fs::create_dir(&directory).map_err(|e| format!("create owned Go cache: {e}"))?;
            env.insert(key.into(), path(&directory)?);
        }
        Ok(Self { go, env })
    }

    fn helper(&self, cache: &Cache, args: &[&str]) -> Helper {
        Helper {
            executable: self.go.clone(),
            args: args.iter().map(|s| (*s).into()).collect(),
            directory: cache.directory.clone(),
            env: self.env.clone(),
            seconds: 600,
            watch: vec![(cache.root.clone(), super::CACHE_BYTES)],
        }
    }
}
