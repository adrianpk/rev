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
        let startup = Startup::provision();
        let buffers = Buffers::with_settings(startup.snapshot).map_err(io::Error::other)?;
        let view = View::with_settings(
            buffers.get(buffers.initial()).map_err(io::Error::other)?,
            buffers.snapshot(),
        );
        let mut session = Self::new(buffers, view, size).map_err(io::Error::other)?;
        session.startup_file(path)?;
        session.startup_clock_state();
        if let Some(diagnostic) = startup.diagnostic {
            session.notice(&format!("Configuration: {diagnostic}"));
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
