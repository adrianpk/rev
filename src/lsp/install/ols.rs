//! A pinned upstream source build with the already installed Odin toolchain.
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::cache::Cache;
use super::helper::{Helper, executable};
use super::{Candidate, Outcome, Storage, package};
use crate::config::Server;
use crate::org::Cancellation;

const REPOSITORY: &str = "https://github.com/DanielGavin/ols.git";

struct Tools {
    git: PathBuf,
    odin: PathBuf,
    bash: PathBuf,
}

pub(super) fn install(
    storage: &mut Storage,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    package::platform()?;
    let path = std::env::var_os("PATH");
    let tools = Tools {
        git: executable("git", path.as_deref())?,
        odin: executable("odin", path.as_deref())?,
        bash: executable("bash", path.as_deref())?,
    };
    let mut cache = Cache::new(&storage.roots().cache)?;
    let result = tools.acquire(storage, &cache, inherited, cancel);
    cache.finish(result)
}

impl Tools {
    fn helper(&self, cache: &Cache, candidate: Option<&Candidate>, args: Vec<String>) -> Helper {
        Helper {
            executable: self.git.clone(),
            args,
            directory: cache.directory.clone(),
            env: [
                ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
                ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
                ("GIT_CONFIG_COUNT".into(), "0".into()),
                ("GIT_TERMINAL_PROMPT".into(), "0".into()),
            ]
            .into(),
            seconds: 180,
            watch: package::watch(cache, candidate),
        }
    }

    fn acquire(
        &self,
        storage: &mut Storage,
        cache: &Cache,
        inherited: Option<Server>,
        cancel: &Cancellation,
    ) -> Result<Outcome, String> {
        storage.progress.set(super::Phase::Resolving);
        let bytes = self
            .helper(
                cache,
                None,
                vec![
                    "ls-remote".into(),
                    "--exit-code".into(),
                    REPOSITORY.into(),
                    "refs/heads/master".into(),
                ],
            )
            .run_isolated(cancel, "GIT_")?;
        let resolved = package::text(&bytes)?;
        let fields: Vec<_> = resolved.split_whitespace().collect();
        if fields.len() != 2 || fields[1] != "refs/heads/master" || !revision(fields[0]) {
            return Err("OLS requires one exact immutable upstream revision".into());
        }
        let mut candidate = storage.candidate(
            "ols",
            fields[0],
            "linux-x86_64-odin",
            &format!("https://github.com/DanielGavin/ols/tree/{}", fields[0]),
        )?;
        let mut launch = package::launch(
            crate::lsp::catalog::launch("ols", "odin").expect("OLS catalog"),
            inherited,
        );
        launch.executable = "source/ols".into();
        launch.env.insert(
            "OLS_BUILTIN_FOLDER".into(),
            package::path(&candidate.directory.join("source/builtin"))?,
        );
        snippets(&mut launch.initialization_options)?;
        candidate.installation.launches.push(launch);
        let result = self.verify(cache, &mut candidate, cancel);
        package::publish(storage, candidate, result)
    }

    fn checkout(
        &self,
        cache: &Cache,
        candidate: &Candidate,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let source = candidate.directory.join("source");
        fs::create_dir(&source).map_err(|e| e.to_string())?;
        let git_dir = format!("--git-dir={}", package::path(&source.join(".git"))?);
        let work_tree = format!("--work-tree={}", package::path(&source)?);
        let commands = vec![
            vec!["init".into(), "--template=".into(), package::path(&source)?],
            vec![
                git_dir.clone(),
                work_tree.clone(),
                "-c".into(),
                "core.hooksPath=/dev/null".into(),
                "fetch".into(),
                "--depth=1".into(),
                "--no-tags".into(),
                REPOSITORY.into(),
                candidate.installation.version.clone(),
            ],
            vec![
                git_dir.clone(),
                work_tree.clone(),
                "-c".into(),
                "core.hooksPath=/dev/null".into(),
                "checkout".into(),
                "--detach".into(),
                candidate.installation.version.clone(),
            ],
            vec![git_dir, work_tree, "rev-parse".into(), "HEAD".into()],
        ];
        let mut head = Vec::new();
        for args in commands {
            head = self
                .helper(cache, Some(candidate), args)
                .run_isolated(cancel, "GIT_")?;
        }
        if package::text(&head)? != candidate.installation.version {
            return Err("OLS checkout differs from resolved revision".into());
        }
        Ok(())
    }

    fn verify(
        &self,
        cache: &Cache,
        candidate: &mut Candidate,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        candidate.progress.set(super::Phase::Acquiring);
        self.checkout(cache, candidate, cancel)?;
        candidate.progress.set(super::Phase::Building);
        let mut helper = self.helper(cache, Some(candidate), vec!["version".into()]);
        helper.executable = self.odin.clone();
        candidate.installation.runtime = package::text(&helper.run(cancel, None)?)?;
        let bin = self
            .odin
            .parent()
            .ok_or("installed Odin executable has no parent")?;
        let path = std::env::var_os("PATH").unwrap_or_default();
        let path = std::env::join_paths(
            std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(&path)),
        )
        .map_err(|e| e.to_string())?;
        helper.executable = self.bash.clone();
        helper.args = vec![
            "--noprofile".into(),
            "--norc".into(),
            "build.sh".into(),
            "release".into(),
        ];
        helper.directory = candidate.directory.join("source");
        helper.env = BTreeMap::from([
            ("PATH".into(), path.to_string_lossy().into_owned()),
            ("OLS_VERSION".into(), candidate.installation.version.clone()),
            ("BASH_ENV".into(), "/dev/null".into()),
            ("TMPDIR".into(), package::path(&cache.directory)?),
        ]);
        helper.seconds = 600;
        let output = helper.run_build(cancel).map_err(|e| {
            format!(
                "OLS build failed; provide an Odin toolchain compatible with this revision: {e}"
            )
        })?;
        built(candidate)
            .map_err(|e| format!("{e}; build output: {}", super::helper::diagnostic(&output)))?;
        candidate.installation.evidence =
            format!("git-revision:{}", candidate.installation.version);
        package::write(&candidate.directory.join("build-provenance.txt"),format!("source={}\nrevision={}\nruntime={}\ncommand=bash --noprofile --norc build.sh release\n",REPOSITORY,candidate.installation.version,candidate.installation.runtime).as_bytes())?;
        package::probe(candidate, cancel)
    }
}

fn built(candidate: &Candidate) -> Result<(), String> {
    let source = candidate.directory.join("source");
    let binary = source.join("ols");
    if !matches!(
        crate::lsp::catalog::executable(binary.to_str().unwrap_or(""), None),
        crate::lsp::catalog::Executable::Available(_)
    ) {
        return Err(
            "OLS build produced no executable; the installed Odin may be incompatible".into(),
        );
    }
    super::storage::contained(&candidate.directory, &source.join("builtin"), true)?;
    if !source.join("builtin").is_dir()
        || fs::read_dir(source.join("builtin"))
            .map_err(|e| e.to_string())?
            .next()
            .is_none()
    {
        return Err("OLS build lacks its required builtin collection".into());
    }
    Ok(())
}

pub(super) fn revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(super) fn snippets(options: &mut toml::Value) -> Result<(), String> {
    let table = options.as_table_mut().ok_or(
        "OLS initialization options must be a table to retain user options and disable snippets",
    )?;
    table.insert("enable_snippets".into(), toml::Value::Boolean(false));
    table.insert("enable_fake_methods".into(), toml::Value::Boolean(false));
    Ok(())
}
