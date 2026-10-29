use super::*;
use crate::org::io::process::Program;
use std::path::Path;
pub(crate) struct Upload {
    program: Program,
}
impl Upload {
    pub fn configured(snapshot: &config::Snapshot, output: &Path) -> Result<Self, String> {
        let policy = &snapshot.settings().org;
        let mut argv = policy.tools.upload.clone();
        config::publishing::upload(&argv)?;
        if argv.is_empty() {
            return Err("No upload tool configured; set org.tools.upload to explicit argv".into());
        }
        let executable = PathBuf::from(&argv[0]);
        if executable.is_relative()
            && argv[0].contains('/')
            && let Some(base) = snapshot
                .origin(&["org", "tools", "upload"])
                .and_then(Path::parent)
        {
            argv[0] = base
                .join(executable)
                .to_str()
                .ok_or("Upload executable requires UTF-8")?
                .into();
        }
        for token in &mut argv {
            if token == "{output_dir}" {
                *token = output
                    .to_str()
                    .ok_or("Upload output path requires UTF-8")?
                    .into();
            }
        }
        Ok(Self {
            program: Program {
                argv,
                directory: output.into(),
                seconds: policy.limits.job_seconds,
                output_bytes: policy.limits.job_output_bytes,
            },
        })
    }
    pub fn label(&self) -> &str {
        &self.program.argv[0]
    }
    pub fn run(mut self, cancel: &Cancellation) -> Result<String, String> {
        let directory =
            crate::org::attachment::directory::Directory::observe(&self.program.directory)?
                .open()?;
        directory.current()?;
        self.program.directory = directory.entry(std::ffi::OsStr::new("."));
        let outcome = self.program.run(cancel)?;
        if !outcome.status.success() {
            return Err(format!(
                "Upload failed ({}): {}",
                outcome.status,
                String::from_utf8_lossy(&outcome.stderr)
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(512)
                    .collect::<String>()
            ));
        }
        Ok("Project upload completed".into())
    }
}
