#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "support/language_assets.rs"]
mod support;
mod language_assets {
    pub mod admission;
    pub mod registration;
}

use rev::buffer::Buffers;
use rev::display::Size;
use rev::session::Session;
use rev::view::View;

fn session(snapshot: rev::config::Snapshot, path: &std::path::Path) -> Session {
    let mut buffers = Buffers::with_settings(snapshot).unwrap();
    let id = buffers.visit(path).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 16,
        },
    )
    .unwrap()
}

fn configured(fixture: &fixture::Fixture, config: &str) -> rev::config::Snapshot {
    let root = fixture.write("config.toml", config);
    rev::config::prepare(&root).snapshot
}

fn rust_config(fixture: &fixture::Fixture, parser: &std::path::Path, symbol: &str) -> String {
    let highlights = fixture.write("highlights.scm", "(identifier) @function");
    let completions = fixture.write(
        "completions.scm",
        "(function_item name: (identifier) @function)",
    );
    format!(
        "[language.rust.external]\n{}",
        support::registration(parser, symbol, &highlights, &completions)
    )
}
