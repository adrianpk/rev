use super::{
    Result,
    arguments::{Input, Options},
    discovery::Workspace,
    profile,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
enum Message {
    #[serde(rename = "compiler-message")]
    Compiler {
        package_id: String,
        message: Diagnostic,
    },
    BuildFinished {
        success: bool,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Diagnostic {
    level: String,
    message: String,
    code: Option<Code>,
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Code {
    code: String,
}

#[derive(Deserialize)]
struct Span {
    file_name: PathBuf,
    line_start: usize,
    is_primary: bool,
}

#[derive(Default)]
struct Admission {
    blocking: usize,
    deferred: BTreeMap<String, usize>,
    finished: Option<bool>,
}

pub(super) fn check(options: &Options, workspace: &Workspace, sources: &[Input]) -> Result<bool> {
    let Some(path) = &options.clippy_json else {
        println!("Clippy: not run (native source check only).");
        return Ok(true);
    };
    let mut reader = BufReader::new(File::open(path)?);
    let admission = read(&mut reader, workspace, sources)?;
    let success = admission.passed(options.clippy_status.unwrap_or(1));
    for (code, count) in &admission.deferred {
        println!("Clippy deferred outside selected sources: {code}: {count}");
    }
    let coverage = if options.scopes.is_empty() {
        "full workspace"
    } else {
        "scoped admission"
    };
    println!(
        "Clippy ({coverage}): {}; {} blocking diagnostics, {} selected-profile diagnostics deferred.",
        if success { "passed" } else { "failed" },
        admission.blocking,
        admission.deferred.values().sum::<usize>()
    );
    Ok(success)
}

fn read(reader: &mut impl BufRead, workspace: &Workspace, sources: &[Input]) -> Result<Admission> {
    let mut admission = Admission::default();
    let mut line = String::new();
    let mut records = 0;
    while record(reader, &mut line)? {
        records += 1;
        if records > 100_000 {
            return Err("Clippy exceeded 100000 JSON records".into());
        }
        admission.message(serde_json::from_str(&line)?, workspace, sources)?;
    }
    if admission.finished.is_none() {
        return Err("Clippy did not emit a build-finished result".into());
    }
    Ok(admission)
}

fn record(reader: &mut impl BufRead, line: &mut String) -> Result<bool> {
    const MAX: u64 = 8 * 1024 * 1024;
    line.clear();
    let bytes = reader.take(MAX + 1).read_line(line)?;
    if u64::try_from(bytes)? > MAX {
        return Err("Clippy JSON record exceeded 8 MiB".into());
    }
    Ok(bytes > 0)
}

impl Admission {
    fn message(
        &mut self,
        message: Message,
        workspace: &Workspace,
        sources: &[Input],
    ) -> Result<()> {
        match message {
            Message::Compiler {
                package_id,
                message,
            } if workspace.packages.contains(&package_id) => {
                self.diagnostic(&message, workspace, sources);
            }
            Message::BuildFinished { success } => self.build_finished(success)?,
            _ => {}
        }
        Ok(())
    }

    fn build_finished(&mut self, success: bool) -> Result<()> {
        if self.finished.is_some() {
            return Err("Clippy emitted multiple build-finished records".into());
        }
        self.finished = Some(success);
        Ok(())
    }

    fn diagnostic(&mut self, diagnostic: &Diagnostic, workspace: &Workspace, sources: &[Input]) {
        if !["warning", "error"].contains(&diagnostic.level.as_str()) {
            return;
        }
        let code = diagnostic
            .code
            .as_ref()
            .map_or("compiler", |code| &code.code);
        if diagnostic.level == "warning"
            && profile::selected(code)
            && outside(diagnostic, workspace, sources)
        {
            *self.deferred.entry(code.to_owned()).or_default() += 1;
            return;
        }
        self.blocking += 1;
        let location = diagnostic
            .spans
            .iter()
            .find(|span| span.is_primary)
            .map_or_else(
                || "workspace".into(),
                |span| {
                    format!(
                        "{}:{}",
                        workspace.display(&span.file_name).display(),
                        span.line_start
                    )
                },
            );
        eprintln!("{location}: {code}: {}", diagnostic.message);
    }

    fn passed(&self, status: i32) -> bool {
        status == 0 && self.finished == Some(true) && self.blocking == 0
    }
}

fn outside(diagnostic: &Diagnostic, workspace: &Workspace, sources: &[Input]) -> bool {
    let primary: Vec<_> = diagnostic
        .spans
        .iter()
        .filter(|span| span.is_primary)
        .collect();
    !primary.is_empty()
        && primary.iter().all(|span| {
            let path = workspace.root.join(&span.file_name);
            // A missing/unmapped location cannot safely defer a selected diagnostic.
            path.canonicalize().is_ok_and(|path| {
                workspace.sources.contains_key(&path) && !selected(&path, sources)
            })
        })
}

fn selected(path: &Path, sources: &[Input]) -> bool {
    sources.iter().any(|source| source.path == path)
}

#[cfg(test)]
#[path = "clippy_tests.rs"]
mod tests;
