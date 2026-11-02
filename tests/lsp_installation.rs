#![forbid(unsafe_code)]
#[path = "lsp_installation/catalog.rs"]
mod catalog;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "lsp_installation/routing.rs"]
mod routing;
#[path = "lsp_installation/selectors.rs"]
mod selectors;
#[path = "lsp_installation/settings.rs"]
mod settings;
#[path = "lsp_installation/workflows.rs"]
mod workflows;

#[path = "lsp_installation/storage.rs"]
mod storage;

#[path = "lsp_installation/acquisition.rs"]
mod acquisition;

#[path = "lsp_installation/managed.rs"]
mod managed;

#[path = "lsp_installation/go.rs"]
mod go;

#[path = "lsp_installation/npm.rs"]
mod npm;

#[path = "lsp_installation/live.rs"]
mod live;

#[path = "lsp_installation/optional.rs"]
mod optional;
