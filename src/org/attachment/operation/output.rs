use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};
/// Only concrete persistence boundaries are injectable. Path observations,
/// staging ownership, collision checks and effect acknowledgements stay native.
pub(in crate::org::attachment) trait Output {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync_directory(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
    }
    fn remove_input(&mut self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}
pub(in crate::org::attachment) struct Files;
impl Output for Files {}
