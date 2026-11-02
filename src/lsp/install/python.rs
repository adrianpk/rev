use super::cache::Cache;
use super::helper::{Helper, executable};
use super::{Candidate, Outcome, Storage, package};
use crate::config::Server;
use crate::org::Cancellation;

const RESOLVE: &str = "import urllib.request,json; r=urllib.request.urlopen('https://pypi.org/pypi/python-lsp-server/json',timeout=30); b=r.read(1048577); assert len(b)<=1048576,'metadata exceeds 1 MiB'; print(json.loads(b)['info']['version'])";
const FACTS: &str = "import importlib.metadata as m,hashlib,json,sys; print(json.dumps({'version':m.version('python-lsp-server'),'runtime':sys.version.splitlines()[0],'report_sha256':hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest()}))";

pub(super) fn install(
    storage: &mut Storage,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    package::platform()?;
    let python = executable("python3", std::env::var_os("PATH").as_deref())?;
    let mut cache = Cache::new(&storage.roots().cache)?;
    let result = acquire(storage, &python, &cache, inherited, cancel);
    cache.finish(result)
}

fn helper(
    python: &std::path::Path,
    cache: &Cache,
    candidate: Option<&Candidate>,
    args: Vec<String>,
) -> Helper {
    Helper {
        executable: python.to_owned(),
        args,
        directory: cache.directory.clone(),
        env: [
            ("PIP_CONFIG_FILE".into(), "/dev/null".into()),
            ("PIP_DISABLE_PIP_VERSION_CHECK".into(), "1".into()),
            ("PYTHONNOUSERSITE".into(), "1".into()),
            (
                "TMPDIR".into(),
                cache.directory.to_string_lossy().into_owned(),
            ),
        ]
        .into(),
        seconds: 300,
        watch: package::watch(cache, candidate),
    }
}

fn acquire(
    storage: &mut Storage,
    python: &std::path::Path,
    cache: &Cache,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    storage.progress.set(super::Phase::Resolving);
    let version = package::text(
        &helper(
            python,
            cache,
            None,
            vec!["-I".into(), "-c".into(), RESOLVE.into()],
        )
        .run(cancel, None)?,
    )?;
    if !package::version(&version) {
        return Err("PyPI lacks an exact stable version".into());
    }
    let mut candidate = storage.candidate(
        "python-lsp-server",
        &version,
        "linux-x86_64-python",
        "https://pypi.org/project/python-lsp-server/",
    )?;
    let mut launch = package::launch(
        crate::lsp::catalog::launch("pylsp", "python").expect("Python catalog"),
        inherited,
    );
    launch.executable = "venv/bin/pylsp".into();
    candidate.installation.launches.push(launch);
    let result = verify(python, cache, &mut candidate, cancel);
    package::publish(storage, candidate, result)
}

fn verify(
    python: &std::path::Path,
    cache: &Cache,
    candidate: &mut Candidate,
    cancel: &Cancellation,
) -> Result<(), String> {
    candidate.progress.set(super::Phase::Acquiring);
    let venv = candidate.directory.join("venv");
    helper(
        python,
        cache,
        Some(candidate),
        vec![
            "-I".into(),
            "-m".into(),
            "venv".into(),
            "--copies".into(),
            package::path(&venv)?,
        ],
    )
    .run(cancel, None)
    .map_err(|e| {
        format!(
            "Python venv/pip unavailable; install them for the selected interpreter separately: {e}"
        )
    })?;
    let python = venv.join("bin/python");
    let args = vec![
        "-I".into(),
        "-m".into(),
        "pip".into(),
        "--isolated".into(),
        "--disable-pip-version-check".into(),
        "--no-input".into(),
        "--cache-dir".into(),
        package::path(&cache.directory.join("pip"))?,
        "install".into(),
        "--only-binary=:all:".into(),
        "--index-url=https://pypi.org/simple".into(),
        "--timeout=30".into(),
        "--retries=0".into(),
        "--report".into(),
        package::path(&candidate.directory.join("pip-report.json"))?,
        format!("python-lsp-server=={}", candidate.installation.version),
    ];
    helper(&python,cache,Some(candidate),args).run(cancel,None)
        .map_err(|e|format!("minimal Python server installation failed; provide a compatible interpreter and wheels: {e}"))?;
    let bytes = package::read(&candidate.directory, "pip-report.json")?;
    admit(&bytes, &candidate.installation.version)?;
    facts(&python, cache, candidate, cancel)?;
    package::probe(candidate, cancel)
}

fn facts(
    python: &std::path::Path,
    cache: &Cache,
    candidate: &mut Candidate,
    cancel: &Cancellation,
) -> Result<(), String> {
    let bytes = helper(
        python,
        cache,
        Some(candidate),
        vec![
            "-I".into(),
            "-c".into(),
            FACTS.into(),
            package::path(&candidate.directory.join("pip-report.json"))?,
        ],
    )
    .run(cancel, None)?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid Python package identity")?;
    if value["version"] != candidate.installation.version
        || !value["report_sha256"]
            .as_str()
            .is_some_and(super::evidence::sha256)
    {
        return Err("installed Python package/report differs from resolved identity".into());
    }
    candidate.installation.evidence = format!(
        "pip-report:sha256:{}",
        value["report_sha256"].as_str().expect("checked digest")
    );
    candidate.installation.runtime = value["runtime"]
        .as_str()
        .ok_or("Python runtime identity missing")?
        .into();
    package::write(&candidate.directory.join("python-packages.json"), &bytes)
}

fn admit(bytes: &[u8], version: &str) -> Result<(), String> {
    let report: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "invalid pip installation report")?;
    let packages = report["install"]
        .as_array()
        .filter(|p| !p.is_empty() && p.len() <= 256)
        .ok_or("pip report requires 1..=256 packages")?;
    if !packages.iter().any(|p| {
        p["metadata"]["name"].as_str().is_some_and(|s| {
            s.replace('_', "-")
                .eq_ignore_ascii_case("python-lsp-server")
        }) && p["metadata"]["version"] == version
    }) {
        return Err("pip report differs from the resolved base package".into());
    }
    for package in packages {
        provenance(package)?;
    }
    Ok(())
}

fn provenance(value: &serde_json::Value) -> Result<(), String> {
    let download = &value["download_info"];
    if !download["url"]
        .as_str()
        .is_some_and(|s| s.starts_with("https://files.pythonhosted.org/"))
        || !download["archive_info"]["hashes"]["sha256"]
            .as_str()
            .is_some_and(super::evidence::sha256)
    {
        return Err("pip report requires official HTTPS wheel sources and SHA-256".into());
    }
    if value["requested_extras"]
        .as_array()
        .is_some_and(|v| !v.is_empty())
        || !value["metadata"]["version"]
            .as_str()
            .is_some_and(package::version)
    {
        return Err("pip report requires exact base packages without optional extras".into());
    }
    Ok(())
}
