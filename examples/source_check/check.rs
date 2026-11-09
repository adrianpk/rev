use super::{
    Result,
    arguments::{Input, Options},
    clippy,
    discovery::Workspace,
    inspect,
};
use std::fs;

pub(super) fn run(options: &Options) -> Result<()> {
    if !options.inputs.is_empty() {
        return legacy(options);
    }
    let workspace = Workspace::discover()?;
    let sources = workspace.select(&options.scopes)?;
    let coverage = if options.scopes.is_empty() {
        "full workspace"
    } else {
        "scoped"
    };
    println!(
        "Coverage: {coverage}; {} / {} first-party Rust files, {} packages; {} / {} Cargo roots selected.",
        sources.len(),
        workspace.sources.len(),
        workspace.packages.len(),
        sources.iter().filter(|source| source.root).count(),
        workspace.sources.values().filter(|root| **root).count()
    );
    let clippy = clippy::check(options, &workspace, &sources)?;
    let passed = native(&workspace, &sources, options.metrics)?;
    outcome("Native source policy", passed);
    finish(passed && clippy)
}

fn native(workspace: &Workspace, sources: &[Input], metrics: bool) -> Result<bool> {
    let mut passed = true;
    for input in sources {
        let issues = inspect(&fs::read_to_string(&input.path)?, input.root, metrics)?;
        passed &= issues.is_empty();
        for issue in issues {
            eprintln!("{}:{issue}", workspace.display(&input.path).display());
        }
    }
    Ok(passed)
}

fn legacy(options: &Options) -> Result<()> {
    let mut failed = false;
    for input in &options.inputs {
        let issues = inspect(&fs::read_to_string(&input.path)?, input.root, true)?;
        failed |= !issues.is_empty();
        for issue in issues {
            eprintln!("{}:{issue}", input.path.display());
        }
    }
    println!(
        "Coverage: legacy explicit inputs; {} files; no workspace coverage claim.",
        options.inputs.len()
    );
    outcome("Native source policy", !failed);
    finish(!failed)
}

fn outcome(component: &str, passed: bool) {
    println!(
        "{component}: {} (safe Rust, parameters <= 5, nesting <= 3, cyclomatic <= 10).",
        if passed { "passed" } else { "failed" }
    );
}

fn finish(passed: bool) -> Result<()> {
    if passed {
        Ok(())
    } else {
        Err("workspace quality check failed".into())
    }
}
