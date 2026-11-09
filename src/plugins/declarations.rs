//! Bounded passive metadata. Reading declarations never probes or starts programs.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use rev_plugin_sdk::{
    API,
    identity::*,
    limits::Limits,
    methods::{self, Validate},
};
use serde::Deserialize;

pub const MANIFEST_BYTES: usize = 256 * 1024;

pub(crate) fn local_path(path: &str) -> bool {
    !path.is_empty() && path.len() <= 4096 && !path.contains('\0') && !path.contains("://")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    Global,
    Buffer,
}

/// The same explicit selectors are used by passive defaults and user overrides.
/// Buffer selectors intersect. Their execution belongs to native keymap admission.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub command: CommandId,
    pub keys: methods::Label,
    pub context: methods::Context,
    pub scope: Scope,
    #[serde(default)]
    pub filetypes: methods::List<methods::Label, 64>,
    pub buffer_owner: Option<PackageId>,
}
impl Binding {
    pub fn validate(&self) -> Result<(), String> {
        crate::input::keymap::Sequence::parse(self.keys.as_str()).map_err(|e| e.to_string())?;
        let selectors = !self.filetypes.is_empty() || self.buffer_owner.is_some();
        if (self.scope == Scope::Buffer) != selectors {
            return Err("buffer scope requires selectors; global scope rejects selectors".into());
        }
        if self
            .filetypes
            .iter()
            .any(|f| !crate::config::language_id(f.as_str()))
            || self
                .filetypes
                .iter()
                .map(|f| f.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != self.filetypes.len()
        {
            return Err("expected distinct native filetype identities".into());
        }
        // Binding installation and prefix/conflict resolution are deliberately
        // deferred until the native contribution owner exists (Slice 7).
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub id: PackageId,
    pub version: methods::Label,
    pub api: methods::Label,
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Process {
    pub program: methods::Label,
    #[serde(default)]
    pub args: methods::List<methods::Label, 64>,
}
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Activation {
    pub filetypes: methods::List<methods::Label, 64>,
    pub events: methods::List<methods::EventKind, 3>,
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub name: CommandName,
    pub description: methods::Label,
    #[serde(default)]
    pub parameters: methods::List<methods::Parameter, 64>,
    pub prefix: methods::PrefixPolicy,
    pub recording: methods::RecordingPolicy,
}
impl Command {
    pub fn declaration(&self) -> methods::CommandDeclaration {
        methods::CommandDeclaration {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            prefix: self.prefix,
            recording: self.recording,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub plugin: Identity,
    pub process: Process,
    #[serde(default)]
    pub commands: methods::List<Command, 64>,
    #[serde(default)]
    pub bindings: methods::List<Binding, 128>,
    #[serde(default)]
    pub activation: Activation,
}
impl Manifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.plugin.api.as_str() != API {
            return Err(format!("unsupported_api: expected {API}"));
        }
        let version = self.plugin.version.as_str();
        if version.is_empty()
            || version.len() > 128
            || !version.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err("package version requires 1..=128 visible ASCII bytes".into());
        }
        if !local_path(self.process.program.as_str())
            || self.process.args.iter().any(|a| a.as_str().contains('\0'))
            || self
                .process
                .args
                .iter()
                .map(|a| a.as_str().len())
                .sum::<usize>()
                > 65536
        {
            return Err(
                "invalid local program or literal argv; arguments exceed 64 KiB or contain NUL"
                    .into(),
            );
        }
        let mut names = BTreeSet::new();
        for command in self.commands.iter() {
            command
                .declaration()
                .validate(&Limits::default())
                .map_err(|e| e.to_string())?;
            if command.description.as_str().chars().any(char::is_control)
                || !names.insert(&command.name)
            {
                return Err("commands require distinct names and single-line descriptions".into());
            }
        }
        for binding in self.bindings.iter() {
            binding.validate()?;
            if !binding.command.belongs_to(&self.plugin.id)
                || !self
                    .commands
                    .iter()
                    .any(|c| self.plugin.id.command(&c.name) == binding.command)
            {
                return Err("binding command is not declared in this package namespace".into());
            }
        }
        let activation = &self.activation;
        if activation
            .filetypes
            .iter()
            .any(|f| !crate::config::language_id(f.as_str()))
            || activation
                .filetypes
                .iter()
                .map(|f| f.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != activation.filetypes.len()
            || activation.events.iter().collect::<BTreeSet<_>>().len() != activation.events.len()
            || activation.events.contains(&methods::EventKind::Resnapshot)
        {
            return Err(
                "activation requires distinct native filetypes and opened/changed/closed events"
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    pub manifest: Manifest,
    pub source: PathBuf,
    pub program: PathBuf,
}

/// One atomically admitted startup catalog; disabled configuration never reads manifests.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Registry {
    packages: BTreeMap<PackageId, Package>,
}
impl Registry {
    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        self.packages.values()
    }
    pub fn package(&self, id: &str) -> Option<&Package> {
        self.packages
            .values()
            .find(|p| p.manifest.plugin.id.as_str() == id)
    }
    pub fn command(&self, id: &str) -> Option<(&Package, &Command)> {
        self.packages.values().find_map(|p| {
            p.manifest
                .commands
                .iter()
                .find(|c| p.manifest.plugin.id.command(&c.name).as_str() == id)
                .map(|c| (p, c))
        })
    }
    pub fn load(settings: &crate::config::Plugins, origin: &Path) -> Result<Self, Error> {
        if !settings.enabled {
            return Ok(Self::default());
        }
        if settings.manifests.len() > 64
            || settings.startup.len() > 8
            || settings.keybindings.len() > 512
        {
            return Err(Error::new(
                origin,
                "plugin configuration exceeds finite declaration limits",
            ));
        }
        for binding in &settings.keybindings {
            binding
                .validate()
                .map_err(|message| Error::new(origin, &message))?;
        }
        let origin = if origin.is_absolute() {
            origin.to_owned()
        } else {
            std::env::current_dir()
                .map_err(|e| Error::new(origin, &format!("resolve configuration origin: {e}")))?
                .join(origin)
        };
        let directory = origin.parent().unwrap_or(Path::new("."));
        let mut registry = Self::default();
        let mut commands = 0;
        let mut bindings = 0;
        for path in &settings.manifests {
            if !local_path(path) {
                return Err(Error::new(
                    &origin,
                    "expected an explicit local manifest path",
                ));
            }
            let path = directory.join(path);
            let manifest = read(&path).map_err(|message| Error::new(&path, &message))?;
            let program = path
                .parent()
                .unwrap_or(Path::new("."))
                .join(manifest.process.program.as_str());
            commands += manifest.commands.len();
            bindings += manifest.bindings.len();
            if commands > 512 || bindings > 512 {
                return Err(Error::new(
                    &path,
                    "aggregate plugin commands/bindings exceed 512",
                ));
            }
            let id = manifest.plugin.id.clone();
            if registry
                .packages
                .insert(
                    id,
                    Package {
                        manifest,
                        source: path.clone(),
                        program,
                    },
                )
                .is_some()
            {
                return Err(Error::new(
                    &path,
                    "namespace_conflict: duplicate package identity",
                ));
            }
        }
        for id in &settings.startup {
            if !registry.packages.contains_key(id) {
                return Err(Error::new(&origin, "startup package is not declared"));
            }
        }
        for binding in &settings.keybindings {
            if registry.command(binding.command.as_str()).is_none() {
                return Err(Error::new(
                    &origin,
                    "user plugin binding command is not declared",
                ));
            }
        }
        Ok(registry)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub source: PathBuf,
    pub message: String,
}
impl Error {
    fn new(source: &Path, message: &str) -> Self {
        Self {
            source: source.to_owned(),
            message: message.to_owned(),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.source.display(), self.message)
    }
}
impl std::error::Error for Error {}

fn read(path: &Path) -> Result<Manifest, String> {
    let descriptor = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| format!("read manifest: {e}"))?;
    let mut file = File::from(descriptor);
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MANIFEST_BYTES as u64 {
        return Err("expected a regular manifest of at most 256 KiB".into());
    }
    let mut bytes = Vec::new();
    file.by_ref()
        .take((MANIFEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e: io::Error| e.to_string())?;
    if bytes.len() > MANIFEST_BYTES {
        return Err("manifest exceeds 256 KiB".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| "manifest must be UTF-8")?;
    let manifest: Manifest =
        toml::from_str(text).map_err(|e| format!("invalid manifest: {}", e.message()))?;
    manifest.validate()?;
    Ok(manifest)
}
