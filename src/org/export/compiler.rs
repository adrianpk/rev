//! Explicit optional PDF jobs use the existing bounded process owner.
//! Configured programs retain host authority; this adapter is not a sandbox.
mod scratch;
use super::{output::Prepared, *};
use crate::{config, org::io::process::Program};
use std::path::{Path, PathBuf};
pub(crate) struct Compiler {
    argv: Vec<String>,
    seconds: usize,
    output_bytes: usize,
}
pub(crate) struct Compilation {
    prepared: Prepared,
    compiler: Compiler,
}
pub(crate) fn argv(argv: &[String]) -> Result<(), String> {
    if argv.len() > 64
        || argv
            .iter()
            .any(|s| s.is_empty() || s.len() > 4096 || s.chars().any(char::is_control))
    {
        return Err("PDF argv requires at most 64 nonempty tokens, 4096 bytes each".into());
    }
    if argv.is_empty() {
        return Ok(());
    }
    substitutions(argv)
}
fn substitutions(argv: &[String]) -> Result<(), String> {
    let mut counts = [0usize; 3];
    for (i, token) in argv.iter().enumerate() {
        let slot = match token.as_str() {
            "{input}" => Some(0),
            "{output}" => Some(1),
            "{output_dir}" => Some(2),
            _ => None,
        };
        if let Some(slot) = slot {
            if i == 0 {
                return Err("PDF executable cannot be a placeholder".into());
            }
            counts[slot] += 1;
        } else if token.contains(['{', '}']) {
            return Err(
                "PDF substitutions must be whole {input}, {output} or {output_dir} tokens".into(),
            );
        }
    }
    if counts[0] != 1 || counts[1..].iter().any(|n| *n > 1) {
        return Err(
            "PDF argv needs exactly one {input} and at most one {output} and {output_dir}".into(),
        );
    }
    Ok(())
}
impl Compiler {
    pub(crate) fn configured(snapshot: &config::Snapshot) -> Result<Self, String> {
        let policy = &snapshot.settings().org;
        let mut argv = policy.tools.pdf.clone();
        self::argv(&argv)?;
        if argv.is_empty() {
            return Err("No PDF compiler configured; set org.tools.pdf to explicit argv".into());
        }
        executable(&mut argv, snapshot)?;
        Ok(Self {
            argv,
            seconds: policy.limits.job_seconds,
            output_bytes: policy.limits.job_output_bytes,
        })
    }
    pub(crate) fn label(&self) -> &str {
        &self.argv[0]
    }
    fn program(self, scratch: &scratch::Scratch) -> Result<Program, String> {
        let argv = self
            .argv
            .into_iter()
            .map(|token| scratch.substitute(token))
            .collect::<Result<_, _>>()?;
        Ok(Program {
            argv,
            directory: scratch.root().into(),
            seconds: self.seconds,
            output_bytes: self.output_bytes,
        })
    }
}
fn executable(argv: &mut [String], snapshot: &config::Snapshot) -> Result<(), String> {
    let path = PathBuf::from(&argv[0]);
    if path.is_relative()
        && argv[0].contains('/')
        && let Some(root) = snapshot
            .origin(&["org", "tools", "pdf"])
            .and_then(Path::parent)
    {
        argv[0] = root
            .join(path)
            .to_str()
            .ok_or("PDF executable path is not UTF-8")?
            .into();
    }
    Ok(())
}
impl Compilation {
    pub(crate) fn new(prepared: Prepared, compiler: Compiler) -> Self {
        Self { prepared, compiler }
    }
    pub(super) fn run(mut self, cancel: &Cancellation) -> Result<Prepared, String> {
        self.prepared.protect(cancel)?;
        let mut scratch = scratch::Scratch::new()?;
        let result = self.compile(&scratch, cancel);
        let cleanup = scratch.clean();
        cleanup?;
        self.prepared.binary = Some(result?);
        self.prepared.protect(cancel)?;
        Ok(self.prepared)
    }
    fn compile(
        &self,
        scratch: &scratch::Scratch,
        cancel: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        scratch.input(&self.prepared.rendered.text, cancel)?;
        let program = Compiler {
            argv: self.compiler.argv.clone(),
            seconds: self.compiler.seconds,
            output_bytes: self.compiler.output_bytes,
        }
        .program(scratch)?;
        let outcome = program.run(cancel)?;
        if !outcome.status.success() {
            let detail = String::from_utf8_lossy(&outcome.stderr)
                .chars()
                .filter(|c| !c.is_control())
                .take(512)
                .collect::<String>();
            return Err(format!(
                "PDF compiler failed ({}): {detail}",
                outcome.status
            ));
        }
        scratch.output(self.prepared.snapshot.export_bytes, cancel)
    }
}
