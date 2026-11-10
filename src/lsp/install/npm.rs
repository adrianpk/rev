mod lock;
mod tools;
mod version;

use std::collections::BTreeMap;

use crate::config::Server;
use crate::org::Cancellation;

use super::cache::Cache;
use super::package;
use super::{Candidate, Outcome, Storage};
use tools::Tools;

pub(super) struct Pin {
    name: &'static str,
    version: String,
}

/// Install one admitted npm recipe into its final owned prefix. All package
/// versions are exact, lifecycle scripts are disabled and the lock is retained.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Preserve the public owned inheritance request for one managed installation invocation."
)]
pub fn install_npm(
    storage: &mut Storage,
    recipe: &str,
    inherited: BTreeMap<String, Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    package::platform()?;
    package::path(&storage.roots().data)?;
    let id = match recipe {
        "typescript-language-server" => "typescript-language-server",
        "vscode-langservers-extracted" => "vscode-langservers-extracted",
        _ => return Err("unsupported managed npm recipe".into()),
    };
    storage.progress.set(super::Phase::Checking);
    let mut cache = Cache::new(&storage.roots().cache)?;
    let result = acquire(storage, id, &cache, &inherited, cancel);
    cache.finish(result)
}

fn acquire(
    storage: &mut Storage,
    recipe: &'static str,
    cache: &Cache,
    inherited: &BTreeMap<String, Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    let tools = Tools::discover(cache, cancel)?;
    storage.progress.set(super::Phase::Resolving);
    let pins = resolve(&tools, cache, recipe, cancel)?;
    let source = format!("https://registry.npmjs.org/{recipe}/{}", pins[0].version);
    let mut candidate =
        storage.candidate(recipe, &pins[0].version, "linux-x86_64-node", &source)?;
    launches(&mut candidate, recipe, inherited);
    let verified = verify(&tools, cache, &mut candidate, &pins, cancel);
    package::publish(storage, candidate, verified)
}

fn resolve(
    tools: &Tools,
    cache: &Cache,
    recipe: &'static str,
    cancel: &Cancellation,
) -> Result<Vec<Pin>, String> {
    let mut pins = vec![resolve_one(tools, cache, (recipe, "latest", None), cancel)?];
    if recipe == "typescript-language-server" {
        pins.push(resolve_one(
            tools,
            cache,
            ("typescript", "6", Some(6)),
            cancel,
        )?);
    }
    Ok(pins)
}

fn resolve_one(
    tools: &Tools,
    cache: &Cache,
    selector: (&'static str, &str, Option<u64>),
    cancel: &Cancellation,
) -> Result<Pin, String> {
    let (name, selector, major) = selector;
    let bytes = tools
        .helper(
            cache,
            &cache.directory,
            &[
                "view".into(),
                format!("{name}@{selector}"),
                "version".into(),
                "--json".into(),
            ],
        )
        .run(cancel, None)?;
    Ok(Pin {
        name,
        version: version::resolved(&bytes, major)?,
    })
}

fn launches(candidate: &mut Candidate, recipe: &str, inherited: &BTreeMap<String, Server>) {
    let recipe = crate::lsp::catalog::RECIPES
        .iter()
        .find(|r| r.id == recipe)
        .expect("admitted npm recipe");
    for profile in recipe.launches {
        let base = profile
            .languages
            .iter()
            .find_map(|language| inherited.get(*language))
            .cloned();
        let mut launch = package::launch(profile, base);
        launch.executable = std::path::Path::new("node_modules/.bin").join(profile.command);
        candidate.installation.launches.push(launch);
    }
}

fn verify(
    tools: &Tools,
    cache: &Cache,
    candidate: &mut Candidate,
    pins: &[Pin],
    cancel: &Cancellation,
) -> Result<(), String> {
    candidate.progress.set(super::Phase::Acquiring);
    package::write(
        &candidate.directory.join("package.json"),
        br#"{"name":"rev-managed-server","version":"0.0.0","private":true}"#,
    )?;
    let args = std::iter::once("install".into())
        .chain(pins.iter().map(|p| format!("{}@{}", p.name, p.version)))
        .collect::<Vec<_>>();
    let mut helper = tools.helper(cache, &candidate.directory, &args);
    helper
        .watch
        .push((candidate.directory.clone(), super::PAYLOAD_BYTES));
    helper.run(cancel, None).map_err(|e| {
        format!("npm acquisition failed; install compatible Node/npm separately if required: {e}")
    })?;
    let bytes = package::read(&candidate.directory, "package-lock.json")?;
    lock::admit(&bytes, pins)?;
    for pin in pins {
        lock::installed(&candidate.directory, pin)?;
    }
    record(tools, cache, candidate, pins, cancel)?;
    package::probe(candidate, cancel)
}

fn record(
    tools: &Tools,
    cache: &Cache,
    candidate: &mut Candidate,
    pins: &[Pin],
    cancel: &Cancellation,
) -> Result<(), String> {
    let script = "const f=require('node:fs'),c=require('node:crypto');process.stdout.write(c.createHash('sha256').update(f.readFileSync(process.argv[1])).digest('hex'))";
    let mut helper = tools.helper(cache, &candidate.directory, &[]);
    helper.executable = tools.node.clone();
    helper.args = vec![
        "--input-type=commonjs".into(),
        "--eval".into(),
        script.into(),
        package::path(&candidate.directory.join("package-lock.json"))?,
    ];
    helper.seconds = 10;
    let digest = package::text(&helper.run(cancel, None)?)?;
    if !super::evidence::sha256(&digest) {
        return Err("invalid npm package-lock SHA-256".into());
    }
    let pins = pins
        .iter()
        .map(|p| format!("{}@{}", p.name, p.version))
        .collect::<Vec<_>>();
    candidate.installation.runtime = format!("{}; {}", tools.runtime, pins.join("; "));
    candidate.installation.evidence = format!("npm-lock:sha256:{digest}");
    package::write(&candidate.directory.join("npm-provenance.json"), &serde_json::to_vec_pretty(
        &serde_json::json!({"runtime":tools.runtime,"packages":pins,"lock_sha256":digest,"registry":"https://registry.npmjs.org","scripts":false})
    ).map_err(|e| e.to_string())?)?;
    std::fs::File::open(candidate.directory.join("package-lock.json"))
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("sync npm lockfile: {e}"))
}
