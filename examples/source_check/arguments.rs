use super::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct Input {
    pub path: PathBuf,
    pub root: bool,
}

#[derive(Debug, Default)]
pub(super) struct Options {
    pub inputs: Vec<Input>,
    pub scopes: Vec<PathBuf>,
    pub clippy_json: Option<PathBuf>,
    pub clippy_status: Option<i32>,
    pub metrics: bool,
    pub print_clippy_flags: bool,
    workspace: bool,
}

impl Options {
    pub fn parse(arguments: impl Iterator<Item = String>) -> Result<Self> {
        let mut options = Self::default();
        let mut arguments = arguments;
        while let Some(flag) = arguments.next() {
            options.argument(&flag, &mut arguments)?;
        }
        options.validate()?;
        Ok(options)
    }

    fn argument(&mut self, flag: &str, arguments: &mut impl Iterator<Item = String>) -> Result<()> {
        match flag {
            "--workspace" => self.workspace = true,
            "--metrics" => self.metrics = true,
            "--print-clippy-flags" => self.print_clippy_flags = true,
            "--scope" => self.scopes.push(value(flag, arguments)?.into()),
            "--root" | "--source" => self.inputs.push(Input {
                path: value(flag, arguments)?.into(),
                root: flag == "--root",
            }),
            _ => self.diagnostic_argument(flag, arguments)?,
        }
        Ok(())
    }

    fn diagnostic_argument(
        &mut self,
        flag: &str,
        arguments: &mut impl Iterator<Item = String>,
    ) -> Result<()> {
        match flag {
            "--clippy-json" if self.clippy_json.is_none() => {
                self.clippy_json = Some(value(flag, arguments)?.into());
            }
            "--clippy-status" if self.clippy_status.is_none() => {
                self.clippy_status = Some(value(flag, arguments)?.parse()?);
            }
            _ => return Err(format!("unknown or repeated argument: {flag}").into()),
        }
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        let workspace = self.workspace || !self.scopes.is_empty() || self.clippy_json.is_some();
        if !self.inputs.is_empty() && workspace {
            return Err("legacy --root/--source cannot be mixed with workspace checks".into());
        }
        if self.clippy_json.is_some() != self.clippy_status.is_some() {
            return Err("--clippy-json and --clippy-status must be supplied together".into());
        }
        if self.print_clippy_flags && (workspace || !self.inputs.is_empty() || self.metrics) {
            return Err("--print-clippy-flags must be used alone".into());
        }
        Ok(())
    }
}

fn value(flag: &str, arguments: &mut impl Iterator<Item = String>) -> Result<String> {
    arguments
        .next()
        .filter(|value| !value.is_empty() && !value.starts_with("--"))
        .ok_or_else(|| format!("missing value for {flag}").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Invalid selectors must fail rather than silently becoming a global or empty check.
    #[test]
    fn arguments() {
        for (name, args, valid) in [
            ("workspace default", vec![], true),
            ("legacy", vec!["--root", "lib.rs"], true),
            ("scope", vec!["--scope", "src"], true),
            ("empty scope", vec!["--scope", ""], false),
            ("missing scope", vec!["--scope"], false),
            ("unknown flag", vec!["--unknown", "src"], false),
            ("mixed", vec!["--root", "lib.rs", "--scope", "src"], false),
            ("missing status", vec!["--clippy-json", "out"], false),
            (
                "mixed profile",
                vec!["--print-clippy-flags", "--root", "lib.rs"],
                false,
            ),
        ] {
            assert_eq!(
                Options::parse(args.into_iter().map(str::to_owned)).is_ok(),
                valid,
                "{name}"
            );
        }
    }
}
