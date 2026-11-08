mod screen;
pub(super) use screen::Screen;
use std::io;
use std::path::Path;

use crate::buffer::Buffers;
use crate::config::Startup;
use crate::view::{Size, View};

use super::Session;

impl Session {
    /// Discover and freeze user settings before creating any buffer or input
    /// owner. Library `empty`/`new` constructors retain explicit host ownership.
    pub fn startup(path: Option<&Path>, size: Size) -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let path = path.map(|path| {
            if path.is_absolute() {
                path.to_owned()
            } else {
                cwd.join(path)
            }
        });
        let startup = Startup::provision();
        let buffers = Buffers::with_settings(startup.snapshot).map_err(io::Error::other)?;
        let view = View::with_settings(
            buffers.get(buffers.initial()).map_err(io::Error::other)?,
            buffers.snapshot(),
        );
        // Admit the actual startup target before observing a visible root;
        // configured visibility must not enumerate an unrelated launch cwd.
        let mut session = Self::from_buffers(buffers, view, size).map_err(io::Error::other)?;
        session.startup_bookmarks();
        session.startup_contexts();
        session.startup_file(path.as_deref())?;
        if path.is_some() {
            session.tree.present()?;
        } else {
            session.dashboard_open();
        }
        if let Some(message) = session.tree.diagnostic() {
            session.notice(message);
        }
        session.follow_tree();
        session.startup_clock_state();
        session.prepare_bookmark_markers();
        let usage_diagnostic = session.startup_command_usage();
        let library_diagnostic = session.startup_macro_library();
        let diagnostic = match (startup.diagnostic, usage_diagnostic) {
            (Some(config), Some(usage)) => Some(format!("Configuration: {config}; {usage}")),
            (Some(config), None) => Some(format!("Configuration: {config}")),
            (None, usage) => usage,
        };
        let diagnostic = [diagnostic, library_diagnostic]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("; ");
        if !diagnostic.is_empty() {
            session.notice(&diagnostic);
        }
        Ok(session)
    }

    fn startup_file(&mut self, path: Option<&Path>) -> io::Result<()> {
        let Some(path) = path else {
            return Ok(());
        };
        self.load(path)?;
        Ok(())
    }
}
