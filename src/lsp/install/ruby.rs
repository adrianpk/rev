use super::cache::Cache;
use super::helper::{Helper, executable};
use super::{Candidate, Outcome, Storage, package};
use crate::config::Server;
use crate::org::Cancellation;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const RESOLVE: &str = r#"require 'net/http';require 'json';require 'bundler'; u=URI('https://rubygems.org/api/v1/gems/ruby-lsp.json'); b=+''; Net::HTTP.start(u.host,u.port,use_ssl:true,open_timeout:10,read_timeout:30){|h|h.request_get(u.request_uri){|r|raise 'RubyGems HTTPS response failed' unless r.is_a?(Net::HTTPSuccess);r.read_body{|s|raise 'metadata exceeds 1 MiB' if b.bytesize+s.bytesize>1048576;b<<s}}};j=JSON.parse(b);j['bundler_version']=Bundler::VERSION;print JSON.generate(j)"#;
const FACTS: &str = r#"require 'json';require 'digest'; home,version=ARGV; packages=Gem::Specification.sort_by(&:name).map{|s|{name:s.name,version:s.version.to_s,platform:s.platform.to_s,default:s.default_gem?}};puts JSON.generate({runtime:RUBY_DESCRIPTION,packages:packages,lock_sha256:Digest::SHA256.file(File.join(home,'Gemfile.lock')).hexdigest,primary_sha256:Digest::SHA256.file(File.join(home,'gems','cache',"ruby-lsp-#{version}.gem")).hexdigest})"#;
const LOCK: &str =
    "require 'bundler'; require 'bundler/cli'; Bundler::CLI.start(['lock','--local'])";

struct Exact {
    version: String,
    bundler: String,
    digest: String,
}

struct Tools {
    ruby: PathBuf,
    gem: PathBuf,
    env: BTreeMap<String, String>,
}

pub(super) fn install(
    storage: &mut Storage,
    inherited: Option<Server>,
    cancel: &Cancellation,
) -> Result<Outcome, String> {
    package::platform()?;
    let path = std::env::var_os("PATH");
    let mut tools = Tools {
        ruby: executable("ruby", path.as_deref())?,
        gem: executable("gem", path.as_deref())?,
        env: Default::default(),
    };
    let mut cache = Cache::new(&storage.roots().cache)?;
    let result = tools.acquire(storage, &cache, inherited, cancel);
    cache.finish(result)
}

impl Tools {
    fn acquire(
        &mut self,
        storage: &mut Storage,
        cache: &Cache,
        inherited: Option<Server>,
        cancel: &Cancellation,
    ) -> Result<Outcome, String> {
        self.environment(cache)?;
        storage.progress.set(super::Phase::Resolving);
        let bytes = self
            .helper(cache, None, vec!["-e".into(), RESOLVE.into()])
            .run(cancel, None)?;
        let exact = resolution(&bytes)?;
        let mut candidate = storage.candidate(
            "ruby-lsp",
            &exact.version,
            "linux-x86_64-ruby",
            "https://rubygems.org/gems/ruby-lsp",
        )?;
        let mut launch = package::launch(
            crate::lsp::catalog::launch("ruby-lsp", "ruby").expect("Ruby catalog"),
            inherited,
        );
        launch.executable = "bin/ruby-lsp".into();
        launch.env.extend(runtime_env(&candidate.directory)?);
        candidate.installation.launches.push(launch);
        let result = self.verify(cache, &mut candidate, &exact, cancel);
        package::publish(storage, candidate, result)
    }

    fn environment(&mut self, cache: &Cache) -> Result<(), String> {
        let temporary = cache.directory.join("tmp");
        fs::create_dir(&temporary).map_err(|e| e.to_string())?;
        package::write(&cache.directory.join("gemrc"), b"")?;
        self.env = [
            ("MAKEFLAGS".into(), "-j1 --jobserver-style=pipe".into()),
            ("MAKE".into(), "make -j1 --jobserver-style=pipe".into()),
            ("RUBYOPT".into(), String::new()),
            ("RUBYLIB".into(), String::new()),
            ("RUBYGEMS_GEMDEPS".into(), String::new()),
            (
                "GEMRC".into(),
                package::path(&cache.directory.join("gemrc"))?,
            ),
            (
                "GEM_SPEC_CACHE".into(),
                package::path(&cache.directory.join("spec-cache"))?,
            ),
            ("TMPDIR".into(), package::path(&temporary)?),
        ]
        .into();
        Ok(())
    }

    fn helper(&self, cache: &Cache, candidate: Option<&Candidate>, args: Vec<String>) -> Helper {
        let mut env = self.env.clone();
        if let Some(candidate) = candidate {
            env.extend(candidate.installation.launches[0].env.clone());
        }
        env.insert("BUNDLE_FROZEN".into(), "false".into());
        Helper {
            executable: self.ruby.clone(),
            args,
            directory: cache.directory.clone(),
            env,
            seconds: 300,
            watch: package::watch(cache, candidate),
        }
    }

    fn verify(
        &self,
        cache: &Cache,
        candidate: &mut Candidate,
        exact: &Exact,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        candidate.progress.set(super::Phase::Acquiring);
        let manifest = format!(
            "source 'https://rubygems.org'\ngem 'ruby-lsp', '= {}'\ngem 'bundler', '= {}'\n",
            candidate.installation.version, exact.bundler
        );
        package::write(&candidate.directory.join("Gemfile"), manifest.as_bytes())?;
        let mut helper = self.helper(
            cache,
            Some(candidate),
            vec![
                "install".into(),
                "--norc".into(),
                format!("ruby-lsp:{}", candidate.installation.version),
                format!("bundler:{}", exact.bundler),
                "--no-document".into(),
                "--clear-sources".into(),
                "--source".into(),
                "https://rubygems.org".into(),
                "--install-dir".into(),
                package::path(&candidate.directory.join("gems"))?,
                "--bindir".into(),
                package::path(&candidate.directory.join("bin"))?,
            ],
        );
        helper.executable = self.gem.clone();
        helper.run(cancel,None).map_err(|e|format!("Ruby gem installation failed; install compatible Ruby and required native build tools separately: {e}"))?;
        self.helper(cache, Some(candidate), vec!["-e".into(), LOCK.into()])
            .run(cancel, None)
            .map_err(|e| {
                format!(
                    "isolated Ruby bundle lock failed; provide Bundler for the installed Ruby: {e}"
                )
            })?;
        self.facts(cache, candidate, &exact.digest, cancel)?;
        candidate.installation.launches[0]
            .env
            .insert("BUNDLE_FROZEN".into(), "true".into());
        package::probe(candidate, cancel)
    }

    fn facts(
        &self,
        cache: &Cache,
        candidate: &mut Candidate,
        digest: &str,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let bytes = self
            .helper(
                cache,
                Some(candidate),
                vec![
                    "-e".into(),
                    FACTS.into(),
                    package::path(&candidate.directory)?,
                    candidate.installation.version.clone(),
                ],
            )
            .run(cancel, None)?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "invalid installed gem resolution")?;
        admit(&value, &candidate.installation.version, digest)?;
        candidate.installation.evidence = format!(
            "gem-lock:sha256:{}",
            value["lock_sha256"]
                .as_str()
                .ok_or("Ruby lock digest missing")?
        );
        candidate.installation.runtime = value["runtime"]
            .as_str()
            .ok_or("Ruby runtime identity missing")?
            .into();
        package::write(&candidate.directory.join("gem-packages.json"), &bytes)
    }
}

fn admit(value: &serde_json::Value, version: &str, digest: &str) -> Result<(), String> {
    let packages = value["packages"]
        .as_array()
        .filter(|p| !p.is_empty() && p.len() <= 256)
        .ok_or("gem resolution requires 1..=256 packages")?;
    if !packages
        .iter()
        .any(|p| p["name"] == "ruby-lsp" && p["version"] == version)
    {
        return Err("installed Ruby package differs from the resolved version".into());
    }
    if value["primary_sha256"] != digest
        || !value["lock_sha256"]
            .as_str()
            .is_some_and(super::evidence::sha256)
    {
        return Err("Ruby gem/lock integrity evidence mismatch".into());
    }
    for package in packages {
        if !package["version"].as_str().is_some_and(package::version) {
            return Err("gem resolution lacks exact dependency versions".into());
        }
    }
    Ok(())
}

pub(super) fn runtime_env(directory: &std::path::Path) -> Result<BTreeMap<String, String>, String> {
    let home = package::path(&directory.join("gems"))?;
    Ok([
        ("GEM_HOME".into(), home.clone()),
        ("GEM_PATH".into(), home),
        (
            "BUNDLE_GEMFILE".into(),
            package::path(&directory.join("Gemfile"))?,
        ),
        ("BUNDLE_IGNORE_CONFIG".into(), "1".into()),
        ("BUNDLE_FROZEN".into(), "true".into()),
        ("BUNDLE_DISABLE_VERSION_CHECK".into(), "true".into()),
        (
            "BUNDLE_USER_HOME".into(),
            package::path(&directory.join("bundle-user"))?,
        ),
        ("RUBYGEMS_GEMDEPS".into(), String::new()),
    ]
    .into())
}

fn resolution(bytes: &[u8]) -> Result<Exact, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "invalid RubyGems metadata")?;
    let version = value["version"]
        .as_str()
        .filter(|v| package::version(v))
        .ok_or("RubyGems lacks an exact stable version")?;
    let bundler = value["bundler_version"]
        .as_str()
        .filter(|v| package::version(v))
        .ok_or("installed Ruby requires a concrete Bundler version")?;
    let digest = value["sha"]
        .as_str()
        .filter(|v| super::evidence::sha256(v))
        .ok_or("RubyGems lacks package SHA-256 evidence")?;

    Ok(Exact {
        version: version.into(),
        bundler: bundler.into(),
        digest: digest.into(),
    })
}
