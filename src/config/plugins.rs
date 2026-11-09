use std::collections::BTreeSet;

use rev_plugin_sdk::identity::PackageId;
use serde::Deserialize;

use crate::plugins::declarations::Binding;

/// Explicit startup preferences. Passive manifest data never starts a process.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plugins {
    pub enabled: bool,
    pub manifests: Vec<String>,
    pub startup: Vec<PackageId>,
    pub keybindings: Vec<Binding>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialPlugins {
    pub enabled: Option<bool>,
    pub manifests: Option<Vec<String>>,
    pub startup: Option<Vec<PackageId>>,
    pub keybindings: Option<Vec<Binding>>,
}

impl PartialPlugins {
    pub(super) fn project(&self) -> Plugins {
        Plugins {
            enabled: self.enabled.unwrap_or(false),
            manifests: self.manifests.clone().unwrap_or_default(),
            startup: self.startup.clone().unwrap_or_default(),
            keybindings: self.keybindings.clone().unwrap_or_default(),
        }
    }
}

impl super::merge::Merge {
    pub(super) fn plugins(&mut self, base: &mut PartialPlugins, later: &PartialPlugins) {
        self.leaf(&["plugins", "enabled"], &mut base.enabled, &later.enabled);
        self.leaf(
            &["plugins", "manifests"],
            &mut base.manifests,
            &later.manifests,
        );
        self.leaf(&["plugins", "startup"], &mut base.startup, &later.startup);
        self.leaf(
            &["plugins", "keybindings"],
            &mut base.keybindings,
            &later.keybindings,
        );
    }
}

pub(super) fn validate(settings: &PartialPlugins) -> Result<(), super::validate::Failure> {
    use super::validate::{Failure, Target};
    let fail = |key, message: &str| Failure::at(&["plugins", key], message, Target::Value);
    if let Some(paths) = &settings.manifests {
        if paths.len() > 64
            || paths
                .iter()
                .any(|p| !crate::plugins::declarations::local_path(p))
        {
            return Err(fail(
                "manifests",
                "expected at most 64 explicit nonempty local manifest paths",
            ));
        }
        if paths.iter().collect::<BTreeSet<_>>().len() != paths.len() {
            return Err(fail("manifests", "duplicate manifest path"));
        }
    }
    if let Some(startup) = &settings.startup
        && (startup.len() > 8 || startup.iter().collect::<BTreeSet<_>>().len() != startup.len())
    {
        return Err(fail(
            "startup",
            "expected at most 8 distinct startup package IDs",
        ));
    }
    keybindings(settings)
}

fn keybindings(settings: &PartialPlugins) -> Result<(), super::validate::Failure> {
    use super::validate::{Failure, Target};
    let fail = |message: &str| Failure::at(&["plugins", "keybindings"], message, Target::Value);
    if let Some(bindings) = &settings.keybindings {
        if bindings.len() > 512 {
            return Err(fail("plugin keybindings exceed 512 entries"));
        }
        for binding in bindings {
            binding.validate().map_err(|message| fail(&message))?;
        }
    }
    Ok(())
}
