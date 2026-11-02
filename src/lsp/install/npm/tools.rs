use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::super::{
    cache::Cache,
    helper::{Helper, executable},
    package,
};
use crate::org::Cancellation;

pub(super) struct Tools {
    pub node: PathBuf,
    npm: PathBuf,
    pub runtime: String,
    env: BTreeMap<String, String>,
}

impl Tools {
    pub fn discover(cache: &Cache, cancel: &Cancellation) -> Result<Self, String> {
        let path = std::env::var_os("PATH");
        let node = executable("node", path.as_deref())?;
        let npm = executable("npm", path.as_deref())?;
        configuration(cache)?;
        let mut tools = Self {
            node,
            npm,
            runtime: String::new(),
            env: [
                ("NODE_OPTIONS".into(), String::new()),
                ("NPM_CONFIG_UPDATE_NOTIFIER".into(), "false".into()),
                (
                    "TMPDIR".into(),
                    package::path(&cache.directory.join("tmp"))?,
                ),
            ]
            .into(),
        };
        tools.runtime = tools.identity(cache, cancel)?;
        Ok(tools)
    }

    fn identity(&self, cache: &Cache, cancel: &Cancellation) -> Result<String, String> {
        let mut helper = self.helper(cache, &cache.directory, &["--version".into()]);
        helper.seconds = 10;
        let npm = package::text(&helper.run(cancel, None)?)?;
        helper.executable = self.node.clone();
        helper.args = vec!["--version".into()];
        let node = package::text(&helper.run(cancel, None)?)?;
        Ok(format!("node {node}; npm {npm}"))
    }

    pub fn helper(&self, cache: &Cache, directory: &Path, args: &[String]) -> Helper {
        let mut options = [
            "--global=false",
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "--package-lock=true",
            "--save-exact",
            "--engine-strict",
            "--registry=https://registry.npmjs.org",
            "--workspaces=false",
            "--fetch-retries=0",
            "--fetch-timeout=60000",
            "--loglevel=error",
        ]
        .map(str::to_owned)
        .to_vec();
        for (key, path) in [
            ("--userconfig", cache.directory.join("user.npmrc")),
            ("--globalconfig", cache.directory.join("global.npmrc")),
            ("--cache", cache.directory.join("npm")),
            ("--prefix", directory.to_owned()),
        ] {
            options.push(format!("{key}={}", path.to_string_lossy()));
        }
        options.extend_from_slice(args);
        Helper {
            executable: self.npm.clone(),
            args: options,
            directory: directory.to_owned(),
            env: self.env.clone(),
            seconds: 300,
            watch: vec![(cache.root.clone(), super::super::CACHE_BYTES)],
        }
    }
}

fn configuration(cache: &Cache) -> Result<(), String> {
    package::path(&cache.directory)?;
    package::write(&cache.directory.join("user.npmrc"), b"")?;
    package::write(&cache.directory.join("global.npmrc"), b"")?;
    package::write(
        &cache.directory.join("package.json"),
        br#"{"name":"rev-owned-resolution","private":true}"#,
    )?;
    std::fs::create_dir(cache.directory.join("tmp"))
        .map_err(|e| format!("create owned npm temporary directory: {e}"))
}
