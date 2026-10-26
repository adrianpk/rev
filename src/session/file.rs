use std::io;
use std::path::Path;

use crate::buffer::Buffer;
use crate::view::View;

use super::Session;

impl Session {
    /// Failed ingress changes only the error echo. Existing buffers, history,
    /// point, scroll and input context remain available without lossy replacement.
    pub fn load(&mut self, path: &Path) -> io::Result<()> {
        let result = self.ingress(path);
        if let Err(error) = &result {
            self.echo = error.to_string();
        }
        result
    }

    fn ingress(&mut self, path: &Path) -> io::Result<()> {
        let loaded = crate::file::load(path)?;
        let buffer = Buffer::from_loaded(loaded).map_err(io::Error::other)?;
        let view = View::new(&buffer);
        self.stop_typing().map_err(io::Error::other)?;
        // All recoverable work is complete before registry/view publication.
        self.buffers.insert(buffer);
        self.view = view;
        self.input.cancel();
        self.echo.clear();
        Ok(())
    }
}
