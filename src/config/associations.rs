use std::path::Path;

use super::Snapshot;

const BUILTINS: [(&str, &str); 28] = [
    ("*.rs", "rust"),
    ("*.org", "org"),
    ("*.org_archive", "org"),
    ("*.go", "go"),
    ("*.rb", "ruby"),
    ("Gemfile", "ruby"),
    ("Rakefile", "ruby"),
    ("*.c", "c"),
    ("*.h", "c"),
    ("*.cc", "cpp"),
    ("*.cpp", "cpp"),
    ("*.cxx", "cpp"),
    ("*.hpp", "cpp"),
    ("*.py", "python"),
    ("*.html", "html"),
    ("*.htm", "html"),
    ("*.css", "css"),
    ("*.js", "javascript"),
    ("*.mjs", "javascript"),
    ("*.cjs", "javascript"),
    ("*.ts", "typescript"),
    ("*.tsx", "typescript"),
    ("*.json", "json"),
    ("*.toml", "toml"),
    ("Cargo.toml", "toml"),
    ("*.md", "markdown"),
    ("*.sh", "shell"),
    ("Makefile", "make"),
];

impl Snapshot {
    /// Known explicit and built-in patterns, without grammar or filesystem IO.
    pub fn patterns(&self, language: &str) -> Vec<String> {
        let mut patterns: std::collections::BTreeMap<String, String> = BUILTINS
            .into_iter()
            .map(|(pattern, id)| (pattern.into(), id.into()))
            .collect();
        patterns.extend(self.settings().files.associations.clone());
        patterns
            .into_iter()
            .filter_map(|(pattern, id)| (id == language).then_some(pattern))
            .collect()
    }
    /// Identify the supplied visiting basename without IO or canonicalization.
    /// Any user match precedes built-ins; exact names precede suffixes within
    /// each class, then the longest suffix wins. Unknown names are plain.
    /// Identity does not require a language table or a bundled grammar.
    pub fn language(&self, path: &Path) -> &str {
        let Some(name) = path.file_name() else {
            return "plain";
        };
        let name = name.as_encoded_bytes();
        let user = self
            .settings()
            .files
            .associations
            .iter()
            .map(|(pattern, id)| (pattern.as_str(), id.as_str()));
        select(name, user)
            .or_else(|| select(name, BUILTINS.into_iter()))
            .unwrap_or("plain")
    }
}

fn select<'a>(name: &[u8], rules: impl Iterator<Item = (&'a str, &'a str)>) -> Option<&'a str> {
    rules
        .filter_map(|(pattern, id)| specificity(name, pattern).map(|rank| (rank, id)))
        .max_by_key(|(rank, _)| *rank)
        .map(|(_, id)| id)
}

fn specificity(name: &[u8], pattern: &str) -> Option<usize> {
    match pattern.strip_prefix('*') {
        Some(suffix) if name.ends_with(suffix.as_bytes()) => Some(suffix.len()),
        None if name == pattern.as_bytes() => Some(usize::MAX),
        _ => None,
    }
}
