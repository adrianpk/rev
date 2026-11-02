//! Explicit, optional managed installation. Registry reads never start work;
//! mutations own one native job slot and OS locks for their entire lifetime.
mod cache;
mod lease;
mod remove;
mod update;
pub(crate) use lease::launch as launch_lease;
pub use remove::{references, remove};
pub use update::Update;
mod download;
mod evidence;
mod go;
mod helper;
mod marksman;
mod npm;
mod ols;
mod package;
mod probe;
mod process;
mod progress;
mod python;
mod record;
mod ruby;
mod runtime;
pub(crate) use runtime::apply as apply_runtime;
mod rust;
mod storage;
mod tree;
mod worker;

pub use go::install_go;
pub use npm::install_npm;
pub use progress::Phase;
pub use record::{Installation, Launch, Registry, State};
pub use rust::install_rust;
pub use storage::{Candidate, Storage};
pub use worker::{Manager, Outcome};

pub const RECORDS: usize = 32;
pub const REGISTRY_BYTES: usize = 2 * 1024 * 1024;
pub const ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;
pub const PAYLOAD_BYTES: u64 = 1024 * 1024 * 1024;
pub const CACHE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const OUTPUT_BYTES: usize = 1024 * 1024;
pub const DISPLAY_BYTES: usize = 64 * 1024;
pub const OPERATION_SECONDS: u64 = 900;

/// Recipes with delivered acquisition behavior. The metadata catalog may also
/// describe later optional recipes without exposing an unimplemented operation.
pub const SUPPORTED_RECIPES: [&str; 8] = [
    "rust-analyzer",
    "gopls",
    "typescript-language-server",
    "vscode-langservers-extracted",
    "marksman",
    "ruby-lsp",
    "python-lsp-server",
    "ols",
];

/// Native management dispatch. Shared multi-language profiles inherit the
/// explicitly chosen language's settings; independent web profiles keep theirs.
pub fn install(
    storage: &mut Storage,
    recipe: &str,
    language: &str,
    mut inherited: std::collections::BTreeMap<String, crate::config::Server>,
    cancel: &crate::org::Cancellation,
) -> Result<Outcome, String> {
    inherit(recipe, language, &mut inherited);
    match recipe {
        "rust-analyzer" => install_rust(storage, inherited.remove("rust"), cancel),
        "gopls" => install_go(storage, inherited.remove("go"), cancel),
        "marksman" => marksman::install(storage, inherited.remove("markdown"), cancel),
        "ruby-lsp" => ruby::install(storage, inherited.remove("ruby"), cancel),
        "python-lsp-server" => python::install(storage, inherited.remove("python"), cancel),
        "ols" => ols::install(storage, inherited.remove("odin"), cancel),
        "typescript-language-server" | "vscode-langservers-extracted" => {
            install_npm(storage, recipe, inherited, cancel)
        }
        _ => Err("unsupported managed recipe".into()),
    }
}

fn inherit(
    recipe: &str,
    language: &str,
    inherited: &mut std::collections::BTreeMap<String, crate::config::Server>,
) {
    if let Some(base) = inherited.get(language).cloned()
        && let Some(profile) = crate::lsp::catalog::RECIPES
            .iter()
            .find(|r| r.id == recipe)
            .and_then(|r| r.launch(language))
    {
        for language in profile.languages {
            inherited.insert((*language).into(), base.clone());
        }
    }
}
